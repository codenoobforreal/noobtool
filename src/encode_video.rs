use anyhow::{Result, bail};
use cli::EncodeVideoArgs;
use ffmpeg_progress_monitor::ProgressMonitor;
use std::path::{Path, PathBuf};
use utils::{format_duration, format_file_size, scan_video_from_path};
use video_encoder::{Config, Encoder, Preset};
use video_metadata::{Metadata, Resolution};

pub fn process_args(args: &EncodeVideoArgs) -> Result<bool> {
    let input_videos: Vec<PathBuf> = args
        .inputs
        .iter()
        .flat_map(|input| scan_video_from_path(input, args.depth).ok())
        .flatten()
        .collect();

    if input_videos.is_empty() {
        bail!("no video found in all your inputs");
    }

    // let input_videos_result: Result<Vec<PathBuf>, _> = args
    //     .inputs
    //     .iter()
    //     .map(|input| scan_video_from_path(input, args.depth)) // Iterator<Item = Result<Vec<PathBuf>, io::Error>>
    //     .collect::<Result<Vec<Vec<PathBuf>>, _>>() // 如果所有 scan 都成功，得到 Ok(Vec<Vec<PathBuf>>)
    //     .map(|nested_vec| nested_vec.into_iter().flatten().collect()); // 将 Ok 内的 Vec<Vec<PathBuf>> 扁平化为 Vec<PathBuf>

    Ok(batch_encode(
        &input_videos,
        &args.preset,
        &args.resolution,
        args.fps,
    ))
}

fn batch_encode(videos: &[PathBuf], preset: &Preset, resolution: &Resolution, fps: u8) -> bool {
    videos.iter().fold(false, |mut has_error, video| {
        if let Err(e) = process_encode(video, preset, resolution, fps) {
            log::error!("{e}");
            has_error = true;
        }
        has_error
    })
}

pub fn process_encode(
    input: &Path,
    preset: &Preset,
    resolution: &Resolution,
    fps: u8,
) -> Result<()> {
    let config = Config::init(PathBuf::from(input), *resolution, *preset, fps);
    let metadata = Metadata::retrive(input)?;
    let encoder = Encoder::new(&config, &metadata)?;
    let stat = encoder.encode(ProgressMonitor::new(metadata.duration())?)?;

    let reduction = stat.1 as f64 / metadata.size() as f64;

    if reduction > 1.0 {
        log::info!(
            "{:?} encoded in {} and shrunk(maybe) to {} ({:.2}% of original)",
            input.file_name().unwrap_or(input.as_os_str()),
            format_duration(stat.0),
            format_file_size(stat.1),
            reduction * 100.0
        );
    }

    Ok(())
}
