use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn copy_file(source: &str, destination: &str) -> Result<()> {
    let src_path = Path::new(source);
    let dest_path = Path::new(destination);

    // Ensure the destination directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).context(format!(
            "Failed to create parent directory for {}",
            destination
        ))?;
    }

    // Copy the file
    fs::copy(src_path, dest_path).context(format!(
        "Failed to copy file from {} to {}",
        source, destination
    ))?;

    // TODO: Preserve file permissions and metadata if necessary
    Ok(())
}

pub fn remove_file(path: &str) -> Result<()> {
    let file_path = Path::new(path);
    if file_path.exists() {
        fs::remove_file(file_path).context(format!("Failed to remove file at {}", path))?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn create_directory(path: &str) -> Result<()> {
    let dir_path = Path::new(path);
    fs::create_dir_all(dir_path).context(format!("Failed to create directory at {}", path))?;
    Ok(())
}
