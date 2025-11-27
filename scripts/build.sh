#!/bin/bash
# Build script for Sikuli-D Tauri IDE
# Usage: ./build.sh [--release] [--no-increment]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
IDE_DIR="$ROOT_DIR/ide-rs-tauri"

RELEASE_FLAG=""
INCREMENT=true

# Parse arguments
for arg in "$@"; do
    case $arg in
        --release)
            RELEASE_FLAG="--release"
            ;;
        --no-increment)
            INCREMENT=false
            ;;
    esac
done

# Increment PATCH version
if [ "$INCREMENT" = true ]; then
    echo "=== Incrementing PATCH version ==="
    "$SCRIPT_DIR/increment_patch.sh" "$IDE_DIR"
    echo ""
fi

# Build
echo "=== Building Tauri IDE ==="
cd "$IDE_DIR"

if [ -n "$RELEASE_FLAG" ]; then
    echo "Building release version..."
    ~/.cargo/bin/cargo-tauri build
else
    echo "Building debug version..."
    ~/.cargo/bin/cargo-tauri build --debug
fi

echo ""
echo "=== Build complete ==="

# Show output location
if [ -n "$RELEASE_FLAG" ]; then
    echo "Output: $IDE_DIR/target/release/sikulix-ide-tauri.exe"
else
    echo "Output: $IDE_DIR/target/debug/sikulix-ide-tauri.exe"
fi
