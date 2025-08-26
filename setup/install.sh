#!/bin/bash
# Trading Bot - Complete Installation Script
# This script installs everything needed for the trading bot (requires Git)

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
REPO_URL="https://github.com/KingGekko/trading-bot.git"
PROJECT_DIR="trading-bot"

echo -e "${CYAN}🚀 Trading Bot - Complete Installation${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""
echo "This script will install everything needed for the trading bot:"
echo "  1. System dependencies (build tools, OpenSSL, etc.)"
echo "  2. Rust programming language" 
echo "  3. Clone and build the trading bot from GitHub"
echo "  4. Install and configure Ollama AI"
echo "  5. Download AI models (tinyllama + optional extras)"
echo "  6. Test the complete installation"
echo ""
echo -e "${YELLOW}⏳ Estimated time: 10-20 minutes (depending on internet speed)${NC}"
echo ""

# Check if Git is available
if ! command -v git &> /dev/null; then
    echo -e "${RED}❌ Git is required but not installed!${NC}"
    echo ""
    echo "Please install Git first:"
    echo "  Ubuntu/Debian: sudo apt update && sudo apt install -y git"
    echo "  CentOS/RHEL:   sudo yum install -y git"
    echo "  Fedora:        sudo dnf install -y git"
    echo "  Alpine:        sudo apk add git"
    echo ""
    echo "After installing Git, run this script again."
    exit 1
fi

echo -e "${GREEN}✅ Git is available: $(git --version)${NC}"

# Confirmation
echo ""
echo -e "${BLUE}🎯 Continue with complete setup? (y/n)${NC}"
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Setup cancelled${NC}"
    exit 0
fi

echo ""
echo -e "${GREEN}🚀 Starting complete setup...${NC}"
echo ""

# ============================================================================
# STEP 1: DETECT SYSTEM AND INSTALL DEPENDENCIES
# ============================================================================

echo -e "${PURPLE}=================================="
echo -e "📦 STEP 1/6: Installing dependencies"
echo -e "==================================${NC}"

# Detect Linux distribution
if [ -f /etc/debian_version ]; then
    DISTRO="debian"
    DISTRO_NAME="Ubuntu/Debian"
elif [ -f /etc/redhat-release ]; then
    DISTRO="redhat"
    DISTRO_NAME="CentOS/RHEL/Fedora"
elif [ -f /etc/alpine-release ]; then
    DISTRO="alpine"
    DISTRO_NAME="Alpine Linux"
else
    DISTRO="unknown"
    DISTRO_NAME="Unknown"
fi

echo -e "${BLUE}📋 Detected OS: $DISTRO_NAME${NC}"

# Update package manager
echo "📦 Updating package manager..."
case $DISTRO in
    "debian")
        sudo apt update
        ;;
    "redhat")
        if command -v dnf &> /dev/null; then
            sudo dnf update -y
        else
            sudo yum update -y
        fi
        ;;
    "alpine")
        sudo apk update
        ;;
    *)
        echo -e "${RED}⚠️  Unknown distribution. Please install dependencies manually.${NC}"
        echo "Required: git, curl, gcc, openssl-dev, pkg-config"
        exit 1
        ;;
esac

# Install build dependencies
echo "🛠️  Installing build dependencies..."
case $DISTRO in
    "debian")
        sudo apt install -y \
            curl \
            build-essential \
            pkg-config \
            libssl-dev \
            ca-certificates
        ;;
    "redhat")
        if command -v dnf &> /dev/null; then
            sudo dnf install -y \
                curl \
                gcc \
                gcc-c++ \
                openssl-devel \
                pkg-config \
                ca-certificates
        else
            sudo yum install -y \
                curl \
                gcc \
                gcc-c++ \
                openssl-devel \
                pkg-config \
                ca-certificates
        fi
        ;;
    "alpine")
        sudo apk add \
            curl \
            build-base \
            openssl-dev \
            pkgconfig \
            ca-certificates
        ;;
esac

echo -e "${GREEN}✅ Dependencies installed successfully!${NC}"

# ============================================================================
# STEP 2: INSTALL RUST
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🦀 STEP 2/6: Installing Rust"
echo -e "==================================${NC}"

# Check if Rust is already installed
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✅ Rust is already installed!${NC}"
    echo "📋 Current version:"
    rustc --version
    cargo --version
else
    echo "📥 Installing Rust programming language..."
    
    # Download and install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the cargo environment
    echo "🔧 Setting up Rust environment..."
    source ~/.cargo/env
    
    # Add cargo to PATH permanently
    if ! grep -q 'source ~/.cargo/env' ~/.bashrc; then
        echo 'source ~/.cargo/env' >> ~/.bashrc
    fi
    
    if ! grep -q 'source ~/.cargo/env' ~/.profile; then
        echo 'source ~/.cargo/env' >> ~/.profile
    fi
    
    echo -e "${GREEN}✅ Rust installed successfully!${NC}"
    echo "📋 Installed versions:"
    rustc --version
    cargo --version
fi

# Ensure Rust is available in current session
source ~/.cargo/env

# ============================================================================
# STEP 3: CLONE AND BUILD TRADING BOT
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🐙 STEP 3/6: Cloning and building"
echo -e "==================================${NC}"

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not available in PATH${NC}"
    echo "🔧 Trying to source Rust environment..."
    source ~/.cargo/env
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Still can't find Rust. Please restart terminal and try again.${NC}"
        exit 1
    fi
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

echo -e "${GREEN}📁 Repository cloned successfully!${NC}"

# Build the project
echo ""
echo "🔨 Building trading bot (release mode)..."
echo -e "${YELLOW}⏳ This may take several minutes on first build...${NC}"

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
    
    echo ""
    echo "🧪 Testing binary..."
    ./target/release/trading_bot --help
    
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo "🔍 Check the error messages above"
    exit 1
