use crate::{find_videos_within_folder, is_video_path, resolve_to_absolute};
use std::{
    fs::symlink_metadata,
    io,
    path::{Path, PathBuf},
};

pub fn scan_video_from_path(path: impl AsRef<Path>, depth: u8) -> Result<Vec<PathBuf>, io::Error> {
    let abs_path = resolve_to_absolute(&path)?;

    let meta = symlink_metadata(&abs_path)?;

    match () {
        _ if meta.is_dir() => Ok(find_videos_within_folder(&abs_path, depth)),
        _ if meta.is_file() => {
            let videos = if is_video_path(&abs_path) {
                vec![abs_path]
            } else {
                vec![]
            };
            Ok(videos)
        }
        _ => Ok(vec![]),
    }
}
