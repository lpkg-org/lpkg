# LinuxPackage (lpkg)

<div align="center">
  <img src="/assets/logo2.png" alt="LinuxPackage Logo">
</div>

**lpkg** is a universal, native, zero-runtime package manager designed for all major Linux distributions. It aims to provide a fast, lightweight, and secure way to install software natively without sandboxing or additional runtimes like Snap or Flatpak.

## Overview

- **Native Installation**: Installs software directly to the system, ensuring compatibility and performance.
- **Custom .lpkg Format**: Uses a compressed `.tar.zst` archive with metadata for efficient and secure package distribution.
- **Written in Rust**: Leverages Rust's performance and safety features for a robust CLI tool.
- **Lightweight CLI**: The `lpkg` command-line interface is minimal, modern, and comparable in performance to tools like `pacman` or `apk`.
- **Signing/Verification**: Supports package signing and verification using tools like `minisign` for security.
- **Remote Repositories**: Optional support for remote repositories with static indexes for package distribution.

**Note**: This is not a wrapper around existing package managers like `apt`, `yum`, or `pacman`. It is a fully native package manager built from scratch.

## Project Goals

The primary goal of LinuxPackage is to create a unified package management solution that works seamlessly across different Linux distributions, providing:

- A consistent user experience.
- High performance with minimal overhead.
- Enhanced security through package signing.
- Flexibility for both local and remote package management.

## Installation

### Quick Install (Recommended)

To quickly install `lpkg` and its dependencies (Rust), run the following command:

```bash
curl -sSf https://raw.githubusercontent.com/lpkg-org/lpkg/refs/heads/main/install.sh | sh
```

## Uninstallation

To completely uninstall `lpkg` from your system, including the binary, database, and installed applications, run the following command:

```bash
curl -sSf https://raw.githubusercontent.com/lpkg-org/lpkg/refs/heads/main/uninstall.sh | sudo bash
```

### Build from Source

Alternatively, you can build `lpkg` from source. Follow these steps:

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/lpkg-org/lpkg.git
   cd lpkg
   ```

2. **Build the Project**:
   Ensure you have Rust and Cargo installed, then build the release version:

   ```bash
   cargo build --release
   ```

3. **Install the Binary**:
   Move the compiled binary to a system path:

   ```bash
   sudo mv target/release/lpkg /usr/local/bin/
   ```

4. **Verify Installation**:
   Check if `lpkg` is installed correctly:

   ```bash
   lpkg --version
   ```

## Usage

`lpkg` provides a simple and intuitive command-line interface. Below are the basic commands implemented so far:

- **Install a Package**:
  Install a local `.lpkg` file:

  ```bash
  sudo lpkg install ./file.lpkg
  ```

- **Remove a Package**:
  Remove an installed package by name:

  ```bash
  sudo lpkg remove <package_name>
  ```

- **List Installed Packages**:
  Display a list of all installed packages:

  ```bash
  lpkg list
  ```

- **Get Package Information**:
  Show detailed information about an installed package:

  ```bash
  lpkg info <package_name>
  ```

- **Pack a Package**:
  Create a `.lpkg` file from a directory containing `meta.toml`, files, and optional scripts:

  ```bash
  sudo lpkg pack /path/to/package_directory
  ```

- **Sign a Package**:
  Sign a `.lpkg` file with a private key:

  ```bash
  lpkg sign <package_path> <key_path>
  ```

- **Verify a Package**:
  Verify a package's content checksum and signature:

  ```bash
  lpkg verify <package_path>
  ```

- **Rollback a Package**:
  Rollback a package installation to a previous state:

  ```bash
  sudo lpkg rollback <package_id>
  ```

- **Update a Package**:
  Update an installed package to its latest version from configured repositories:

  ```bash
  sudo lpkg update <package_name>
  ```

## Package Format (.lpkg)

The `.lpkg` format is a compressed `.tar.zst` archive with the following structure:

- **meta.toml**: Contains metadata about the package (name, version, description, dependencies, etc.).
- **files/**: Directory containing the files to be installed on the system.
- **scripts/**: Optional directory for scripts like `pre-install.sh`, `post-install.sh`, etc., to run during installation or removal.

## Contributing

We welcome contributions to the LinuxPackage project! To get involved:

1. Fork the repository on GitHub: [https://github.com/lpkg-org/lpkg](https://github.com/lpkg-org/lpkg)
2. Make your changes and submit a pull request.
3. Join the discussion on our community forums or issue tracker for feature requests and bug reports.

## License

This project is licensed under the MIT License or Apache License 2.0, at your option. See the LICENSE file for details.

## Contact

For more information, visit our website at [https://lpkg.org](https://lpkg.org) or contact us at [contact@lpkg.org](mailto:contact@lpkg.org).
