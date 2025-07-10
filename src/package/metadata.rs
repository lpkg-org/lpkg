use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::from_str;

#[derive(Deserialize, Serialize, Debug)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub authors: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub content_checksum: Option<String>,
    pub scripts: Option<Scripts>,
    pub application_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Scripts {
    pub pre_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_install: Option<String>,
    pub post_remove: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MetaFile {
    pub package: PackageMetadata,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
}

pub fn parse_metadata(data: &str) -> Result<MetaFile> {
    let meta_file: MetaFile = from_str(data).context("Failed to parse meta.toml content")?;
    if meta_file.package.name.is_empty() {
        return Err(anyhow::anyhow!("Package name cannot be empty"));
    }
    if meta_file.package.version.is_empty() {
        return Err(anyhow::anyhow!("Package version cannot be empty"));
    }
    Ok(meta_file)
}
