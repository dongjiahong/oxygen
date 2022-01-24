use chrono::prelude::*;
use color_eyre::eyre::{eyre, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use dasp::{interpolate::linear::Linear, signal, Signal};
use std::sync::{mpsc::Sender, Arc, Mutex};

/// Raw mono audio data.
#[derive(Clone)]
pub struct AudioClip {
    pub id: Option<usize>,
    pub name: String,
    pub date: DateTime<Utc>,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioClip {
    // 重新采样
    pub fn resample(&self, sample_rate: u32) -> AudioClip {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();
        let interp = Linear::new(a, b);

        AudioClip {
            id: self.id,
            name: self.name.clone(),
            date: self.date,
            samples: signal
                .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
                .take(self.samples.len() * (sample_rate as usize) / (self.sample_rate as usize))
                .collect(),
            sample_rate,
        }
    }

    pub fn record(name: String) -> Result<AudioClip> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| eyre!("No input device"))?;
        println!("Input device: {}", device.name()?);
        let config = device.default_input_config()?;

        let clip = AudioClip {
            id: None,
            name,
            date: Utc::now(),
            samples: Vec::new(),
            sample_rate: config.sample_rate().0,
        };

        let clip = Arc::new(Mutex::new(Some(clip)));
        let clip2 = clip.clone();

        println!("Begin recording...");
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let channels = config.channels();
        type ClipHandle = Arc<Mutex<Option<AudioClip>>>;

        fn write_input_data<T>(input: &[T], channels: u16, writer: &ClipHandle)
        where
            T: cpal::Sample,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(clip) = guard.as_mut() {
                    for frame in input.chunks(channels.into()) {
                        clip.samples.push(frame[0].to_f32());
                    }
                }
            }
        }

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(&config.into(),move |data, _: &_| write_input_data::<f32>(data,channels,&clip_2),err_fn)?,
        }
    }
}
