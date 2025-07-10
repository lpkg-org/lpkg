use crate::db::operations::list_packages;
use anyhow::{Context, Result};
use rusqlite::Connection;

pub fn list(conn: &Connection) -> Result<()> {
    println!("Listing installed packages");

    let packages = list_packages(conn).context("Failed to list installed packages")?;

    if packages.is_empty() {
        println!("No packages installed.");
    } else {
        for pkg in packages {
            println!("{}", pkg);
        }
    }

    Ok(())
}
