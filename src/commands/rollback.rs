use anyhow::{Context, Result};
use crate::db::operations::{get_package_files_by_id, remove_package_by_id};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

pub fn rollback(conn: &mut Connection, package_id: i64) -> Result<()> {
    println!("Attempting to rollback package with ID: {}", package_id);

    let files_to_remove = get_package_files_by_id(conn, package_id)
        .context(format!("Failed to get files for package ID {}", package_id))?;

    if files_to_remove.is_empty() {
        println!("No files found for package ID {}. Already rolled back or never installed.", package_id);
        return Ok(());
    }

    for file_path in &files_to_remove {
        let path = Path::new(file_path);
        if path.exists() {
            if path.is_file() {
                fs::remove_file(path).context(format!("Failed to remove file: {}", file_path))?;
            } else if path.is_dir() {
                // Only remove empty directories, or handle recursively if needed
                // For now, we'll just try to remove files and let the user clean up empty dirs
                println!("Skipping directory: {} (manual cleanup might be required)", file_path);
            }
        }
        println!("Removed: {}", file_path);
    }

    remove_package_by_id(conn, package_id)
        .context(format!("Failed to remove package with ID {} from database", package_id))?;

    println!("Rollback successful for package ID {}.", package_id);

    Ok(())
}
