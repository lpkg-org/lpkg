use anyhow::{Context, Result};
use rusqlite::Connection;

pub fn initialize_schema(conn: &mut Connection) -> Result<()> {
    // Create packages table for storing package metadata
    conn.execute(
        "CREATE TABLE IF NOT EXISTS packages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT,
            license TEXT,
            homepage TEXT,
            repository TEXT,
            authors TEXT,
            lpkg_path TEXT,
            installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(name, version)
        )",
        [],
    )
    .context("Failed to create packages table")?;

    // Create package_files table for tracking files installed by each package
    conn.execute(
        "CREATE TABLE IF NOT EXISTS package_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            checksum TEXT,
            FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE,
            UNIQUE(package_id, path)
        )",
        [],
    )
    .context("Failed to create package_files table")?;

    // Create dependencies table for tracking package dependencies
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dependencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id INTEGER NOT NULL,
            dependency_name TEXT NOT NULL,
            dependency_version TEXT,
            FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE
        )",
        [],
    )
    .context("Failed to create dependencies table")?;

    // Create conflicts table for tracking package conflicts
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conflicts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_id INTEGER NOT NULL,
            conflict_name TEXT NOT NULL,
            conflict_version TEXT,
            FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE
        )",
        [],
    )
    .context("Failed to create conflicts table")?;

    Ok(())
}
