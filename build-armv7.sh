#!/usr/bin/env bash
set -e

echo "Building Speedtest Tracker Admin for armv7..."

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "Installing cross for easy cross-compilation..."
    cargo install cross  --git https://github.com/cross-rs/cross.git --rev "65fe72b0cdb1e7e0cc0652517498d4389cc8f5cf"
fi


# mount /nix/store into the container if it exists
if [[ -d /nix/store ]]; then
    export NIX_STORE=/nix/store
fi

# Build for armv7
echo "Building release binary..."
SUPPRESS_BOLTDB_WARNING=1 cross build --release --target armv7-unknown-linux-musleabihf

# Show binary info
BINARY="target/armv7-unknown-linux-musleabihf/release/speedtest-tracker"
echo ""
echo "✅ Build complete!"
echo "Binary location: $BINARY"
echo "Binary size: $(du -h $BINARY | cut -f1)"
echo ""
echo "To deploy:"
echo "1. Copy binary to your armv7 device"
echo "2. Set DATABASE_URL environment variable"
echo "3. Run: ./speedtest-tracker"
