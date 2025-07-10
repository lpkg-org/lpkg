use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

use crate::db::operations::{get_package_files_by_id, remove_package_by_id, get_package_data};

pub fn remove(conn: &mut Connection, package_name: &str) -> Result<()> {
    println!("Removing package: {}", package_name);

    let (package_id, _version, lpkg_path) = match get_package_data(conn, package_name)? {
        Some((id, version, path)) => (id, version, path),
        None => {
            println!("Package '{}' is not installed.", package_name);
            return Ok(());
        }
    };

    

    let files = get_package_files_by_id(conn, package_id)?;
    for file in &files {
        println!("Removing file: {}", file);
        crate::utils::file_ops::remove_file(file)?;
    }

    

    // Remove .desktop file for desktop integration
    let desktop_file_name = format!("{}.desktop", package_name);
    let desktop_file_path = Path::new("/usr/local/share/applications/").join(&desktop_file_name);
    if desktop_file_path.exists() {
        fs::remove_file(&desktop_file_path).context(format!(
            "Failed to remove .desktop file at {}",
            desktop_file_path.display()
        ))?;
        println!("Removed desktop entry: {}", desktop_file_path.display());
    }

    // Remove .desktop file for desktop integration
    let desktop_file_name = format!("{}.desktop", package_name);
    let desktop_file_path = Path::new("/usr/local/share/applications/").join(&desktop_file_name);
    if desktop_file_path.exists() {
        fs::remove_file(&desktop_file_path).context(format!(
            "Failed to remove .desktop file at {}",
            desktop_file_path.display()
        ))?;
        println!("Removed desktop entry: {}", desktop_file_path.display());
    }

    // Remove symlink from /usr/local/bin
    let symlink_path = PathBuf::from("/usr/local/bin/").join(package_name);
    println!("DEBUG: Checking for symlink at: {}", symlink_path.display());
    if symlink_path.exists() || symlink_path.is_symlink() { // Check both exists and is_symlink for robustness
        println!("DEBUG: Found existing entry at {}. Attempting to remove.", symlink_path.display());
        if symlink_path.is_symlink() {
            match fs::remove_file(&symlink_path) {
                Ok(_) => println!("Removed symlink: {}", symlink_path.display()),
                Err(e) => eprintln!("ERROR: Failed to remove symlink {}: {}", symlink_path.display(), e),
            }
        } else if symlink_path.is_file() {
            // If it's a regular file, remove it
            match fs::remove_file(&symlink_path) {
                Ok(_) => println!("Removed file: {}", symlink_path.display()),
                Err(e) => eprintln!("ERROR: Failed to remove file {}: {}", symlink_path.display(), e),
            }
        } else if symlink_path.is_dir() {
            // If it's a directory, we should not remove it automatically.
            eprintln!("ERROR: Cannot remove {}: it is an existing directory. Manual intervention may be required.", symlink_path.display());
        }
    } else {
        println!("DEBUG: No existing entry found at: {}", symlink_path.display());
    }

    let removed = remove_package_by_id(conn, package_id)?;

    // Clean up ld.so.conf.d entry
    let ld_conf_file = Path::new("/etc/ld.so.conf.d/").join(format!("lpkg-{}.conf", package_name));
    if ld_conf_file.exists() {
        fs::remove_file(&ld_conf_file).context(format!(
            "Failed to remove ld.so.conf file at {}",
            ld_conf_file.display()
        ))?;
        println!("Removed dynamic linker config: {}", ld_conf_file.display());

        // Run ldconfig to update the dynamic linker cache
        let output = std::process::Command::new("sudo")
            .arg("ldconfig")
            .output()
            .context("Failed to execute ldconfig")?;

        if !output.status.success() {
            eprintln!("ldconfig failed with status: {}. Stderr: {}",
                       output.status,
                       String::from_utf8_lossy(&output.stderr));
        } else {
            println!("ldconfig executed successfully.");
        }
    }

    if removed {
        println!("Package '{}' successfully removed.", package_name);
    } else {
        println!("Package '{}' not found in database.", package_name);
    }

    Ok(())
}