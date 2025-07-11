use anyhow::{Context, Result};
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use std::fs;
use std::path::Path;

pub fn sign(package_path: &str, key_path: &str, comment: Option<&str>) -> Result<()> {
    println!("Signing package: {}", package_path);

    let package_file = Path::new(package_path);
    if !package_file.exists() {
        return Err(anyhow::anyhow!(
            "Package file '{}' does not exist",
            package_path
        ));
    }

    // Read the secret key from the provided key file
    let key_content =
        fs::read(key_path).context(format!("Failed to read secret key from {}", key_path))?;

    // Parse the key pair from the secret key seed.
    // The ring crate uses the raw 32-byte seed for Ed25519.
    if key_content.len() != 32 {
        return Err(anyhow::anyhow!(
            "Invalid secret key length: Ed25519 seeds must be 32 bytes."
        ));
    }
    let _rng = SystemRandom::new();
    let key_pair = Ed25519KeyPair::from_seed_unchecked(&key_content)
        .map_err(|e| anyhow::anyhow!("Failed to create key pair from seed: {:?}", e))?;

    // Read the package file content to sign
    let package_data =
        fs::read(package_file).context(format!("Failed to read package file {}", package_path))?;

    // Sign the package data
    let signature = key_pair.sign(&package_data);

    // Write the signature to a file with .sig extension
    let sig_path = format!("{}.sig", package_path);
    fs::write(&sig_path, signature.as_ref())
        .context(format!("Failed to write signature to {}", sig_path))?;

    // Optionally write the comment if provided
    if let Some(cmt) = comment {
        let comment_path = format!("{}.comment", sig_path);
        fs::write(&comment_path, cmt)
            .context(format!("Failed to write comment to {}", comment_path))?;
    }

    println!("Signature created at: {}", sig_path);
    Ok(())
}