use audiopus::{
    coder::Encoder, Bitrate, Channels, Error as OpusError, ErrorCode as OpusErrorCode, SampleRate,
};
use color_eyre::{eyre::eyre, Result};

use crate::audio_clip::AudioClip;

pub fn decode_v0(bytes: &[u8]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks(4) {
        samples.push(f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    samples
}

/// Encode a clip into a custom opus container.
///
/// Format is:
///  - 4 bytes, number of samples as a u32 in big endian
///  - for each packet:
///     - 2 bytes, number of bytes in packet as a u16 in big endian
///     - the raw packet
pub fn encode_v1(clip: &AudioClip) -> Result<(u32, Vec<u8>)> {
    let sample_rate: i32 = clip.sample_rate.try_into()?;
    let resampled: AudioClip;
    let (samples, sample_rate) = match SampleRate::try_from(sample_rate) {
        Ok(sample_rate) => (&clip.samples, sample_rate),
        Err(_) => {
            resampled = clip.resample(48000);
            (&resampled.samples, SampleRate::Hz48000)
        }
    };

    let mut encoder = Encoder::new(sample_rate, Channels::Mono, audiopus::Application::Audio)?;
    encoder.set_bitrate(Bitrate::BitsPerSecond(24000))?;

    let frame_size = (sample_rate as i32 / 1000 * 20) as usize;

    let mut output = vec![0u8; samples.len().max(128)];
    let mut samples_i = 0;
    let mut output_i = 0;
    let mut end_buffer = vec![0f32; frame_size];

    // Store number of samples.
    {
        let samples: u32 = samples.len().try_into()?;
        let bytes = samples.to_be_bytes();
        output[..4].clone_from_slice(&bytes[..4]);
        output_i += 4;
    }

    while samples_i < samples.len() {
        match encoder.encode_float(
            if samples_i + frame_size < samples.len() {
                &samples[samples_i..(samples_i + frame_size)]
            } else {
                end_buffer[..(samples.len() - samples_i)].clone_from_slice(
                    &samples[samples_i..((samples.len() - samples_i) + samples_i)],
                );
                &end_buffer
            },
            &mut output[output_i + 2..],
        ) {
            Ok(pkt_len) => {
                samples_i += frame_size;
                let bytes = u16::try_from(pkt_len)?.to_be_bytes();
                output[output_i] = bytes[0];
                output[output_i + 1] = bytes[1];
                output_i += pkt_len + 2;
            }
            Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                eprintln!(
                    "Needed to increase buffer size, opus is compressing less well than expected."
                );
                output.resize(output.len() * 2, 0u8);
            }
            Err(e) => {
                return Err(eyre!(e));
            }
        }
    }

    output.truncate(output_i);

    Ok((sample_rate as i32 as u32, output))
}