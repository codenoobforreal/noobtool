mod constants;
mod encoder;
mod progress;

use crate::encoder::process_encode;
use anyhow::{Result, bail};
use clap::{Parser, value_parser};
use std::{
    io::{Error, ErrorKind},
    path::{Display, PathBuf},
};
use utils::{handle_walkdir_error, scan_video_from_path};

#[derive(Parser)]
#[command(version,about = "Batch HEVC video encoding", long_about = None,next_line_help = true)]
struct Cli {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    inputs: Vec<PathBuf>,
    #[arg(short, long, default_value_t = 1,value_parser = value_parser!(u8).range(1..), help = "folder recursive depth")]
    depth: u8,
}

pub fn run() -> Result<bool> {
    let cli = Cli::parse();
    let inputs_len = cli.inputs.len();
    log::info!("enter {inputs_len} paths");
    let videos = scan_videos(cli.inputs, cli.depth);

    if videos.is_empty() {
        bail!("no video found in all your inputs");
    }
    log::info!("found {} videos", videos.len());
    Ok(batch_encode(&videos))
}

fn batch_encode(videos: &[PathBuf]) -> bool {
    videos.iter().fold(false, |mut has_error, video| {
        if let Err(e) = process_encode(video) {
            log::error!("{e}");
            has_error = true;
        }
        has_error
    })
}

fn scan_videos(inputs: Vec<PathBuf>, depth: u8) -> Vec<PathBuf> {
    inputs
        .iter()
        .fold(Vec::with_capacity(inputs.len() * 5), |mut videos, input| {
            let display_path = input.display();
            match scan_video_from_path(input, depth) {
                Ok((success_paths, error_paths)) => {
                    videos.extend(success_paths);
                    handle_scan_errors(&error_paths);
                }
                Err(e) => handle_input_error(&display_path, e),
            }
            videos
        })
}

fn handle_scan_errors(error_paths: &[walkdir::Error]) {
    error_paths.iter().for_each(|error| {
        let (error_path, cause) = handle_walkdir_error(error);
        log::warn!("access {} failed: {}", error_path, cause);
    });
}

fn handle_input_error(display_path: &Display, e: Error) {
    match e.kind() {
        ErrorKind::NotFound => {
            log::warn!("{} not found", display_path);
        }
        ErrorKind::PermissionDenied => {
            log::warn!("{} no permission", display_path);
        }
        _ => {
            log::warn!("{} unknown error: {}", display_path, e);
        }
    }
}
