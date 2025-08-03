use crate::{find_video_within_folder, is_root_path, is_video_path};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

// todo: remove?
pub fn is_file_creatable(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("文件已存在: {}", path.display()));
    }

    let parent = path.parent().ok_or("无效路径（无父目录）")?;

    match fs::metadata(parent) {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                Err(format!("父目录不可写: {}", parent.display()))
            } else {
                Ok(())
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Err(format!("父目录不存在: {}", parent.display()))
        }
        Err(e) => Err(format!("无法访问父目录: {e}")),
    }
}

pub fn collect_videos_from_userinput(max_depth: usize) -> Vec<PathBuf> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return vec![];
    }

    let mut video_files = Vec::with_capacity(args.len());

    for arg in args {
        let path = Path::new(&arg);
        if !is_root_path(path) {
            match fs::symlink_metadata(path).ok() {
                Some(meta) if meta.is_dir() => {
                    video_files.extend(find_video_within_folder(path, max_depth));
                }
                Some(meta) if meta.is_file() && is_video_path(path) => {
                    video_files.push(PathBuf::from(arg));
                }
                _ => {}
            }
        }
    }

    video_files.shrink_to_fit();

    video_files
}
