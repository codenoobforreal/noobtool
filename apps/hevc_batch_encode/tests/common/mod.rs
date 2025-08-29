use std::{
    fs::{read_dir, remove_file},
    io,
    path::{Path, PathBuf},
};

pub const TEST_VIDEOS: [&str; 1] = ["bear-1280x720.mp4"];

pub struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = delete_except_files(fixtures_path(), &TEST_VIDEOS);
    }
}

fn delete_except_files(folder_path: impl AsRef<Path>, keep_files: &[&str]) -> io::Result<()> {
    let dir = folder_path.as_ref();

    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !keep_files.contains(&file_name) {
                remove_file(&path)?;
            }
        }
    }
    Ok(())
}

pub fn fixtures_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}
