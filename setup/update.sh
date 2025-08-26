#!/bin/bash
# Trading Bot - Update Script
# This script updates the trading bot binary without reinstalling dependencies

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_DIR="trading-bot"

echo -e "${CYAN}🔄 Trading Bot - Update Script${NC}"
echo -e "${CYAN}=============================${NC}"
echo ""
echo "This script will update your trading bot to the latest version:"
echo "  1. Pull latest code from GitHub"
echo "  2. Rebuild the binary (release mode)"
echo "  3. Test the updated binary"
echo "  4. Keep all existing configurations and logs"
echo ""
echo -e "${YELLOW}⏳ Estimated time: 5-15 minutes (much faster than full install)${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "install.sh" ] && [ ! -d "$PROJECT_DIR" ]; then
    echo -e "${RED}❌ Please run this script from the setup directory or trading-bot root${NC}"
    echo ""
    echo "Run from setup directory:"
    echo "  cd setup && ./update.sh"
    echo ""
    echo "OR run from trading-bot root:"
    echo "  cd trading-bot && ../setup/update.sh"
    exit 1
fi

# Determine the correct paths
if [ -f "install.sh" ]; then
    # We're in setup directory
    SETUP_DIR="."
    PROJECT_PATH="../$PROJECT_DIR"
else
    # We're in trading-bot root
    SETUP_DIR="../setup"
    PROJECT_PATH="."
fi

# Check if project directory exists
if [ ! -d "$PROJECT_PATH" ]; then
    echo -e "${RED}❌ Trading bot directory not found!${NC}"
    echo ""
    echo "Please run the full installation first:"
    echo "  curl -fsSL https://raw.githubusercontent.com/KingGekko/trading-bot/main/setup/install.sh -o install.sh && chmod +x install.sh && ./install.sh"
    exit 1
fi

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not available!${NC}"
    echo ""
    echo "Please run the full installation first:"
    echo "  curl -fsSL https://raw.githubusercontent.com/KingGekko/trading-bot/main/setup/install.sh -o install.sh && chmod +x install.sh && ./install.sh"
    exit 1
fi

# Check if Ollama is available
if ! command -v ollama &> /dev/null; then
    echo -e "${YELLOW}⚠️  Ollama not found, but continuing with binary update...${NC}"
    echo "You may need to install Ollama separately if you want to use the AI features."
fi

echo -e "${GREEN}✅ Prerequisites check passed!${NC}"
echo "📋 Rust version: $(rustc --version)"
echo "📋 Cargo version: $(cargo --version)"
echo ""

# Confirmation
echo -e "${BLUE}🎯 Continue with update? (y/n)${NC}"
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Update cancelled${NC}"
    exit 0
fi

echo ""
echo -e "${GREEN}🚀 Starting update process...${NC}"
echo ""

# ============================================================================
# STEP 1: UPDATE SOURCE CODE
# ============================================================================

echo -e "${PURPLE}=================================="
echo -e "📥 STEP 1/3: Updating source code"
echo -e "==================================${NC}"

cd "$PROJECT_PATH"

# Always download latest source (no Git required)
echo "📥 Downloading latest source code from GitHub..."

# Backup existing config and logs
echo "💾 Backing up existing configuration and logs..."
if [ -f "config.env" ]; then
    cp config.env config.env.backup
    echo "✅ Config backed up to config.env.backup"
fi

if [ -d "ollama_logs" ]; then
    cp -r ollama_logs ollama_logs.backup
    echo "✅ Logs backed up to ollama_logs.backup"
fi

# Remove existing source and download fresh
cd ..
rm -rf "$PROJECT_DIR"

echo "📥 Downloading latest source code..."
curl -L -o trading-bot.zip https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip

# Install unzip if not available
if ! command -v unzip &> /dev/null; then
    echo "📦 Installing unzip..."
    if command -v yum &> /dev/null; then
        sudo yum install -y unzip
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y unzip
    elif command -v apt &> /dev/null; then
        sudo apt install -y unzip
    elif command -v apk &> /dev/null; then
        sudo apk add unzip
    else
        echo -e "${RED}❌ Cannot install unzip automatically${NC}"
        echo "Please install unzip manually and try again"
        exit 1
    fi
