#!/bin/bash
# Increment PATCH version for Sikuli-D Rust projects
# Usage: ./increment_patch.sh <project_dir>
# Example: ./increment_patch.sh ../ide-rs-tauri

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

PROJECT_DIR="${1:-$ROOT_DIR/ide-rs-tauri}"

if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Directory $PROJECT_DIR does not exist"
    exit 1
fi

VERSION_FILE="$PROJECT_DIR/version.txt"
CARGO_TOML="$PROJECT_DIR/Cargo.toml"
TAURI_CONF="$PROJECT_DIR/tauri.conf.json"

# Read MAJOR.MINOR from VERSION file
MAJOR_MINOR=$(cat "$ROOT_DIR/VERSION" | tr -d '\n\r')

# Read current PATCH or initialize to 0
if [ -f "$VERSION_FILE" ]; then
    PATCH=$(cat "$VERSION_FILE" | tr -d '\n\r')
else
    PATCH=0
fi

# Increment PATCH
NEW_PATCH=$((PATCH + 1))

# Write new PATCH
echo "$NEW_PATCH" > "$VERSION_FILE"

# Full version string
FULL_VERSION="${MAJOR_MINOR}.${NEW_PATCH}"

echo "Version: $FULL_VERSION"

# Update Cargo.toml
if [ -f "$CARGO_TOML" ]; then
    sed -i "s/^version = \"[^\"]*\"/version = \"$FULL_VERSION\"/" "$CARGO_TOML"
    echo "Updated: $CARGO_TOML"
fi

# Update tauri.conf.json (if exists)
if [ -f "$TAURI_CONF" ]; then
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$FULL_VERSION\"/" "$TAURI_CONF"
    echo "Updated: $TAURI_CONF"
fi

echo "PATCH incremented to $NEW_PATCH (full version: $FULL_VERSION)"
