use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use std::fmt::Write;
use semver::{Version, VersionReq};

pub fn add_package(
    conn: &mut Connection,
    package_name: &str,
    version: &str,
    description: Option<&str>,
    license: Option<&str>,
    homepage: Option<&str>,
    repository: Option<&str>,
    authors: Option<&str>,
    lpkg_path: Option<&str>,
) -> Result<i64> {
    let mut stmt = conn
        .prepare(
            "INSERT INTO packages (name, version, description, license, homepage, repository, authors, lpkg_path)\n         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)\n         RETURNING id",
        )
        .context("Failed to prepare insert statement for packages")?;

    let package_id: i64 = stmt
        .query_row(
            params![
                package_name,
                version,
                description,
                license,
                homepage,
                repository,
                authors,
                lpkg_path
            ],
            |row| row.get(0),
        )
        .context("Failed to insert package into database")?;

    Ok(package_id)
}

pub fn add_package_file(
    conn: &mut Connection,
    package_id: i64,
    path: &str,
    checksum: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO package_files (package_id, path, checksum) VALUES (?1, ?2, ?3)",
        params![package_id, path, checksum],
    )
    .context("Failed to insert package file into database")?;
    Ok(())
}

pub fn add_dependency(
    conn: &mut Connection,
    package_id: i64,
    dependency_name: &str,
    dependency_version: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO dependencies (package_id, dependency_name, dependency_version) VALUES (?1, ?2, ?3)",
        params![package_id, dependency_name, dependency_version]
    ).context("Failed to insert dependency into database")?;
    Ok(())
}

#[allow(dead_code)]
pub fn add_conflict(
    conn: &mut Connection,
    package_id: i64,
    conflict_name: &str,
    conflict_version: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO conflicts (package_id, conflict_name, conflict_version) VALUES (?1, ?2, ?3)",
        params![package_id, conflict_name, conflict_version],
    )
    .context("Failed to insert conflict into database")?;
    Ok(())
}

#[allow(dead_code)]
pub fn remove_package(conn: &mut Connection, package_name: &str) -> Result<bool> {
    let result = conn
        .execute(
            "DELETE FROM packages WHERE name = ?1",
            params![package_name],
        )
        .context("Failed to delete package from database")?;

    Ok(result > 0)
}

pub fn list_packages(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT name, version FROM packages ORDER BY name ASC")
        .context("Failed to prepare select statement for listing packages")?;

    let packages = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let version: String = row.get(1)?;
            Ok(format!("{} {}", name, version))
        })
        .context("Failed to query packages from database")?;

    let result: Vec<String> = packages.collect::<Result<Vec<String>, _>>()?;
    Ok(result)
}

pub fn get_package_info(conn: &Connection, package_name: &str) -> Result<Option<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT name, version, description, license, homepage, repository, authors, installed_at\n         FROM packages WHERE name = ?1",
        )
        .context("Failed to prepare select statement for package info")?;

    let mut rows = stmt
        .query(params![package_name])
        .context("Failed to query package info from database")?;

    if let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let version: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let license: Option<String> = row.get(3)?;
        let homepage: Option<String> = row.get(4)?;
        let repository: Option<String> = row.get(5)?;
        let authors: Option<String> = row.get(6)?;
        let installed_at: String = row.get(7)?;

        let mut info = String::new();
        writeln!(info, "Name: {}", name)?;
        writeln!(info, "Version: {}", version)?;
        writeln!(info, "Installed At: {}", installed_at)?;
        if let Some(desc) = description {
            writeln!(info, "Description: {}", desc)?;
        }
        if let Some(lic) = license {
            writeln!(info, "License: {}", lic)?;
        }
        if let Some(home) = homepage {
            writeln!(info, "Homepage: {}", home)?;
        }
        if let Some(repo) = repository {
            writeln!(info, "Repository: {}", repo)?;
        }
        if let Some(auth) = authors {
            writeln!(info, "Authors: {}", auth)?;
        }

        

        // TODO: Fetch and display dependencies and conflicts
        Ok(Some(info))
    } else {
        Ok(None)
    }
}


#[allow(dead_code)]
pub fn get_package_files(conn: &Connection, package_name: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT pf.path FROM package_files pf\n         JOIN packages p ON pf.package_id = p.id\n         WHERE p.name = ?1\n         ORDER BY pf.path ASC",
        )
        .context("Failed to prepare select statement for package files")?;

    let files = stmt
        .query_map(params![package_name], |row| {
            let path: String = row.get(0)?;
            Ok(path)
        })
        .context("Failed to query package files from database")?;

    let result: Vec<String> = files.collect::<Result<Vec<String>, _>>()?;
    Ok(result)
}

pub fn is_package_installed(
    conn: &Connection,
    package_name: &str,
    version_constraint: Option<&str>,
) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT version FROM packages WHERE name = ?1")?;
    let installed_version_str: Option<String> = stmt.query_row(params![package_name], |row| row.get(0)).optional()?;

    if let Some(installed_version_str) = installed_version_str {
        let installed_version = Version::parse(&installed_version_str)
            .context(format!("Failed to parse installed version: {}", installed_version_str))?;

        if let Some(constraint_str) = version_constraint {
            let version_req = VersionReq::parse(constraint_str)
                .context(format!("Failed to parse version constraint: {}", constraint_str))?;

            Ok(version_req.matches(&installed_version))
        } else {
            // If no version constraint is provided, and the package is installed, return true
            Ok(true)
        }
    } else {
        // Package not installed
        Ok(false)
    }
}

pub fn get_package_data(conn: &Connection, package_name: &str) -> Result<Option<(i64, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, version, lpkg_path FROM packages WHERE name = ?1")?;
    let result = stmt.query_row(rusqlite::params![package_name], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }).optional()?;
    Ok(result)
}

pub fn get_package_files_by_id(conn: &Connection, package_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM package_files WHERE package_id = ?1")?;
    let files = stmt.query_map(params![package_id], |row| row.get(0))?;
    files.collect::<Result<Vec<String>, _>>().context("Failed to collect package files by ID")
}

pub fn remove_package_by_id(conn: &mut Connection, package_id: i64) -> Result<bool> {
    let result = conn.execute("DELETE FROM packages WHERE id = ?1", params![package_id])
        .context("Failed to delete package by ID from database")?;
    Ok(result > 0)
}
