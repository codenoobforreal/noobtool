use crate::constants::VIDEO_EXTS_SET;
use path_clean::clean;
use std::{
    io,
    path::{Component, Path, PathBuf, Prefix},
};
use walkdir::WalkDir;

pub fn find_video_within_folder<P: AsRef<Path>>(path: P, max_depth: usize) -> Vec<PathBuf> {
    WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|r| {
            // note: r.ok() discard error
            r.ok().and_then(|dir| {
                if dir.file_type().is_file() && is_video_path(dir.path()) {
                    Some(dir.into_path())
                } else {
                    None
                }
            })
        })
        .collect()
}

#[cfg(windows)]
pub fn is_root_path<P: AsRef<Path>>(path: P) -> bool {
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
