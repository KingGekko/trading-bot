#!/bin/bash
# Trading Bot - Clone and Build Script
# This script clones the repository and builds the trading bot

set -e  # Exit on any error

echo "🐙 Trading Bot - Clone and Build"
echo "================================"

# Configuration
REPO_URL="https://github.com/KingGekko/trading-bot.git"
PROJECT_DIR="trading-bot"

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed or not in PATH"
    echo "🔧 Please run ./install_rust.sh first"
    echo "💡 Or run: source ~/.cargo/env"
    exit 1
fi

echo "📋 Using Rust version:"
rustc --version
cargo --version
echo ""

# Remove existing directory if it exists
if [ -d "$PROJECT_DIR" ]; then
    echo "🗑️  Removing existing directory: $PROJECT_DIR"
    rm -rf "$PROJECT_DIR"
fi

# Clone the repository
echo "📥 Cloning repository from GitHub..."
git clone "$REPO_URL"

# Navigate to project directory
cd "$PROJECT_DIR"

echo "📁 Repository cloned successfully!"
echo "📋 Project structure:"
ls -la

# Build the project
echo ""
echo "🔨 Building trading bot (release mode)..."
echo "⏳ This may take several minutes on first build..."

# Build with release optimizations
cargo build --release

# Check if build was successful
if [ -f "target/release/trading_bot" ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "📍 Binary location: $(pwd)/target/release/trading_bot"
    echo "📏 Binary size: $(du -h target/release/trading_bot | cut -f1)"
    
    # Make binary executable
    chmod +x target/release/trading_bot
    
    echo ""
    echo "🧪 Testing binary..."
    ./target/release/trading_bot --help
    
    echo ""
    echo "🎉 Trading bot is ready!"
    echo "📋 Available commands:"
    echo "   • Test mode:        ./target/release/trading_bot -t 'Your prompt'"
    echo "   • Interactive mode: ./target/release/trading_bot -i"
    echo "   • Single prompt:    ./target/release/trading_bot -p 'Your prompt'"
    echo "   • View logs:        ./target/release/trading_bot -l"
    echo ""
    echo "⚠️  Important: Make sure Ollama is installed and running!"
    echo "🎯 Next step: Run ../install_ollama.sh"
    
else
    echo "❌ Build failed!"
    echo "🔍 Check the error messages above"
    exit 1
fi