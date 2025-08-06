use crate::constants::VIDEO_EXTS_SET;
use path_clean::clean;
use std::{
    io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn find_videos_within_folder<P: AsRef<Path>>(
    path: P,
    max_depth: usize,
) -> (Vec<PathBuf>, Vec<walkdir::Error>) {
    let mut videos = Vec::new();
    let mut errors = Vec::new();

    for entry in WalkDir::new(path).max_depth(max_depth).into_iter() {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() && is_video_path(entry.path()) {
                    videos.push(entry.into_path());
                }
            }
            Err(e) => errors.push(e),
        }
    }

    (videos, errors)
}

pub fn handle_walkdir_error(error: walkdir::Error) -> (String, &'static str) {
    let path = error
        .path()
        .unwrap_or(Path::new(" "))
        .to_string_lossy()
        .into_owned();

    let cause = if let Some(inner) = error.io_error() {
        match inner.kind() {
            io::ErrorKind::InvalidData => "contains invalid data",
            io::ErrorKind::PermissionDenied => "no permission",
            _ => "unexpected error",
        }
    } else {
        "unexpected error"
    };

    (path, cause)
}

#[allow(unused_variables)]
pub fn is_root_path<P: AsRef<Path>>(path: P) -> bool {
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        let path = path.as_ref();
        let mut components = path.components();

        let is_drive = matches!(
            components.next(),
            Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_))
        );

        if !is_drive {
            return false;
        }

        match components.next() {
            None => true,                                            // C:
            Some(Component::RootDir) => components.next().is_none(), // C:\ or C:\\
            _ => false,
        }
    }
    #[cfg(not(windows))]
    {
        todo!()
    }
}

pub fn is_video_path<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTS_SET.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn resolve_to_absolute<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let cleaned = clean(path);

    if Path::new(&cleaned).is_absolute() {
        return Ok(cleaned);
    }

    let cwd = std::env::current_dir()?;
    let abs_path = cwd.join(cleaned);
    Ok(abs_path)
}
