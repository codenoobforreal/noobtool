mod encode_video;
mod generate_video_thumbnail;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use env_logger::{Env, WriteStyle};
use std::{io::Write, process};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .write_style(WriteStyle::Never)
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    match run() {
        // has_error
        Ok(true) => process::exit(1),
        Ok(_) => process::exit(0),
        Err(e) => {
            log::error!("{}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::EncodeVideo(args) => Ok(encode_video::run(args)?),
        Commands::GenerateVideoThumbnail(args) => Ok(generate_video_thumbnail::run(args)?),
    }
}
