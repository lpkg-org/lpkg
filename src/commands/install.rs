use crate::package::{archive::extract_archive, metadata::parse_metadata};
use crate::utils::file_ops::copy_file;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn install(conn: &mut Connection, file: &str) -> Result<()> {
    println!("Installing package from: {}", file);

    let temp_base = env::var("TMPDIR").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/lpkg_temp", home)
    });
    fs::create_dir_all(&temp_base).context(format!(
        "Failed to create temporary base directory: {}",
        temp_base
    ))?;
    let temp_dir = format!("{}/lpkg_install_{}", temp_base, std::process::id());
    let temp_path = Path::new(&temp_dir);

    // Extract the .lpkg archive to a temporary directory
    extract_archive(file, &temp_dir).context("Failed to extract .lpkg archive")?;

    // Read and parse meta.toml from the extracted archive
    let meta_path = temp_path.join("meta.toml");
    let meta_content = fs::read_to_string(&meta_path).context(format!(
        "Failed to read meta.toml from {}",
        meta_path.display()
    ))?;
    let meta_file = parse_metadata(&meta_content).context("Failed to parse package metadata")?;
    let metadata = &meta_file.package;

    let install_base_path = PathBuf::from("/usr/local/lpkg/packages")
        .join(format!("{}-{}", metadata.name, metadata.version));

    println!(
        "DEBUG: Parsed meta_file.dependencies: {:?}",
        meta_file.dependencies
    );

    println!(
        "Installing package: {} version {}",
        metadata.name, metadata.version
    );

    // Check dependencies
    println!("DEBUG: Starting dependency check...");
    if let Some(deps) = &meta_file.dependencies {
        println!("DEBUG: Dependencies found: {:?}", deps);
        for (dep_name, dep_version_constraint) in deps {
            println!(
                "DEBUG: Checking dependency: {} {}",
                dep_name, dep_version_constraint
            );
            let is_installed = crate::db::operations::is_package_installed(
                conn,
                dep_name,
                Some(dep_version_constraint),
            )?;
            println!(
                "DEBUG: Is {} {} installed? {}",
                dep_name, dep_version_constraint, is_installed
            );
            if !is_installed {
                return Err(anyhow::anyhow!(
                    "Dependency not met: {} {}",
                    dep_name,
                    dep_version_constraint
                ));
            }
            println!(
                "DEBUG: Dependency met: {} {}",
                dep_name, dep_version_constraint
            );
        }
        println!("DEBUG: All dependencies met.");
    } else {
        println!("DEBUG: No dependencies found.");
    }

    // Check if package is already installed
    println!("DEBUG: Checking if package is already installed...");
    if crate::db::operations::is_package_installed(
        conn,
        &metadata.name,
        Some(&metadata.version),
    )? {
        println!("DEBUG: Package already installed.");
        return Err(anyhow::anyhow!(
            "Package '{}' version '{}' is already installed.",
            metadata.name,
            metadata.version
        ));
    }
    println!("DEBUG: Package not installed, proceeding with installation.");

    // Update package database first to get package_id for file recording
    let package_id = crate::db::operations::add_package(
        conn,
        &metadata.name,
        &metadata.version,
        metadata.description.as_deref(),
        metadata.license.as_deref(),
        metadata.homepage.as_deref(),
        metadata.repository.as_deref(),
        metadata.authors.as_ref().map(|a| a.join(", ")).as_deref(),
        Some(file),
    )
    .context("Failed to add package to database")?;

    // Add dependencies to database
    if let Some(deps) = &meta_file.dependencies {
        for (dep_name, dep_version_constraint) in deps {
            crate::db::operations::add_dependency(
                conn,
                package_id,
                dep_name,
                Some(dep_version_constraint),
            )
            .context(format!(
                "Failed to add dependency {} {} to database",
                dep_name,
                dep_version_constraint
            ))?;
        }
    }

    // Run pre-install script if specified
    let scripts_dir = temp_path.join("scripts");
    if scripts_dir.exists() && scripts_dir.is_dir() {
        if let Some(scripts) = &metadata.scripts {
            if let Some(pre_install) = &scripts.pre_install {
                let script_path = scripts_dir.join(pre_install);
                if script_path.exists() {
                    println!("Running pre-install script: {}", script_path.display());
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(script_path.to_str().unwrap_or_default())
                        .output()
                        .context(format!(
                            "Failed to execute pre-install script {}",
                            script_path.display()
                        ))?;
                    if !output.status.success() {
                        return Err(anyhow::anyhow!(
                            "Pre-install script failed with status: {}. Stderr: {}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    println!("Pre-install script completed successfully");
                }
            }
        }
    }

    // Copy files to system locations from temp_path.join("files")
    let files_dir = temp_path.join("files");
    if files_dir.exists() && files_dir.is_dir() {
        use walkdir::WalkDir;

        let mut installed_files = Vec::new();
        for entry in WalkDir::new(&files_dir).into_iter().filter_map(|e| e.ok()) {
            let src_path = entry.path();
            if src_path.is_file() {
                // Calculate the relative path from files_dir
                let rel_path = src_path.strip_prefix(&files_dir).context(format!(
                    "Failed to strip prefix from path {}",
                    src_path.display()
                ))?;

                // Destination is in the permanent installation directory
                let dest_path = install_base_path.join(rel_path);

                // Create parent directories if they don't exist
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).context(format!(
                        "Failed to create parent directory for {}",
                        dest_path.display()
                    ))?;
                }
                println!(
                    "Installing file: {} -> {}",
                    src_path.display(),
                    dest_path.display()
                );
                copy_file(
                    src_path.to_str().unwrap_or_default(),
                    dest_path.to_str().unwrap_or_default(),
                )
                .context(format!("Failed to install file {}", src_path.display()))?;

                installed_files.push(dest_path.to_string_lossy().to_string());
            }
        }
        // Record installed_files in database with checksums
        use crate::utils::checksum::calculate_sha256;
        for file in &installed_files {
            let src_path = files_dir.join(file);
            let checksum = calculate_sha256(src_path.to_str().unwrap_or_default()).context(
                format!("Failed to calculate checksum for {}", src_path.display()),
            )?;
            crate::db::operations::add_package_file(conn, package_id, file, Some(&checksum))
                .context(format!("Failed to record file {} in database", file))?;
        }
    } else {
        return Err(anyhow::anyhow!(
            "No 'files' directory found in extracted archive"
        ));
    }

    // Copy .desktop file and icon to standard locations for desktop integration
    let desktop_file_src = temp_path
        .join("files/usr/share/applications")
        .join(format!("{}.desktop", metadata.name));
    let desktop_file_dest_dir = PathBuf::from("/usr/local/share/applications");
    let desktop_file_dest = desktop_file_dest_dir.join(format!("{}.desktop", metadata.name));

    if desktop_file_src.exists() {
        fs::create_dir_all(&desktop_file_dest_dir).context(format!(
            "Failed to create directory for desktop file: {}",
            desktop_file_dest_dir.display()
        ))?;
        copy_file(
            desktop_file_src.to_str().unwrap_or_default(),
            desktop_file_dest.to_str().unwrap_or_default(),
        )
        .context(format!(
            "Failed to copy desktop file to {}",
            desktop_file_dest.display()
        ))?;
        crate::db::operations::add_package_file(
            conn,
            package_id,
            desktop_file_dest.to_str().unwrap_or_default(),
            None,
        )
        .context(format!(
            "Failed to record desktop file {} in database",
            desktop_file_dest.display()
        ))?;
        println!("Copied desktop file to: {}", desktop_file_dest.display());
    }

    let icon_file_src = temp_path
        .join("files/usr/share/icons/hicolor/scalable/apps")
        .join(format!("{}.png", metadata.name));
    let icon_file_dest_dir = PathBuf::from("/usr/local/share/icons/hicolor/scalable/apps");
    let icon_file_dest = icon_file_dest_dir.join(format!("{}.png", metadata.name));

    if icon_file_src.exists() {
        fs::create_dir_all(&icon_file_dest_dir).context(format!(
            "Failed to create directory for icon file: {}",
            icon_file_dest_dir.display()
        ))?;
        copy_file(
            icon_file_src.to_str().unwrap_or_default(),
            icon_file_dest.to_str().unwrap_or_default(),
        )
        .context(format!(
            "Failed to copy icon file to {}",
            icon_file_dest.display()
        ))?;
        crate::db::operations::add_package_file(
            conn,
            package_id,
            icon_file_dest.to_str().unwrap_or_default(),
            None,
        )
        .context(format!(
            "Failed to record icon file {} in database",
            icon_file_dest.display()
        ))?;
        println!("Copied icon file to: {}", icon_file_dest.display());
    }

    // Refresh icon cache
    println!("Refreshing icon cache...");
    let output = std::process::Command::new("sudo")
        .arg("gtk-update-icon-cache")
        .arg("-f")
        .output()
        .context("Failed to refresh icon cache")?;

    if !output.status.success() {
        eprintln!("Warning: Failed to refresh icon cache. Stderr: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("Icon cache refreshed successfully.");
    }

    // Run post-install script if specified after file installation
    let scripts_dir = temp_path.join("scripts");
    if scripts_dir.exists() && scripts_dir.is_dir() {
        if let Some(scripts) = &metadata.scripts {
            if let Some(post_install) = &scripts.post_install {
                let script_path = scripts_dir.join(post_install);
                if script_path.exists() {
                    println!("Running pre-install script: {}", script_path.display());
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(script_path.to_str().unwrap_or_default())
                        .output()
                        .context(format!(
                            "Failed to execute pre-install script {}",
                            script_path.display()
                        ))?;
                    if !output.status.success() {
                        return Err(anyhow::anyhow!(
                            "Pre-install script failed with status: {}. Stderr: {}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    println!("Pre-install script completed successfully");
                }
            }
        }
    }

    // Create wrapper script for the executable
    let executable_name = metadata.name.clone();
    let wrapper_script_name = format!("{}-wrapper.sh", executable_name);
    let wrapper_script_path = install_base_path.join(&wrapper_script_name);

    let wrapper_content = format!(
        "#!/bin/bash\n\n# Change to the application's installation directory\ncd \"{}\" || exit 1\n\n# Execute the Flutter application\nexec \"./{}\" \"$@\"",
        install_base_path.display(),
        executable_name
    );

    fs::write(&wrapper_script_path, wrapper_content).context(format!(
        "Failed to write wrapper script to {}",
        wrapper_script_path.display()
    ))?;
    fs::set_permissions(&wrapper_script_path, fs::Permissions::from_mode(0o755)).context(
        format!(
            "Failed to set permissions for wrapper script {}",
            wrapper_script_path.display()
        ),
    )?;
    println!("Created wrapper script: {}", wrapper_script_path.display());

    // Create symlink for the wrapper script in /usr/local/bin
    let symlink_path = PathBuf::from("/usr/local/bin").join(&executable_name);

    if symlink_path.exists() {
        if symlink_path.is_symlink() {
            fs::remove_file(&symlink_path).context(format!(
                "Failed to remove existing symlink at {}",
                symlink_path.display()
            ))?;
        } else if symlink_path.is_file() {
            // If it's a regular file, remove it
            fs::remove_file(&symlink_path).context(format!(
                "Failed to remove existing file at {}",
                symlink_path.display()
            ))?;
        } else if symlink_path.is_dir() {
            // If it's a directory, we should not remove it automatically.
            return Err(anyhow::anyhow!(
                "Cannot create symlink: {} is an existing directory.",
                symlink_path.display()
            ));
        }
    }
    std::os::unix::fs::symlink(&wrapper_script_path, &symlink_path).context(format!(
        "Failed to create symlink from {} to {}",
        wrapper_script_path.display(),
        symlink_path.display()
    ))?;
    println!(
        "Created symlink: {} -> {}",
        symlink_path.display(),
        wrapper_script_path.display()
    );

    // Clean up temporary directory
    fs::remove_dir_all(temp_path).context("Failed to clean up temporary directory")?;

    Ok(())
}
