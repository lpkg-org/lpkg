use anyhow::{Context, Result};
use rusqlite::Connection;
use crate::db::operations::{get_package_data, remove_package_by_id};
use crate::commands::install::install;
use crate::repository::{search_package, fetch_index, download_package};
use std::fs;
use std::path::PathBuf;

pub fn update(conn: &mut Connection, package_name: &str) -> Result<()> {
    println!("Attempting to update package: {}", package_name);

    // 1. Check if the package is installed and get its current version and ID
    let (package_id, current_version, lpkg_path) = match get_package_data(conn, package_name)? {
        Some((id, version, path)) => (id, version, path),
        None => {
            println!("Package '{}' is not installed. Cannot update.", package_name);
            return Ok(());
        }
    };

    let _lpkg_path = lpkg_path;

    println!("Currently installed version of '{}': {}", package_name, current_version);

    // 2. Search for a newer version in repositories
    // For simplicity, we'll assume a 'default' repository for now.
    // In a real scenario, you'd iterate through configured repositories.
    let _cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let repo_name = "default"; // Assuming a default repo
    let _cache_path = _cache_dir.join(format!("lpkg_repo_{}.json", repo_name));

    let index = fetch_index("file:///home/raja/Desktop/linuxpackage/lpkg/repo/index.json")?;
    let latest_package_meta = search_package(&index, package_name);

    let latest_package = match latest_package_meta {
        Some(p) => p,
        None => {
            println!("No newer version of '{}' found in repositories.", package_name);
            return Ok(());
        }
    };

    if latest_package.version == current_version {
        println!("Package '{}' is already at the latest version ({}).", package_name, current_version);
        return Ok(());
    }

    println!("Newer version available: {}", latest_package.version);

    // 3. Download the new version
    let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let new_lpkg_path = download_dir.join(format!("{}-{}.lpkg", latest_package.name, latest_package.version));

    crate::repository::download_package(&latest_package, new_lpkg_path.to_str().unwrap())
        .context("Failed to download new package version")?;

    println!("Downloaded new version to: {}", new_lpkg_path.display());

    // 4. Perform atomic update: Install new, remove old on success, rollback on failure
    // This is a simplified atomic update. A more robust solution would involve transactions
    // and potentially a temporary installation directory for the new version.
    let install_result = install(conn, new_lpkg_path.to_str().unwrap());

    match install_result {
        Ok(_) => {
            println!("Successfully installed new version of '{}'. Removing old version.", package_name);
            // Remove old package from DB and file system
            remove_package_by_id(conn, package_id)
                .context(format!("Failed to remove old package (ID: {}) from database", package_id))?;
            // Note: Actual file removal for the old version would happen here, 
            // but it's complex due to shared files and would require a more sophisticated
            // file tracking system. For now, we rely on the 'remove' command's logic.
            println!("Update of '{}' to version {} completed successfully.", package_name, latest_package.version);
        },
        Err(e) => {
            eprintln!("Failed to install new version of '{}': {:?}. Attempting rollback.", package_name, e);
            // Rollback the new installation (if any files were written before failure)
            // This rollback is tricky as 'install' might have partially written files.
            // A more robust rollback would involve snapshotting the system or using transactions.
            // For now, we'll just report the failure and leave the old version installed.
            println!("Old version of '{}' (ID: {}) remains installed.", package_name, package_id);
            return Err(anyhow::anyhow!("Update failed, old version retained."));
        }
    }

    // Clean up downloaded .lpkg file
    fs::remove_file(&new_lpkg_path)
        .context(format!("Failed to remove downloaded file: {}", new_lpkg_path.display()))?;

    Ok(())
}
