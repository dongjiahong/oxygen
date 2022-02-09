mod audio_clip;
mod db;
mod internal_encoding;

use audio_clip::AudioClip;
use chrono::prelude::*;
use clap::{AppSettings, Parser, Subcommand};
use color_eyre::eyre::Result;
use db::Db;

#[derive(Parser, Debug)]
#[clap(name = "oxygen")]
#[clap(
    about = "A voice journal and audio analysis toolkit for people who want to change the way their voice comes across."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Record {
        name: Option<String>,
    },

    List {},

    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Play {
        name: String,
    },

    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Delete {
        name: String,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let db = Db::open()?;

    match args.command {
        Commands::Record { name } => {
            let name = name.unwrap_or_else(|| Local::now().format("%Y-%m-%d-%H:%M:%S").to_string());
            let mut clip = AudioClip::record(name)?;
            db.save(&mut clip)?;
        }
        Commands::List {} => {
            println!("{:5} {:30} {:30}", "id", "name", "date");
            for entry in db.list()? {
                println!(
                    "{:5} {:30} {:30}",
                    entry.id,
                    entry.name,
                    entry
                        .date
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                )
            }
        }
        Commands::Play { name } => {
            if let Some(clip) = db.load(&name)? {
                clip.play()?
            } else {
                eprintln!("No such clip.");
            }
        }
        Commands::Delete { name } => {
            db.delete(&name)?;
        }
    }

    Ok(())
}
