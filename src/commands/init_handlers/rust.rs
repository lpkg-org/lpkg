use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::{Document, value};
use crate::utils::file_ops::copy_dir_all;

pub fn prepare_rust_project(conn: &mut Connection, project_root: &Path, meta_path: &Path) -> Result<()> {
    println!("Detected Rust project. Preparing for lpkg packaging...");

    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_toml_content =
        fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;
    let cargo_toml_doc = cargo_toml_content
        .parse::<Document>()
        .context("Failed to parse Cargo.toml")?;

    let package_name = cargo_toml_doc["package"]["name"]
        .as_str()
        .context("Cargo.toml missing package name")?;
    let package_version = cargo_toml_doc["package"]["version"]
        .as_str()
        .context("Cargo.toml missing package version")?;
    let package_description = cargo_toml_doc["package"]["description"]
        .as_str()
        .unwrap_or("A Rust application packaged with lpkg");
    let package_license = cargo_toml_doc["package"]["license"]
        .as_str()
        .unwrap_or("MIT");

    // Build Rust project
    println!("Building Rust project...");
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(project_root)
        .output()
        .context("Failed to run 'cargo build --release'")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Rust build failed: {}. Stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    println!("Rust build completed successfully.");

    // Create files/ directory structure and copy build output
    let files_dir = project_root.join("files");
    fs::create_dir_all(&files_dir).context("Failed to create files directory")?;

    let usr_bin_dir = files_dir.join("usr/bin");
    fs::create_dir_all(&usr_bin_dir).context("Failed to create usr/bin directory")?;

    let executable_path = project_root.join("target/release").join(package_name);
    fs::copy(&executable_path, usr_bin_dir.join(package_name)).context(format!(
        "Failed to copy executable from {} to {}",
        executable_path.display(),
        usr_bin_dir.display()
    ))?;

    // Generate meta.toml
    let mut meta_doc = Document::new();
    meta_doc["package"]["name"] = value(package_name);
    meta_doc["package"]["version"] = value(package_version);
    meta_doc["package"]["description"] = value(package_description);
    meta_doc["package"]["license"] = value(package_license);

    fs::write(meta_path, meta_doc.to_string())
        .context("Failed to write auto-generated meta.toml for Rust project")?;
    println!("Auto-generated meta.toml for Rust project.");

    Ok(())
}