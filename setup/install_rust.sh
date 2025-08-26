#!/bin/bash
# Trading Bot - Rust Installation Script
# This script installs Rust programming language

set -e  # Exit on any error

echo "🦀 Trading Bot - Rust Installation"
echo "=================================="

# Check if Rust is already installed
if command -v cargo &> /dev/null; then
    echo "✅ Rust is already installed!"
    echo "📋 Current version:"
    rustc --version
    cargo --version
    echo ""
    echo "🔄 To update Rust, run: rustup update"
    echo "🎯 Next step: Run ./clone_and_build.sh"
    exit 0
fi

echo "📥 Installing Rust programming language..."

# Download and install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source the cargo environment
echo "🔧 Setting up Rust environment..."
source ~/.cargo/env

# Verify installation
echo "✅ Rust installed successfully!"
echo "📋 Installed versions:"
rustc --version
cargo --version

# Add cargo to PATH permanently
echo ""
echo "📝 Adding Rust to your PATH..."
if ! grep -q 'source ~/.cargo/env' ~/.bashrc; then
    echo 'source ~/.cargo/env' >> ~/.bashrc
    echo "✅ Added Rust to ~/.bashrc"
fi

if ! grep -q 'source ~/.cargo/env' ~/.profile; then
    echo 'source ~/.cargo/env' >> ~/.profile
    echo "✅ Added Rust to ~/.profile"
fi

echo ""
echo "🎯 Installation complete!"
echo "💡 Restart your terminal or run: source ~/.cargo/env"
echo "🎯 Next step: Run ./clone_and_build.sh"