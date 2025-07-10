use crate::db::operations::get_package_info;
use anyhow::{Context, Result};
use rusqlite::Connection;

pub fn info(conn: &Connection, package: &str) -> Result<()> {
    println!("Showing info for package: {}", package);

    let info = get_package_info(conn, package).context("Failed to get package information")?;

    if let Some(info_str) = info {
        println!("{}", info_str);
    } else {
        println!("Package '{}' not found.", package);
    }

    Ok(())
}
