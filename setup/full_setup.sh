#!/bin/bash
# Trading Bot - Complete Setup Script
# This script runs the full installation process

set -e  # Exit on any error

echo "🚀 Trading Bot - Complete Setup"
echo "==============================="
echo ""
echo "This script will install everything needed for the trading bot:"
echo "  1. System dependencies (git, build tools, etc.)"
echo "  2. Rust programming language" 
echo "  3. Clone and build the trading bot"
echo "  4. Install and configure Ollama AI"
echo "  5. Test the complete installation"
echo ""
echo "⏳ Estimated time: 10-20 minutes (depending on internet speed)"
echo ""

# Confirmation
echo "🎯 Continue with full setup? (y/n)"
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo "❌ Setup cancelled"
    exit 0
fi

echo ""
echo "🚀 Starting full setup..."
echo ""

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Step 1: Install dependencies
echo "=================================="
echo "📦 STEP 1/5: Installing dependencies"
echo "=================================="
bash "$SCRIPT_DIR/install_dependencies.sh"

echo ""
echo "=================================="
echo "🦀 STEP 2/5: Installing Rust"
echo "=================================="
bash "$SCRIPT_DIR/install_rust.sh"

# Source Rust environment
source ~/.cargo/env

echo ""
echo "=================================="
echo "🐙 STEP 3/5: Cloning and building"
echo "=================================="
bash "$SCRIPT_DIR/clone_and_build.sh"

echo ""
echo "=================================="
echo "🤖 STEP 4/5: Installing Ollama"
echo "=================================="
bash "$SCRIPT_DIR/install_ollama.sh"

echo ""
echo "=================================="
echo "🧪 STEP 5/5: Testing installation"
echo "=================================="
bash "$SCRIPT_DIR/test_installation.sh"

echo ""
echo "🎉 SETUP COMPLETE!"
echo "=================="
echo ""
echo "✅ Trading bot is fully installed and tested!"
echo "📍 Location: $(pwd)/trading-bot/target/release/trading_bot"
echo ""
echo "🎯 What's next?"
echo "   • Try interactive mode: cd trading-bot && ./target/release/trading_bot -i"
echo "   • Test with prompts: ./target/release/trading_bot -t 'Analyze Bitcoin'"
echo "   • View performance logs: ./target/release/trading_bot -l"
echo ""
echo "📚 Documentation: https://github.com/KingGekko/trading-bot"
echo ""
echo "🚀 Happy trading!"