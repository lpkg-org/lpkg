use crate::package::archive::{create_archive, create_tar_and_checksum};
use crate::package::metadata::{MetaFile, parse_metadata};
use anyhow::{Context, Result};
use std::fs;

pub fn pack() -> Result<()> {
    let directory = std::env::current_dir().context("Failed to get current working directory")?;
    let directory_str = directory
        .to_str()
        .context("Failed to convert current working directory to string")?;

    println!("Building package from current directory: {}", directory_str);

    let dir_path = &directory;
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Current directory '{}' does not exist or is not a directory",
            directory_str
        ));
    }

    let meta_path = dir_path.join("meta.toml");
    if !meta_path.exists() {
        return Err(anyhow::anyhow!(
            "meta.toml not found in '{}'. Please run 'lpkg init' first.",
            directory_str
        ));
    }

    let files_dir = dir_path.join("files");
    if !files_dir.exists() || !files_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "'files/' directory not found in package source. A package must contain a \"files/\" directory."
        ));
    }

    // Get the absolute path of the files directory
    let files_dir_abs = files_dir
        .canonicalize()
        .context("Failed to get absolute path for files directory")?;

    // Calculate checksum of the files directory
    let content_checksum = create_tar_and_checksum(&files_dir_abs)
        .context("Failed to calculate checksum for package content")?;

    // Read existing meta.toml content
    let meta_content = fs::read_to_string(&meta_path).context(format!(
        "Failed to read meta.toml from {}",
        meta_path.display()
    ))?;

    // Parse it to a mutable struct to update the checksum
    let mut meta_file: MetaFile =
        toml::from_str(&meta_content).context("Failed to parse meta.toml content for update")?;
    meta_file.package.content_checksum = Some(content_checksum);

    // Serialize back to TOML and write to file
    let updated_meta_content =
        toml::to_string_pretty(&meta_file).context("Failed to serialize metadata to TOML")?;
    fs::write(&meta_path, updated_meta_content).context("Failed to write updated meta.toml")?;

    // Re-read meta.toml to ensure the checksum is included in the metadata struct
    let meta_content_final = fs::read_to_string(&meta_path).context(format!(
        "Failed to read meta.toml from {}",
        meta_path.display()
    ))?;
    let meta_file_final = parse_metadata(&meta_content_final)
        .context("Failed to parse package metadata after checksum update")?;

    let output_filename = format!(
        "{}-{}.lpkg",
        meta_file_final.package.name, meta_file_final.package.version
    );
    let output_path = dir_path.join(&output_filename);

    // Create the .lpkg archive
    create_archive(
        &files_dir_abs,
        &meta_path,
        output_path.to_str().unwrap_or(&output_filename),
    )
    .context("Failed to create .lpkg archive")?;

    println!("Created package: {}", output_filename);
    Ok(())
}
