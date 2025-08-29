mod generator;

use anyhow::{Result, bail};
use clap::{Parser, value_parser};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::{
    io::{Error, ErrorKind},
    path::{Display, PathBuf},
};
use utils::{format_duration, handle_walkdir_error, scan_video_from_path};

#[derive(Parser)]
#[command(version,about = "Batch video thumbnail generation", long_about = None, next_line_help = true)]
struct Cli {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    pub inputs: Vec<PathBuf>,
    #[arg(short, long, default_value_t = 1,value_parser = value_parser!(u8).range(1..), help = "folder recursive depth")]
    pub depth: u8,
    #[arg(short, long, default_value_t = 200, help = "base thumbnail grid size")]
    pub length: u16,
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
    Ok(generate_thumbnails(&videos, cli.length))
}

fn generate_thumbnails(videos: &[PathBuf], base_dim: u16) -> bool {
    let mut errors = vec![];

    let pb = setup_progress_bar(
        videos
            .len()
            .try_into()
            .expect("[ERROR] Video count too large"),
    );

    pb.set_position(0);
    pb.set_message(format!("{}", videos.first().unwrap().display()));

    videos.iter().enumerate().for_each(|(index, video)| {
        match generator::generate_thumbnail(video, base_dim) {
            Ok(_) if index + 1 < videos.len() => {
                pb.inc(1);
                pb.set_message(format!("{}", videos[index + 1].display()));
            }
            Ok(_) => pb.inc(1),
            Err(e) => errors.push(e),
        }
    });

    let success_count = pb.position();
    let duration = pb.elapsed();

    pb.finish_and_clear();

    errors.iter().for_each(|e| log::error!("{e}"));

    log::info!(
        "generated {success_count} thumbnails in {} with {} failures",
        format_duration(duration),
        errors.len(),
    );

    !errors.is_empty()
}

fn setup_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_draw_target(ProgressDrawTarget::stderr_with_hz(4));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner} [{elapsed_precise}] [{bar:40}] {percent}% ({eta}) {pos}/{len} | {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
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
