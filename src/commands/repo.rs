use anyhow::{Context, Result};
use rusqlite::Connection;
use crate::repository::{fetch_index, save_index, load_cached_index, search_package, download_package};
use std::path::PathBuf;

pub fn add_repo(_conn: &mut Connection, url: &str, name: &str) -> Result<()> {
    println!("Adding repository: {} with name {}", url, name);

    // In a real scenario, you might want to store repository info in the DB
    // For now, we just fetch and cache the index.

    let index = fetch_index(url).context(format!("Failed to fetch index from {}", url))?;

    let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let cache_path = cache_dir.join(format!("lpkg_repo_{}.json", name));
    save_index(&index, cache_path.to_str().unwrap_or_default())
        .context(format!("Failed to cache repository index for {}", name))?;

    println!("Repository '{}' added successfully.", name);
    Ok(())
}

pub fn search_repo(_conn: &Connection, package_name: &str, repo_name: Option<&str>) -> Result<()> {
    println!(
        "Searching for package: {} in repository: {:?}",
        package_name, repo_name
    );

    let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let repo_name = repo_name.unwrap_or("default"); // Assumes a default repo if not specified
    let cache_path = cache_dir.join(format!("lpkg_repo_{}.json", repo_name));

    if let Some(index) = load_cached_index(cache_path.to_str().unwrap_or_default())? {
        if let Some(package) = search_package(&index, package_name) {
            println!("Found package: {} ({}) - {}", package.name, package.version, package.description.as_deref().unwrap_or_default());
        } else {
            println!("Package '{}' not found in repository '{}'.", package_name, repo_name);
        }
    } else {
        println!("Repository '{}' not found. Please add it first with 'lpkg repo add'.", repo_name);
    }

    Ok(())
}

pub fn install_from_repo(conn: &mut Connection, package_name: &str, repo_name: Option<&str>) -> Result<()> {
    println!(
        "Installing package: {} from repository: {:?}",
        package_name, repo_name
    );

    let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let repo_name = repo_name.unwrap_or("default"); // Assumes a default repo if not specified
    let cache_path = cache_dir.join(format!("lpkg_repo_{}.json", repo_name));

    if let Some(index) = load_cached_index(cache_path.to_str().unwrap_or_default())? {
        if let Some(package) = search_package(&index, package_name) {
            let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
            let destination = download_dir.join(format!("{}-{}.lpkg", package.name, package.version));
            download_package(package, destination.to_str().unwrap_or_default())?;
            // After downloading, you would typically call the local install command
            crate::commands::install::install(conn, destination.to_str().unwrap_or_default())?;
            println!("Package installed from repository.");
        } else {
            println!("Package '{}' not found in repository '{}'.", package_name, repo_name);
        }
    } else {
        println!("Repository '{}' not found. Please add it first with 'lpkg repo add'.", repo_name);
    }

    Ok(())
}
