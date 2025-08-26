#!/bin/bash
# Trading Bot - Complete Installation Script
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

# Function to install Git from source - DEFINED FIRST
install_git_from_source() {
    echo "Installing Git from source..."
    
    if command -v curl &> /dev/null; then
        echo "Downloading Git source directly..."
        
        # Create temporary directory for Git build
        mkdir -p /tmp/git_build
        cd /tmp/git_build
        
        # Try multiple Git download sources with better error handling
        echo "Attempting to download Git source from GitHub..."
        
        # Method 1: Try direct GitHub release with proper headers
        if curl -L -H "Accept: application/octet-stream" -o git.tar.gz https://github.com/git/git/releases/download/v2.44.0/git-2.44.0.tar.gz; then
            # Verify the downloaded file is actually a tarball
            if file git.tar.gz | grep -q "gzip compressed data"; then
                echo "Git source downloaded successfully from GitHub"
            else
                echo "Downloaded file is not a valid tarball, trying alternative source..."
                rm -f git.tar.gz
                
                # Method 2: Try kernel.org mirror
                if curl -L -o git.tar.gz https://mirrors.edge.kernel.org/pub/software/scm/git/git-2.44.0.tar.gz; then
                    if file git.tar.gz | grep -q "gzip compressed data"; then
                        echo "Git source downloaded successfully from kernel.org"
                    else
                        echo "Kernel.org download also failed, trying direct source..."
                        rm -f git.tar.gz
                        
                        # Method 3: Try direct source with verbose output
                        if curl -v -L -o git.tar.gz https://github.com/git/git/archive/refs/tags/v2.44.0.tar.gz; then
                            if file git.tar.gz | grep -q "gzip compressed data"; then
                                echo "Git source downloaded successfully from GitHub archive"
                            else
                                echo "All download methods failed. File contents:"
                                head -5 git.tar.gz
                                echo "Error: All Git download methods failed"
                                exit 1
                            fi
                        else
                            echo "Error: All Git download methods failed"
                            exit 1
                        fi
                    fi
                else
                    echo "Error: Failed to download Git from kernel.org"
                    exit 1
                fi
            fi
        else
            echo "Error: Failed to download Git from GitHub"
            exit 1
        fi
        
        # Extract and build
        echo "Extracting Git source..."
        tar -xzf git.tar.gz
        
        # Handle different directory names from different sources
        if [ -d "git-2.44.0" ]; then
            cd git-2.44.0
        elif [ -d "git-v2.44.0" ]; then
            cd git-v2.44.0
        else
            echo "Error: Unexpected directory structure after extraction"
            ls -la
            exit 1
        fi
        
        echo "Building Git from source..."
        make prefix=/usr/local all
        
        echo "Installing Git..."
        sudo make prefix=/usr/local install
        
        # Clean up
        cd ~
        rm -rf /tmp/git_build
        
        echo "Git built and installed from source successfully!"
    else
        echo "Error: curl not available. Please install Git manually:"
        echo "sudo yum install -y git"
        exit 1
    fi
}

# Configuration
PROJECT_DIR="trading-bot"

echo -e "${CYAN}🚀 Trading Bot - Complete Installation${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""
echo "This script will install everything needed for the trading bot:"
echo "  1. Git (from source)"
echo "  2. Rust programming language"
echo "  3. Download and build the trading bot from GitHub"
echo "  4. Install and configure Ollama AI"
echo "  5. Download AI models (tinyllama + optional extras)"
echo "  6. Test the complete installation"
echo ""
echo -e "${YELLOW}⏳ Estimated time: 8-18 minutes (depending on internet speed)${NC}"
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
# STEP 1: DETECT SYSTEM AND INSTALL GIT
# ============================================================================

echo -e "${PURPLE}=================================="
echo -e "📦 STEP 1/6: Installing Git"
echo -e "==================================${NC}"

# Detect OS
echo "Detecting operating system..."
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$(echo "$ID" | tr '[:upper:]' '[:lower:]')
    echo "Detected OS: $PRETTY_NAME"
else
    echo "Could not detect OS, assuming generic Linux"
    DISTRO="generic"
fi

echo "Installing Git (required for downloading source code)..."
echo "Note: This script will install Git first, then clone the repository"

# Clean up any hanging processes first
echo "Cleaning up any hanging processes..."
sudo pkill -9 -f yum 2>/dev/null || true
sudo pkill -9 -f dnf 2>/dev/null || true
sudo pkill -9 -f git 2>/dev/null || true
sudo pkill -9 -f apt 2>/dev/null || true

# Wait for processes to clean up
sleep 2

# Install Git from source (no package managers needed)
echo "Building Git from source (no package managers required)..."
install_git_from_source

# Verify Git installation
if command -v git &> /dev/null; then
    echo "Git installed successfully: $(git --version)"
else
    echo "Error: Git installation failed"
    exit 1
fi

echo "Git installation completed!"

# ============================================================================
# STEP 2: CHECK BUILD TOOLS
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🔧 STEP 2/6: Checking build tools"
echo -e "==================================${NC}"

echo "Checking for essential build tools..."
echo "Note: This script will use existing system tools without package managers"
echo ""

# Check if required tools exist
MISSING_TOOLS=()

if ! command -v gcc &> /dev/null; then
    MISSING_TOOLS+=("gcc")
fi

if ! command -v make &> /dev/null; then
    MISSING_TOOLS+=("make")
fi

if ! command -v curl &> /dev/null; then
    MISSING_TOOLS+=("curl")
fi

if ! command -v tar &> /dev/null; then
    MISSING_TOOLS+=("tar")
fi

if ! command -v perl &> /dev/null; then
    MISSING_TOOLS+=("perl")
fi

# Report missing tools
if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo -e "${GREEN}✅ All essential build tools found!${NC}"
else
    echo -e "${RED}❌ Missing essential build tools: ${MISSING_TOOLS[*]}${NC}"
    echo ""
    echo "Please install these tools manually:"
    echo "  - gcc: C compiler"
    echo "  - make: Build automation tool"
    echo "  - curl: File download utility"
    echo "  - tar: Archive utility"
    echo "  - perl: Perl interpreter"
    echo ""
    echo "On Oracle Linux/RHEL/CentOS:"
    echo "  sudo yum groupinstall -y 'Development Tools'"
    echo "  sudo yum install -y curl perl"
    echo ""
    echo "On Ubuntu/Debian:"
    echo "  sudo apt-get update"
    echo "  sudo apt-get install -y build-essential curl perl"
    echo ""
    exit 1
fi

echo "Build tools check completed!"

# ============================================================================
# STEP 3: INSTALL RUST
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🦀 STEP 3/6: Installing Rust"
echo -e "==================================${NC}"

echo "Installing Rust programming language..."

# Check if Rust is already installed
if command -v rustc &> /dev/null; then
    echo "Rust is already installed: $(rustc --version)"
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

echo "Rust installation completed!"

# ============================================================================
# STEP 4: DOWNLOAD AND BUILD TRADING BOT
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "📥 STEP 4/6: Downloading and building trading bot"
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
    exit 1
fi

echo "Trading bot build completed!"

# ============================================================================
# STEP 5: INSTALL AND CONFIGURE OLLAMA
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🤖 STEP 5/6: Installing and configuring Ollama"
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

echo "Ollama installation completed!"

# ============================================================================
# STEP 6: DOWNLOAD AI MODELS AND TEST
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🧠 STEP 6/6: Downloading AI models and testing"
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