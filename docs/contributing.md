# Contributing to lpkg

We welcome contributions to the `lpkg` project! Whether you're a seasoned Rustacean or new to open source, your help is valuable. This document outlines the guidelines for contributing to `lpkg`.

## How to Contribute

There are many ways to contribute to `lpkg`:

*   **Reporting Bugs**: If you find a bug, please report it on our [issue tracker](https://github.com/linuxpackage-org/lpkg/issues). Provide clear steps to reproduce, expected behavior, and actual behavior.
*   **Suggesting Features**: Have an idea for a new feature or improvement? Open an issue on the [issue tracker](https://github.com/linuxpackage-org/lpkg/issues) to discuss it.
*   **Writing Code**: Contribute bug fixes, new features, or improvements to existing code.
*   **Improving Documentation**: Help us improve our documentation, including the `README.md`, `VISION.md`, and the `docs/` directory.
*   **Testing**: Help us test new features and bug fixes.

## Getting Started with Code Contributions

1.  **Fork the Repository**: Fork the `lpkg` repository on GitHub.
2.  **Clone Your Fork**: Clone your forked repository to your local machine:

    ```bash
    git clone https://github.com/YOUR_USERNAME/lpkg.git
    cd lpkg
    ```

3.  **Create a New Branch**: Create a new branch for your feature or bug fix:

    ```bash
    git checkout -b feature/your-feature-name
    # or
    git checkout -b bugfix/your-bug-fix
    ```

4.  **Make Your Changes**: Implement your changes. Ensure your code adheres to the existing coding style and conventions.

5.  **Write Tests**: If you're adding new features or fixing bugs, please write unit and/or integration tests to cover your changes. This helps ensure the stability and correctness of `lpkg`.

6.  **Run Tests**: Before submitting your changes, run all tests to ensure everything passes:

    ```bash
    cargo test
    ```

7.  **Format and Lint Your Code**: Ensure your code is properly formatted and linted:

    ```bash
    cargo fmt --all
    cargo clippy --all-targets -- -D warnings
    ```

8.  **Commit Your Changes**: Write clear and concise commit messages. Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification if possible.

    ```bash
    git commit -m "feat: Add new feature X"
    # or
    git commit -m "fix: Fix bug Y"
    ```

9.  **Push to Your Fork**: Push your changes to your forked repository:

    ```bash
    git push origin feature/your-feature-name
    ```

10. **Create a Pull Request**: Open a pull request from your forked repository to the `main` branch of the official `lpkg` repository. Provide a clear description of your changes.

## Code Style and Conventions

*   **Rustfmt**: We use `rustfmt` for code formatting. Please run `cargo fmt --all` before committing.
*   **Clippy**: We use `clippy` for linting. Please run `cargo clippy --all-targets -- -D warnings` to catch common mistakes and improve code quality.
*   **Error Handling**: Use `anyhow::Result` for error propagation and `Context` for adding context to errors.
*   **Comments**: Add comments where necessary to explain complex logic or design decisions. Avoid excessive comments for self-explanatory code.

## Issue Tracker

Our issue tracker is hosted on GitHub: [https://github.com/linuxpackage-org/lpkg/issues](https://github.com/linuxpackage-org/lpkg/issues)

## Community

Join our community to discuss `lpkg`, ask questions, and connect with other contributors. (Link to community forums/chat will be added here once available).

Thank you for contributing to `lpkg`!