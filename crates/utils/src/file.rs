use std::{fs, io, path::Path};

// todo: remove?
fn is_file_creatable(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("file exists: {}", path.display()));
    }

    let parent = path.parent().ok_or("no parent path")?;

    match fs::metadata(parent) {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                Err(format!("no write permission: {}", parent.display()))
            } else {
                Ok(())
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Err(format!("parent path doesn't exist: {}", parent.display()))
        }
        Err(e) => Err(format!("{e}")),
    }
}
