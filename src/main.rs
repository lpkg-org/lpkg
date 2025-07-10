use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod db;
mod package;
mod repository;
mod utils;

#[derive(Parser, Debug)]
#[command(author = "LinuxPackage Team <contact@linuxpackage.org>", version = "0.1.0", about = "A universal, native package manager for Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a project for lpkg packaging
    Init,
    /// Initialize the lpkg database schema
    Setup,
    /// Install a local .lpkg package file
    Install {
        /// Path to the .lpkg file
        file: String,
    },
    /// Remove an installed package
    Remove {
        /// Name of the package to remove
        package: String,
    },
    /// List all installed packages
    List,
    /// Show detailed information about an installed package
    Info {
        /// Name of the package to query
        package: String,
    },
    /// Build a .lpkg file from a prepared directory
    Pack,
    /// Sign a package with a private key
    Sign {
        /// Path to the .lpkg file
        package: String,
        /// Path to the private key file
        key: String,
        /// Optional comment for the signature
        #[arg(short, long)]
        comment: Option<String>,
    },
    /// Verify a package's content checksum
    Verify {
        /// Path to the .lpkg file
        package: String,
    },
    /// Rollback a package installation
    Rollback {
        /// ID of the package to rollback
        package_id: i64,
    },
    /// Update an installed package to its latest version
    Update {
        /// Name of the package to update
        package: String,
    },
    /// Repository management commands
    #[command(subcommand)]
    Repo(RepoCommands),
}

#[derive(Subcommand, Debug)]
enum RepoCommands {
    /// Add a new repository
    Add {
        /// URL of the repository index
        url: String,
        /// Name to identify the repository
        name: String,
    },
    /// Search for a package in a repository
    Search {
        /// Name of the package to search for
        package: String,
        /// Optional repository name to search in
        #[arg(short, long)]
        repo: Option<String>,
    },
    /// Install a package from a repository
    Install {
        /// Name of the package to install
        package: String,
        /// Optional repository name to install from
        #[arg(short, long)]
        repo: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let is_read_only_command = match &cli.command {
        Commands::List
        | Commands::Info { .. }
        | Commands::Verify { .. }
        | Commands::Pack
        | Commands::Sign { .. } => true,
        Commands::Repo(repo_cmd) => match repo_cmd {
            RepoCommands::Search { .. } => true,
            _ => false,
        },
        _ => false,
    };

    let mut conn = db::connection::get_connection(is_read_only_command)?;

    if !is_read_only_command {
        // Only initialize schema for commands that might write to the DB
        db::schema::initialize_schema(&mut conn)?;
    }

    let result = match &cli.command {
        Commands::Init => commands::init::init(&mut conn),
        Commands::Setup => commands::setup::setup(),
        Commands::Install { file } => commands::install::install(&mut conn, file),
        Commands::Remove { package } => commands::remove::remove(&mut conn, package),
        Commands::List => commands::list::list(&conn),
        Commands::Info { package } => commands::info::info(&conn, package),
        Commands::Pack => commands::pack::pack(),
        Commands::Sign {
            package,
            key,
            comment,
        } => commands::sign::sign(package, key, comment.as_deref()),
        Commands::Verify {
            package,
        } => commands::verify::verify_content_checksum(package),
        Commands::Rollback { package_id } => commands::rollback::rollback(&mut conn, *package_id),
        Commands::Update { package } => commands::update::update(&mut conn, package),
        Commands::Repo(repo_cmd) => match repo_cmd {
            RepoCommands::Add { url, name } => commands::repo::add_repo(&mut conn, url, name),
            RepoCommands::Search { package, repo } => {
                commands::repo::search_repo(&conn, package, repo.as_deref())
            }
            RepoCommands::Install { package, repo } => {
                commands::repo::install_from_repo(&mut conn, package, repo.as_deref())
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}
