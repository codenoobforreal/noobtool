use anyhow::{Result, bail};
use cli::EncodeVideoArgs;
use ffmpeg_progress_monitor::ProgressMonitor;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use utils::{format_file_size, scan_videos_from_paths};
use video_encoder::{Config, Encoder};
use video_metadata::{Metadata, Resolution};

pub fn run(args: &EncodeVideoArgs) -> Result<bool> {
    let input_videos: Vec<PathBuf> = scan_videos_from_paths(&args.inputs, args.depth);

    if input_videos.is_empty() {
        bail!("no video found in all your inputs");
    }

    Ok(batch_encode(
        &input_videos,
        // &args.preset,
        &args.resolution,
        args.fps,
    ))
}

fn batch_encode(videos: &[PathBuf], resolution: &Resolution, fps: u8) -> bool {
    videos.iter().fold(false, |mut has_error, video| {
        if let Err(e) = process_encode(video, resolution, fps) {
            log::error!("{e}");
            has_error = true;
        }
        has_error
    })
}

fn process_encode(input: &Path, resolution: &Resolution, fps: u8) -> Result<()> {
    let config = Config::init(PathBuf::from(input), *resolution, fps);
    let metadata = Metadata::retrive(input)?;
    let encoder = Encoder::new(&config, &metadata)?;
    let stat = encoder.encode(ProgressMonitor::new(
        metadata.duration(),
        config.input().to_string_lossy().into_owned(),
    )?)?;

    let reduction = stat.1 as f64 / metadata.size() as f64;
    if reduction > 1.0 {
        log::info!(
            "{:?} output {} ({:.2}% of original)",
            input.file_name().unwrap_or(OsStr::new("unknown file")),
            // format_duration(stat.0),
            format_file_size(stat.1),
            reduction * 100.0
        );
    }

    Ok(())
}
