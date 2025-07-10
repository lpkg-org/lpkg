#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "🗑️ Starting lpkg uninstallation script..."

# 1. Remove the lpkg binary
echo "Removing lpkg binary from /usr/local/bin/lpkg..."
sudo rm -f /usr/local/bin/lpkg
echo "✅ lpkg binary removed."

# 2. Remove the lpkg database and its directory
echo "Removing lpkg database and its directory (/var/lib/lpkg)..."
sudo rm -rf /var/lib/lpkg
echo "✅ lpkg database and directory removed."

# 3. Remove installed applications directory
echo "Removing lpkg installed applications directory (/usr/local/lpkg/packages)..."
sudo rm -rf /usr/local/lpkg/packages
echo "✅ lpkg installed applications directory removed."

# 4. Remove the lpkg repository directory (if it exists in the current working directory)
#    This assumes the script is run from the directory where lpkg was cloned.
if [ -d "lpkg" ]; then
    echo "Removing lpkg source directory..."
    rm -rf lpkg
    echo "✅ lpkg source directory removed."
else
    echo "⚠️ lpkg source directory not found in current location. Skipping removal."
fi

echo "
" # Add a newline for better readability
echo "Uninstallation complete. All lpkg components, including the database and installed applications, have been removed."