fi

# ============================================================================
# STEP 4: INSTALL OLLAMA
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🤖 STEP 4/6: Installing Ollama"
echo -e "==================================${NC}"

# Check if Ollama is already installed
if command -v ollama &> /dev/null; then
    echo -e "${GREEN}✅ Ollama is already installed!${NC}"
    ollama --version
else
    echo "📥 Installing Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
    echo -e "${GREEN}✅ Ollama installed successfully!${NC}"
fi

# Start Ollama service in background
echo "🚀 Starting Ollama service..."
ollama serve &

# Wait a moment for service to start
sleep 5

# Verify Ollama is running
echo "🔍 Verifying Ollama service..."
if pgrep -x "ollama" > /dev/null; then
    echo -e "${GREEN}✅ Ollama service is running${NC}"
else
    echo -e "${YELLOW}⚠️  Ollama service not detected, trying to start again...${NC}"
    ollama serve &
    sleep 3
    if pgrep -x "ollama" > /dev/null; then
        echo -e "${GREEN}✅ Ollama service started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start Ollama service${NC}"
        echo "🔧 Please check Ollama installation and try again"
        exit 1
    fi
fi

# ============================================================================
# STEP 5: DOWNLOAD AI MODELS
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "📦 STEP 5/6: Downloading AI models"
echo -e "==================================${NC}"

# Pull the default model (llama2 - around 6GB, good quality)
echo "📥 Downloading llama2 model (default for trading bot)..."
echo -e "${YELLOW}⏳ This will download ~6GB and may take 10-30 minutes depending on internet speed...${NC}"
ollama pull llama2

# Ask about additional models
echo ""
echo -e "${BLUE}🎯 Would you like to install additional models for different use cases? (y/n)${NC}"
echo "   • tinyllama (1.1GB) - Ultra-fast responses, basic analysis"
echo "   • phi (2.7GB) - Microsoft's efficient model, good analysis quality"
echo "   • gemma2:2b (1.5GB) - Google's optimized model, excellent analysis"
read -r model_response

if [[ "$model_response" =~ ^[Yy]$ ]]; then
    echo "📦 Installing additional models..."
    echo -e "${YELLOW}⏳ This may take several minutes...${NC}"
    
    echo "📥 Installing tinyllama (ultra-fast)..."
    ollama pull tinyllama
    
    echo "📥 Installing phi (Microsoft's efficient model)..."
    ollama pull phi
    
    echo "📥 Installing gemma2:2b (Google's optimized model)..."
    ollama pull gemma2:2b
    
    echo -e "${GREEN}✅ Additional models installed!${NC}"
fi

echo ""
echo "📋 Installed models:"
ollama list

# ============================================================================
# STEP 6: TEST COMPLETE INSTALLATION
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🧪 STEP 6/6: Testing installation"
echo -e "==================================${NC}"

# Test binary functionality
echo "🧪 Testing binary functionality..."
./target/release/trading_bot --help
echo -e "${GREEN}✅ Binary is working!${NC}"

# Test Ollama connectivity
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
echo "📊 Expected: 15-25 second response with excellent analysis (llama2)"
echo ""

# Run the test
./target/release/trading_bot -t "What is blockchain?"

# ============================================================================
# SETUP COMPLETE
# ============================================================================

echo ""
echo -e "${GREEN}🎉 SETUP COMPLETE!${NC}"
echo -e "${GREEN}==================${NC}"
echo ""
echo -e "${GREEN}✅ Trading bot is fully installed and tested!${NC}"
echo "📍 Location: $(pwd)/target/release/trading_bot"
echo ""
echo -e "${CYAN}📊 Performance Summary:${NC}"
echo "   • Response time: 15-25 seconds (llama2 default)"
echo "   • Analysis quality: ⭐⭐⭐⭐⭐ Excellent structured analysis"
echo "   • Response length: ~300-500 words"
echo "   • Streaming: Real-time output during generation"
echo "   • Model size: ~6GB (llama2)"
echo ""
echo -e "${CYAN}📋 Quick Reference:${NC}"
echo "   • Test mode:        ./target/release/trading_bot -t 'Your prompt'"
echo "   • Interactive mode: ./target/release/trading_bot -i"
echo "   • Single prompt:    ./target/release/trading_bot -p 'Your prompt'"
echo "   • View logs:        ./target/release/trading_bot -l"
echo ""
echo -e "${CYAN}🔧 Configuration:${NC}"
echo "   • Config file: $(pwd)/config.env"
echo "   • Log directory: $(pwd)/ollama_logs/"
echo "   • Binary size: $(du -h target/release/trading_bot | cut -f1)"
echo "   • Default model: llama2 (~6GB)"
echo ""
echo -e "${CYAN}💡 Tips:${NC}"
echo "   • For faster responses: Set OLLAMA_MODEL=tinyllama in config.env"
echo "   • For best analysis: Set OLLAMA_MODEL=llama2 in config.env (default)"
echo "   • For balanced performance: Set OLLAMA_MODEL=phi in config.env"
echo "   • For system-wide access: sudo cp target/release/trading_bot /usr/local/bin/"
echo ""
echo -e "${CYAN}🎯 What's next?${NC}"
echo "   • Try interactive mode: ./target/release/trading_bot -i"
echo "   • Test with prompts: ./target/release/trading_bot -t 'Analyze Bitcoin'"
echo "   • View performance logs: ./target/release/trading_bot -l"
echo ""
echo -e "${BLUE}📚 Documentation: https://github.com/KingGekko/trading-bot${NC}"
echo ""
echo -e "${GREEN}🚀 Happy trading!${NC}" 