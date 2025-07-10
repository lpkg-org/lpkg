# lpkg Architecture Overview

This document provides a high-level overview of the `lpkg` package manager's architecture.

## Core Components

`lpkg` is designed with a modular architecture, separating concerns into distinct components:

1.  **CLI (Command-Line Interface)**: Located in `src/main.rs` and `src/commands/`, this component handles user interaction, parses commands, and orchestrates operations by calling functions in other modules.

2.  **Package Management Core**: This is the heart of `lpkg`, responsible for handling `.lpkg` files.
    *   **Archive Management (`src/package/archive.rs`)**: Manages the creation and extraction of `.lpkg` archives (which are essentially `.tar.zst` files).
    *   **Metadata Handling (`src/package/metadata.rs`)**: Parses and manages `meta.toml` files, which contain package information like name, version, dependencies, and scripts.

3.  **Database (`src/db/`)**: `lpkg` uses an SQLite database to store information about installed packages, their files, and dependencies.
    *   **Connection Management (`src/db/connection.rs`)**: Handles establishing and managing connections to the SQLite database.
    *   **Operations (`src/db/operations.rs`)**: Provides an API for interacting with the database, including adding, removing, querying, and updating package and file information.
    *   **Schema (`src/db/schema.rs`)**: Defines the database schema and handles database initialization/migrations.

4.  **Repository Management (`src/repository/`)**: This component handles interactions with package repositories.
    *   **Index Management**: Fetches, caches, and searches package indexes from remote or local repositories.
    *   **Package Download**: Manages downloading `.lpkg` files from repositories.

5.  **Utilities (`src/utils/`)**: Contains common utility functions used across different components.
    *   **Checksums (`src/utils/checksum.rs`)**: Provides functionality for calculating and verifying file checksums (SHA256).
    *   **File Operations (`src/utils/file_ops.rs`)**: Basic file system operations like copying and removing files.

## Data Flow (High-Level)

When a user executes a command (e.g., `lpkg install package.lpkg`):

1.  The **CLI** component (`src/main.rs`, `src/commands/install.rs`) receives the command.
2.  It calls the **Archive Management** (`src/package/archive.rs`) to extract the `.lpkg` file to a temporary location.
3.  The **Metadata Handling** (`src/package/metadata.rs`) parses the `meta.toml` file from the extracted package.
4.  The **Database Operations** (`src/db/operations.rs`) are used to check for existing installations and dependencies.
5.  If dependencies are met and the package is not already installed, files are copied to their system locations using **File Operations** (`src/utils/file_ops.rs`).
6.  Package and file information is recorded in the database via **Database Operations**.
7.  Pre/post-installation scripts (if any) are executed.
8.  Temporary files are cleaned up.

## Security Considerations

`lpkg` incorporates security features such as package signing and verification (`src/commands/sign.rs`, `src/commands/verify.rs`) to ensure the authenticity and integrity of packages. This helps prevent the installation of tampered or malicious software.

## Future Enhancements

Future architectural considerations include:

*   **Advanced Dependency Resolution**: Implementing a more sophisticated dependency solver to handle complex dependency graphs, conflicts, and version ranges.
*   **Rollback Capabilities**: Designing a mechanism to revert installations or updates to a previous stable state.
*   **Plugin System**: Exploring a plugin architecture to allow for extensible functionality and support for various package types or repository protocols.
