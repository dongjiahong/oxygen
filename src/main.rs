mod audio_clip;
mod db;

use clap::{AppSettings, Parser, Subcommand};

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
    let db = Db::parse()?;
}
