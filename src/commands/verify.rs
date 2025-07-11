use anyhow::{Context, Result};
use ring::signature::{ED25519, UnparsedPublicKey};
use std::fs;
use std::path::Path;
use crate::package::archive::{extract_archive, create_tar_and_checksum};
use crate::package::metadata::parse_metadata;

#[allow(dead_code)]
pub fn verify_signature(package_path: &str, sig_path: &str, key_path: &str) -> Result<()> {
    println!("Verifying package signature for: {}", package_path);

    let package_file = Path::new(package_path);
    if !package_file.exists() {
        return Err(anyhow::anyhow!(
            "Package file '{}' does not exist",
            package_path
        ));
    }

    let sig_file = Path::new(sig_path);
    if !sig_file.exists() {
        return Err(anyhow::anyhow!(
            "Signature file '{}' does not exist",
            sig_path
        ));
    }

    // The ring crate uses the raw 32-byte public key for Ed25519 verification.
    let key_content =
        fs::read(key_path).context(format!("Failed to read public key from {}", key_path))?;

    if key_content.len() != 32 {
        return Err(anyhow::anyhow!(
            "Invalid public key length: Ed25519 public keys must be 32 bytes."
        ));
    }

    let public_key = UnparsedPublicKey::new(&ED25519, &key_content);

    // Read the signature file
    let signature =
        fs::read(sig_file).context(format!("Failed to read signature from {}", sig_path))?;

    // Read the package file content to verify
    let package_data =
        fs::read(package_file).context(format!("Failed to read package file {}", package_path))?;

    // Verify the signature
    public_key
        .verify(&package_data, &signature)
        .map_err(|e| anyhow::anyhow!("Signature verification failed: {:?}", e))?;

    println!("Signature verification successful for {}", package_path);
    Ok(())
}

pub fn verify_content_checksum(package_path: &str) -> Result<()> {
    println!("Verifying package content checksum for: {}", package_path);

    let package_file = Path::new(package_path);
    if !package_file.exists() {
        return Err(anyhow::anyhow!(
            "Package file '{}' does not exist",
            package_path
        ));
    }

    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    // Extract the .lpkg archive to the temporary directory
    extract_archive(package_path, temp_path.to_str().unwrap())
        .context("Failed to extract .lpkg archive for verification")?;

    // Read meta.toml from the extracted archive
    let meta_path = temp_path.join("meta.toml");
    let meta_content = fs::read_to_string(&meta_path).context(format!(
        "Failed to read meta.toml from {}",
        meta_path.display()
    ))?;
    let metadata = parse_metadata(&meta_content)
        .context("Failed to parse package metadata from extracted archive")?;

    let expected_checksum = metadata.package.content_checksum.context("Package does not contain a content checksum")?;

    // Calculate checksum of the files directory within the extracted archive
    let files_dir = temp_path.join("files");
    if !files_dir.exists() || !files_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "'files/' directory not found in extracted package. Cannot verify content checksum."
        ));
    }
    let calculated_checksum = create_tar_and_checksum(&files_dir)
        .context("Failed to calculate checksum for extracted package content")?;

    if expected_checksum == calculated_checksum {
        println!("Content checksum verification successful for {}", package_path);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Content checksum verification failed for {}. Expected: {}, Calculated: {}",
            package_path,
            expected_checksum,
            calculated_checksum
        ))
    }
}