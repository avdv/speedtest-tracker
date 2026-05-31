#!/usr/bin/env bash
set -e

echo "Building Speedtest Tracker Admin for armv7..."

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "Installing cross for easy cross-compilation..."
    cargo install cross
fi

# Build for armv7
echo "Building release binary..."
cross build --release --target armv7-unknown-linux-gnueabihf

# Show binary info
BINARY="target/armv7-unknown-linux-gnueabihf/release/speedtest-admin"
echo ""
echo "✅ Build complete!"
echo "Binary location: $BINARY"
echo "Binary size: $(du -h $BINARY | cut -f1)"
echo ""
echo "To deploy:"
echo "1. Copy binary to your armv7 device"
echo "2. Set DATABASE_URL environment variable"
echo "3. Run: ./speedtest-admin"
