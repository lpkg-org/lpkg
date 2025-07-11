use crate::commands::init_handlers::{flutter, rust};
use anyhow::{Context, Result};
use rusqlite::Connection;


pub fn init(conn: &mut Connection) -> Result<()> {
    let directory = std::env::current_dir().context("Failed to get current working directory")?;
    let directory_str = directory
        .to_str()
        .context("Failed to convert current working directory to string")?;

    println!(
        "Initializing project in current directory: {}",
        directory_str
    );

    let dir_path = &directory;
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Current directory '{}' does not exist or is not a directory",
            directory_str
        ));
    }

    let pubspec_yaml_path = dir_path.join("pubspec.yaml");
    let cargo_toml_path = dir_path.join("Cargo.toml");
    let meta_path = dir_path.join("meta.toml");

    if pubspec_yaml_path.exists() {
        flutter::prepare_flutter_project(conn, dir_path, &meta_path)?;
    } else if cargo_toml_path.exists() {
        rust::prepare_rust_project(conn, dir_path, &meta_path)?;
    } else {
        return Err(anyhow::anyhow!(
            "No recognized project type (Flutter or Rust) found in '{}'.",
            directory_str
        ));
    }

    println!(
        "Project initialized successfully in '{}'. You can now run 'lpkg pack'",
        directory_str
    );
    Ok(())
}





// Helper function to recursively copy a directory

