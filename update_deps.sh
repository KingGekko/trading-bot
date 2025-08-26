#!/bin/bash

# Simple wrapper script to update Rust dependencies
# Run this from the project root directory

echo "🦀 Rust Dependency Update Wrapper"
echo "=================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Not in a Rust project directory"
    echo "Please run this script from the project root (where Cargo.toml is located)"
    exit 1
fi

# Run the comprehensive update script
if [ -f "scripts/update_dependencies.sh" ]; then
    echo "🚀 Running comprehensive dependency update..."
    ./scripts/update_dependencies.sh
else
    echo "❌ Update script not found at scripts/update_dependencies.sh"
    echo "Please ensure the script exists and is executable"
    exit 1
fi 