fi

unzip trading-bot.zip
mv trading-bot-main "$PROJECT_DIR"
cd "$PROJECT_DIR"

# Restore config and logs
if [ -f "../config.env.backup" ]; then
    cp ../config.env.backup config.env
    echo "✅ Config restored from backup"
fi

if [ -d "../ollama_logs.backup" ]; then
    cp -r ../ollama_logs.backup ollama_logs
    echo "✅ Logs restored from backup"
fi

echo -e "${GREEN}✅ Source code updated successfully!${NC}"

# ============================================================================
# STEP 2: REBUILD BINARY
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🔨 STEP 2/3: Rebuilding binary"
echo -e "==================================${NC}"

echo "🔨 Building trading bot (release mode)..."
echo -e "${YELLOW}⏳ This may take 5-15 minutes...${NC}"

# Clean previous build artifacts
echo "🧹 Cleaning previous build artifacts..."
cargo clean

# Build with release optimizations
cargo build --release

# Check if build was successful
if [ -f "target/release/trading_bot" ]; then
    echo -e "${GREEN}✅ Build completed successfully!${NC}"
    echo ""
    echo "📍 Binary location: $(pwd)/target/release/trading_bot"
    echo "📏 Binary size: $(du -h target/release/trading_bot | cut -f1)"
    
    # Make binary executable
    chmod +x target/release/trading_bot
    
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo "🔍 Check the error messages above"
    exit 1
fi

# ============================================================================
# STEP 3: TEST UPDATED BINARY
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🧪 STEP 3/3: Testing updated binary"
echo -e "==================================${NC}"

# Test binary functionality
echo "🧪 Testing binary functionality..."
./target/release/trading_bot --help
echo -e "${GREEN}✅ Binary is working!${NC}"

# Test Ollama connectivity if available
if command -v ollama &> /dev/null; then
    echo ""
    echo "🧪 Testing Ollama connectivity..."
    echo "📋 Ollama version:"
    ollama --version
    
    echo "📋 Available models:"
    ollama list
    
    # Quick response test
    echo ""
    echo "🧪 Running quick response test..."
    echo -e "${YELLOW}⏳ Testing with prompt: 'What is blockchain?'${NC}"
    echo "📊 Expected: 8-12 second response with good analysis"
    echo ""
    
    # Run the test
    ./target/release/trading_bot -t "What is blockchain?"
else
    echo ""
    echo -e "${YELLOW}⚠️  Ollama not available, skipping AI tests${NC}"
    echo "Binary update completed successfully!"
fi

# ============================================================================
# UPDATE COMPLETE
# ============================================================================

echo ""
echo -e "${GREEN}🎉 UPDATE COMPLETE!${NC}"
echo -e "${GREEN}==================${NC}"
echo ""
echo -e "${GREEN}✅ Trading bot has been updated successfully!${NC}"
echo "📍 Location: $(pwd)/target/release/trading_bot"
echo ""
echo -e "${CYAN}📊 What Was Updated:${NC}"
echo "   • Source code: Latest version from GitHub"
echo "   • Binary: Freshly compiled with latest optimizations"
echo "   • Configuration: Preserved (config.env)"
echo "   • Logs: Preserved (ollama_logs/)"
echo ""
echo -e "${CYAN}📋 Quick Reference:${NC}"
echo "   • Test mode:        ./target/release/trading_bot -t 'Your prompt'"
echo "   • Interactive mode: ./target/release/trading_bot -i"
echo "   • Single prompt:    ./target/release/trading_bot -p 'Your prompt'"
echo "   • View logs:        ./target/release/trading_bot -l"
echo ""
echo -e "${CYAN}💡 Next Time:${NC}"
echo "   • Run this script again for future updates:"
echo "     cd setup && ./update.sh"
echo "   • Or from trading-bot root:"
echo "     ../setup/update.sh"
echo ""
echo -e "${BLUE}📚 Documentation: https://github.com/KingGekko/trading-bot${NC}"
echo ""
echo -e "${GREEN}🚀 Happy trading with your updated bot!${NC}" 