use anyhow::{Result, bail};
use cli::GenerateVideoThumbnailArgs;
use std::path::PathBuf;
use utils::scan_videos_from_paths;
use video_metadata::Metadata;
use video_thumbnail::{Generator, Grid};

pub fn run(args: &GenerateVideoThumbnailArgs) -> Result<bool> {
    let input_videos: Vec<PathBuf> = scan_videos_from_paths(&args.inputs, args.depth);

    if input_videos.is_empty() {
        bail!("no video found in all your inputs");
    }

    Ok(generate_thumbnails(input_videos, args.grid, args.base))
}

fn generate_thumbnails(videos: Vec<PathBuf>, grid: Grid, base_dimesion: u16) -> bool {
    let mut errors = vec![];

    videos.iter().for_each(|video| {
        if let Err(e) = generate_thumbnail(video.to_path_buf(), grid, base_dimesion) {
            errors.push(e);
        }
    });

    errors.iter().for_each(|e| log::error!("{e}"));

    let success_count = videos.len() - errors.len();

    log::info!(
        "generated {} thumbnails,{} failed",
        success_count,
        errors.len()
    );

    !errors.is_empty()
}

pub fn generate_thumbnail(input: PathBuf, grid: Grid, base_dimesion: u16) -> Result<()> {
    let metadata = Metadata::retrive(&input)?;
    let generator = Generator::new(
        &input,
        metadata.duration(),
        grid,
        base_dimesion,
        metadata.ratio(),
    );

    Ok(generator.generate()?)
}
