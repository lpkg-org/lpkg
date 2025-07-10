#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "📦 Starting lpkg installation script..."

# 1. Install Rust
if command -v cargo &> /dev/null; then
    echo "✅ Rust is already installed. Skipping Rust toolchain installation."
    . "$HOME/.cargo/env" # Ensure cargo is in PATH for current session
else
    echo "🔧 Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
    echo "✅ Rust installed successfully."
fi

# 2. Clone or update lpkg repository
echo "📥 Cloning lpkg repository..."
if [ -d "lpkg" ]; then
    echo "📂 'lpkg' directory already exists. Pulling latest changes..."
    cd lpkg
    git pull origin main || echo "⚠️ Warning: Failed to pull from remote. Using existing code."
else
    git clone https://github.com/mada-muniraja/lpkg.git
    cd lpkg
fi
echo "✅ lpkg repository ready."

# 3. Build lpkg in release mode
echo "⚙️ Building lpkg in release mode..."
cargo build --release
echo "✅ Build completed."

# 4. Install binary and configure system directories
echo "🚀 Installing lpkg binary and setting up system directories (requires sudo)..."
sudo bash -c '
    set -e
    install -Dm755 target/release/lpkg /usr/local/bin/lpkg
    mkdir -p /var/lib/lpkg
    chown root:root /var/lib/lpkg
    chmod 755 /var/lib/lpkg

    touch /var/lib/lpkg/db.sqlite
    chown root:root /var/lib/lpkg/db.sqlite
    chmod 644 /var/lib/lpkg/db.sqlite
'

echo "✅ System directories configured and binary installed."

# 5. Verify installation
echo "🔍 Verifying lpkg installation..."
if lpkg --version >/dev/null 2>&1; then
    echo "✅ lpkg installation complete! 🎉"
else
    echo "❌ lpkg installation failed or not in PATH."
    exit 1
fi
