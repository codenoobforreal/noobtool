use crate::constants::VIDEO_EXTS;
use path_absolutize::Absolutize;
use std::{
    io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn find_videos_within_folder(path: impl AsRef<Path>, max_depth: u8) -> Vec<PathBuf> {
    WalkDir::new(path)
        .follow_root_links(false)
        .max_depth((max_depth).into())
        .into_iter()
        .filter_map(|e| e.ok())
        .fold(Vec::new(), |mut videos, entry| {
            if entry.file_type().is_file() && is_video_path(entry.path()) {
                videos.push(entry.into_path());
            }
            videos
        })
}

#[allow(unused_variables)]
pub fn is_root_path(path: impl AsRef<Path>) -> bool {
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

pub fn is_video_path(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn resolve_to_absolute(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path_ref = path.as_ref();
    let abs_path = path_ref.absolutize()?.to_path_buf();
    Ok(abs_path)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn handle_dot_path() -> io::Result<()> {
        let absolute_path = resolve_to_absolute(Path::new("."))?;
        assert!(
            !absolute_path.to_string_lossy().contains('.'),
            "{}",
            absolute_path.display()
        );
        Ok(())
    }
}
