#!/bin/bash

# Simple wrapper script to fix protobuf issues
# Run this from the project root directory

echo "🔧 Protobuf Fix Wrapper"
echo "======================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Not in a Rust project directory"
    echo "Please run this script from the project root (where Cargo.toml is located)"
    exit 1
fi

# Run the comprehensive fix script
if [ -f "scripts/fix_protobuf.sh" ]; then
    echo "🚀 Running protobuf fix script..."
    ./scripts/fix_protobuf.sh
else
    echo "❌ Fix script not found at scripts/fix_protobuf.sh"
    echo "Please ensure the script exists and is executable"
    exit 1
fi 