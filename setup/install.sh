#!/bin/bash
# Trading Bot - Complete Installation Script for Ubuntu/Canonical
# This script installs everything needed for the trading bot

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

echo -e "${CYAN}🚀 Trading Bot - Complete Installation for Ubuntu${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""
echo "This script will install everything needed for the trading bot:"
echo "  1. System dependencies (build tools, OpenSSL, Git, etc.)"
echo "  2. Rust programming language"
echo "  3. Download and build the trading bot from GitHub"
echo "  4. Install and configure Ollama AI"
echo "  5. Download AI models (tinyllama + optional extras)"
echo "  6. Test the complete installation"
echo ""
echo -e "${YELLOW}⏳ Estimated time: 10-20 minutes (depending on internet speed)${NC}"
echo ""

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
# STEP 1: UPDATE SYSTEM AND INSTALL DEPENDENCIES
# ============================================================================

echo -e "${PURPLE}=================================="
echo -e "📦 STEP 1/6: Installing system dependencies"
echo -e "==================================${NC}"

echo "Detecting Ubuntu version..."
if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo "Detected OS: $PRETTY_NAME"
    if [[ "$ID" != "ubuntu" && "$ID" != "debian" && "$ID" != "linuxmint" ]]; then
        echo -e "${YELLOW}⚠️  This script is optimized for Ubuntu/Debian systems${NC}"
        echo "You may encounter issues on other distributions"
        echo ""
        echo -e "${BLUE}Continue anyway? (y/n)${NC}"
        read -r continue_response
        if [[ ! "$continue_response" =~ ^[Yy]$ ]]; then
            echo -e "${RED}❌ Setup cancelled${NC}"
            exit 0
        fi
    fi
else
    echo "Could not detect OS, assuming Ubuntu/Debian"
fi

echo ""
echo "Updating package lists..."
sudo apt update

echo ""
echo "Installing essential build dependencies..."
sudo apt install -y \
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    libssl-dev \
    libssl3 \
    ca-certificates \
    software-properties-common \
    apt-transport-https \
    gnupg \
    lsb-release \
    unzip \
    tar \
    gzip \
    bzip2 \
    xz-utils \
    zlib1g-dev \
    libbz2-dev \
    liblzma-dev \
    libncurses5-dev \
    libreadline-dev \
    libsqlite3-dev \
    libffi-dev \
    libgdbm-dev \
    libgdbm-compat-dev \
    libnss3-dev \
    libtinfo-dev \
    libc6-dev \
    libgcc-s1 \
    libstdc++6 \
    libc6 \
    libgcc-s1 \
    libstdc++6 \
    libc6 \
    libgcc-s1 \
    libstdc++6

echo ""
echo "Installing additional development tools..."
sudo apt install -y \
    cmake \
    ninja-build \
    clang \
    llvm \
    lld \
    gdb \
    valgrind \
    strace \
    ltrace \
    perf \
    linux-tools-common \
    linux-tools-generic

echo ""
echo "Installing Python and pip (for some build tools)..."
sudo apt install -y \
    python3 \
    python3-pip \
    python3-dev \
    python3-venv \
    python3-setuptools \
    python3-wheel

echo ""
echo "Installing Node.js and npm (for some build tools)..."
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt install -y nodejs

echo ""
echo "Installing Go (for some build tools)..."
if ! command -v go &> /dev/null; then
    GO_VERSION="1.21.5"
    GO_ARCH="linux-amd64"
    if [ "$(uname -m)" = "aarch64" ]; then
        GO_ARCH="linux-arm64"
    fi
    
    echo "Downloading Go $GO_VERSION..."
    curl -L -o go.tar.gz "https://go.dev/dl/go${GO_VERSION}.${GO_ARCH}.tar.gz"
    sudo tar -C /usr/local -xzf go.tar.gz
    echo 'export PATH=$PATH:/usr/local/go/bin' | sudo tee -a /etc/profile.d/go.sh
    export PATH=$PATH:/usr/local/go/bin
    rm -f go.tar.gz
    echo "Go installed successfully"
fi

echo ""
echo "System dependencies installation completed!"

# ============================================================================
# STEP 2: INSTALL RUST
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🦀 STEP 2/6: Installing Rust"
echo -e "==================================${NC}"

echo "Installing Rust programming language..."

# Check if Rust is already installed
if command -v rustc &> /dev/null; then
    echo "Rust is already installed: $(rustc --version)"
    echo "Updating Rust to latest version..."
    rustup update
else
    echo "Downloading and installing Rust..."
    
    # Download and run rustup installer
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        echo "Rust installed successfully!"
        
        # Source Rust environment
        source ~/.cargo/env
        
        # Verify installation
        if command -v rustc &> /dev/null; then
            echo "Rust verified: $(rustc --version)"
        else
            echo "Error: Rust installation verification failed"
            exit 1
        fi
    else
        echo "Error: Failed to install Rust"
        exit 1
    fi
fi

# Add Rust to PATH permanently
if ! grep -q 'source ~/.cargo/env' ~/.bashrc; then
    echo 'source ~/.cargo/env' >> ~/.bashrc
fi

if ! grep -q 'source ~/.cargo/env' ~/.profile; then
    echo 'source ~/.cargo/env' >> ~/.profile
fi

# Ensure Rust is available in current session
source ~/.cargo/env

echo "Rust installation completed!"

