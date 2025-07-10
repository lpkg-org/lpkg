use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use ureq;

// Define the structure for a package index entry in the repository
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageIndex {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub url: String,                   // URL to download the .lpkg file
    pub signature_url: Option<String>, // URL to download the signature file if available
    pub dependencies: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
}

// Define the structure for the repository index file
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub packages: HashMap<String, PackageIndex>,
}

// Function to fetch and parse a remote repository index
pub fn fetch_index(url: &str) -> Result<RepositoryIndex> {
    println!("Fetching repository index from {}", url);
    if url.starts_with("file://") {
        let path = url.trim_start_matches("file://");
        let content = fs::read_to_string(path)
            .context(format!("Failed to read local repository index from {}", path))?;
        let index: RepositoryIndex = serde_json::from_str(&content)
            .context("Failed to parse local repository index JSON")?;
        Ok(index)
    } else {
        let agent = ureq::agent();
        let response = agent.get(url).call().context("Failed to fetch repository index")?;
        let index: RepositoryIndex = response.into_json().context("Failed to parse repository index JSON")?;
        Ok(index)
    }
}

// Function to save a repository index locally for caching purposes
pub fn save_index(index: &RepositoryIndex, cache_path: &str) -> Result<()> {
    let index_json = serde_json::to_string_pretty(index)
        .context("Failed to serialize repository index to JSON")?;
    fs::write(cache_path, index_json).context(format!(
        "Failed to write repository index to {}",
        cache_path
    ))?;
    println!("Repository index cached at {}", cache_path);
    Ok(())
}

// Function to load a cached repository index
pub fn load_cached_index(cache_path: &str) -> Result<Option<RepositoryIndex>> {
    let cache_file = Path::new(cache_path);
    if !cache_file.exists() {
        return Ok(None);
    }

    let index_json = fs::read_to_string(cache_file)
        .context(format!("Failed to read cached index from {}", cache_path))?;
    let index: RepositoryIndex = serde_json::from_str(&index_json)
        .context("Failed to deserialize cached repository index")?;
    Ok(Some(index))
}

// Function to search for a package in the repository index
pub fn search_package<'a>(
    index: &'a RepositoryIndex,
    package_name: &str,
) -> Option<&'a PackageIndex> {
    index.packages.get(package_name)
}

// Function to download a package from the repository
pub fn download_package(package: &PackageIndex, destination: &str) -> Result<()> {
    println!(
        "Downloading package {} from {} to {}",
        package.name, package.url, destination
    );

    if package.url.starts_with("file://") {
        let src_path = package.url.trim_start_matches("file://");
        fs::copy(src_path, destination).context(format!("Failed to copy package from {} to {}", src_path, destination))?;
    } else {
        let agent = ureq::agent();
        let response = agent.get(&package.url).call().context("Failed to download package")?;
        let mut bytes = Vec::new();
        response.into_reader().read_to_end(&mut bytes)?;
        fs::write(destination, &bytes).context(format!("Failed to write package to {}", destination))?;
    }

    println!("Package {} downloaded to {}", package.name, destination);
    Ok(())
}



