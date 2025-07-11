use anyhow::{Context, Result};
use regex;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::{Document, value};
use yaml_rust::YamlLoader;
use crate::utils::file_ops::copy_dir_all;

pub fn prepare_flutter_project(_conn: &mut Connection, project_root: &Path, meta_path: &Path) -> Result<()> {
    println!("Detected Flutter project. Preparing for lpkg packaging...");

    // 1. Read pubspec.yaml
    let pubspec_path = project_root.join("pubspec.yaml");
    let pubspec_content =
        fs::read_to_string(&pubspec_path).context("Failed to read pubspec.yaml")?;
    let docs =
        YamlLoader::load_from_str(&pubspec_content).context("Failed to parse pubspec.yaml")?;
    let doc = &docs[0];

    let package_name =
        doc["name"].as_str().context("pubspec.yaml missing package name")?;
    let package_version =
        doc["version"].as_str().context("pubspec.yaml missing package version")?;
    let package_description =
        doc["description"].as_str().unwrap_or("A Flutter application packaged with lpkg");

    // 2. Build Flutter project for Linux
    println!("Building Flutter project for Linux...");
    let output = Command::new("flutter")
        .arg("build")
        .arg("linux")
        .current_dir(project_root)
        .output()
        .context("Failed to run 'flutter build linux'")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Flutter build failed: {}. Stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    println!("Flutter build completed successfully.");

    // 3. Create files/ directory structure and copy build output
    let files_dir = project_root.join("files");
    // Ensure a clean slate for the files directory
    if files_dir.exists() {
        fs::remove_dir_all(&files_dir).context("Failed to clean up existing files directory")?;
    }
    fs::create_dir_all(&files_dir).context("Failed to create files directory")?;

    let usr_share_applications_dir = files_dir.join("usr/share/applications");
    let usr_share_icons_dir = files_dir.join("usr/share/icons/hicolor/128x128/apps");

    fs::create_dir_all(&usr_share_applications_dir)
        .context("Failed to create applications directory")?;
    fs::create_dir_all(&usr_share_icons_dir).context("Failed to create icons directory")?;

    let flutter_build_bundle_path = project_root.join("build/linux/x64/release/bundle");

    // Copy the entire bundle content directly into files/
    copy_dir_all(&flutter_build_bundle_path, &files_dir).context(format!(
        "Failed to copy Flutter build bundle from {} to {}",
        flutter_build_bundle_path.display(),
        files_dir.display()
    ))?;

    // 4. Generate meta.toml
    let mut meta_doc = Document::new();
    meta_doc["package"]["name"] = value(package_name);
    meta_doc["package"]["version"] = value(package_version);
    meta_doc["package"]["description"] = value(package_description);
    meta_doc["package"]["license"] = value("MIT"); // Default license for auto-generated

    // Extract APPLICATION_ID from CMakeLists.txt
    let cmake_lists_path = project_root.join("linux/CMakeLists.txt");
    let cmake_lists_content =
        fs::read_to_string(&cmake_lists_path).context("Failed to read linux/CMakeLists.txt")?;
    let re = regex::Regex::new(r#"set\(APPLICATION_ID\s+"([^"]+)"\)"#).unwrap();
    let application_id = re
        .captures(&cmake_lists_content)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| format!("org.lpkg.{}", package_name.replace("-", "_")));

    meta_doc["package"]["application_id"] = value(application_id.clone());

    // Handle icon
    let icon_path = project_root.join("assets/logo.png");
    if icon_path.exists() {
        fs::copy(
            &icon_path,
            usr_share_icons_dir.join(format!("{}.png", package_name)),
        )
        .context(format!(
            "Failed to copy icon from {} to {}",
            icon_path.display(),
            usr_share_icons_dir
                .join(format!("{}.png", package_name))
                .display()
        ))?;
    }

    fs::write(meta_path, meta_doc.to_string())
        .context("Failed to write auto-generated meta.toml for Flutter project")?;
    println!("Auto-generated meta.toml for Flutter project.");

    // Create .desktop file
    let desktop_file_name = format!("{}.desktop", package_name);
    let desktop_file_path = usr_share_applications_dir.join(&desktop_file_name);
    let desktop_content = format!(
        "[Desktop Entry]\nVersion=1.0\nType=Application\nName={}\nComment={}\nExec={}\nIcon={}\nTerminal=false\nCategories=Utility;\nStartupWMClass={}\n",
        package_name, package_description, package_name, package_name, application_id
    );

    fs::write(&desktop_file_path, desktop_content).context(format!(
        "Failed to create .desktop file at {}",
        desktop_file_path.display()
    ))?;
    println!("Created desktop entry: {}", desktop_file_path.display());

    Ok(())
}