# ============================================================================
# STEP 3: DOWNLOAD AND BUILD TRADING BOT
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "📥 STEP 3/6: Downloading and building trading bot"
echo -e "==================================${NC}"

echo "Downloading trading bot source code..."

# Create project directory
if [ -d "$PROJECT_DIR" ]; then
    echo "Project directory already exists. Removing old version..."
    rm -rf "$PROJECT_DIR"
fi

# Clone repository
echo "Cloning repository from GitHub..."
if git clone https://github.com/KingGekko/trading-bot.git; then
    echo "Repository cloned successfully!"
else
    echo "Error: Failed to clone repository"
    exit 1
fi

# Enter project directory
cd "$PROJECT_DIR"

echo "Building trading bot..."
if cargo build --release; then
    echo "Trading bot built successfully!"
else
    echo "Error: Failed to build trading bot"
    echo ""
    echo "Common causes and solutions:"
    echo "1. Missing OpenSSL development files: sudo apt install -y libssl-dev"
    echo "2. Rust not in PATH: source ~/.cargo/env"
    echo "3. Outdated packages: sudo apt update && sudo apt upgrade"
    exit 1
fi

echo "Trading bot build completed!"

# ============================================================================
# STEP 4: INSTALL AND CONFIGURE OLLAMA
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🤖 STEP 4/6: Installing and configuring Ollama"
echo -e "==================================${NC}"

echo "Installing Ollama AI..."

# Check if Ollama is already installed
if command -v ollama &> /dev/null; then
    echo "Ollama is already installed: $(ollama --version)"
else
    echo "Downloading and installing Ollama..."
    
    # Download and install Ollama
    if curl -fsSL https://ollama.ai/install.sh | sh; then
        echo "Ollama installed successfully!"
        
        # Start Ollama service
        echo "Starting Ollama service..."
        sudo systemctl start ollama || sudo ollama serve &
        
        # Wait for service to start
        sleep 5
        
        # Verify installation
        if command -v ollama &> /dev/null; then
            echo "Ollama verified: $(ollama --version)"
        else
            echo "Error: Ollama installation verification failed"
            exit 1
        fi
    else
        echo "Error: Failed to install Ollama"
        exit 1
    fi
fi

# Ensure Ollama service is running
if ! pgrep -x "ollama" > /dev/null; then
    echo "Starting Ollama service..."
    sudo systemctl start ollama || sudo ollama serve &
    sleep 5
fi

echo "Ollama installation completed!"

# ============================================================================
# STEP 5: DOWNLOAD AI MODELS AND TEST
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🧠 STEP 5/6: Downloading AI models and testing"
echo -e "==================================${NC}"

echo "Downloading AI models..."

# Download tinyllama (default model)
echo "Downloading tinyllama model (default, ~1.1GB)..."
if ollama pull tinyllama; then
    echo "tinyllama model downloaded successfully!"
else
    echo "Error: Failed to download tinyllama model"
    exit 1
fi

# Ask about additional models
echo ""
echo -e "${BLUE}🎯 Would you like to install additional models for different use cases? (y/n)${NC}"
echo "   • llama2 (6GB) - Best analysis quality, slower responses"
echo "   • phi3 (2.7GB) - Microsoft's efficient model, good analysis quality"
echo "   • gemma2:2b (1.5GB) - Google's optimized model, excellent analysis"
read -r model_response

if [[ "$model_response" =~ ^[Yy]$ ]]; then
    echo "Installing additional models..."
    echo -e "${YELLOW}⏳ This may take several minutes...${NC}"
    
    echo "Installing llama2 (best analysis quality)..."
    ollama pull llama2
    
    echo "Installing phi3 (Microsoft's efficient model)..."
    ollama pull phi3
    
    echo "Installing gemma2:2b (Google's optimized model)..."
    ollama pull gemma2:2b
    
    echo -e "${GREEN}✅ Additional models installed!${NC}"
fi

# Test the installation
echo ""
echo "Testing the complete installation..."

# Test Rust
echo "Testing Rust..."
if rustc --version &> /dev/null; then
    echo "✅ Rust: OK"
else
    echo "❌ Rust: FAILED"
    exit 1
fi

# Test trading bot
echo "Testing trading bot..."
if ./target/release/trading-bot --help &> /dev/null; then
    echo "✅ Trading bot: OK"
else
    echo "❌ Trading bot: FAILED"
    exit 1
fi

# Test Ollama
echo "Testing Ollama..."
if ollama --version &> /dev/null; then
    echo "✅ Ollama: OK"
else
    echo "❌ Ollama: FAILED"
    exit 1
fi

# Test model
echo "Testing AI model..."
if ollama list | grep -q tinyllama; then
    echo "✅ AI model: OK"
else
    echo "❌ AI model: FAILED"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All tests passed! Installation completed successfully!${NC}"
echo ""
echo -e "${CYAN}🚀 Your trading bot is ready to use!${NC}"
echo ""
echo "To start using the bot:"
echo "  cd $PROJECT_DIR"
echo "  ./target/release/trading-bot --help"
echo ""
echo "To run a quick test:"
echo "  ./target/release/trading-bot -t"
echo ""
echo "To start interactive mode:"
echo "  ./target/release/trading-bot -i"
echo ""
echo -e "${YELLOW}💡 Tip: The bot is configured to use tinyllama by default for fast responses${NC}"
echo ""
echo -e "${GREEN}✨ Happy trading!${NC}" 