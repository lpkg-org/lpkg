use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::Path;

pub fn get_connection(read_only: bool) -> Result<Connection> {
    let db_path = "/var/lib/lpkg/db.sqlite";
    let db_dir = Path::new("/var/lib/lpkg");

    // Ensure the directory exists (requires sudo for system-wide location)
    if !db_dir.exists() {
        fs::create_dir_all(db_dir).context(format!(
            "Failed to create database directory: {}",
            db_dir.display()
        ))?;
    }

    let flags = if read_only {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    };

    let conn = Connection::open_with_flags(db_path, flags)
        .context(format!("Failed to open database at: {}", db_path))?;
    Ok(conn)
}
