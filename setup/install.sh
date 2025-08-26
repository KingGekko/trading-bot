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

# Function to install Git from pre-compiled binary - DEFINED FIRST
install_git_from_binary() {
    echo "Installing Git from pre-compiled binary..."
    
    if command -v curl &> /dev/null; then
        echo "Downloading Git binary for Linux..."
        
        # Create temporary directory for Git installation
        mkdir -p /tmp/git_install
        cd /tmp/git_install
        
        # Detect architecture
        ARCH=$(uname -m)
        if [ "$ARCH" = "x86_64" ]; then
            GIT_ARCH="amd64"
        elif [ "$ARCH" = "aarch64" ]; then
            GIT_ARCH="arm64"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        
        echo "Detected architecture: $ARCH ($GIT_ARCH)"
        
        # Try to install via package manager first (most reliable)
        echo "Attempting package manager installation first..."
        if command -v yum &> /dev/null; then
            echo "Trying yum installation..."
            if sudo yum install -y git; then
                if command -v git &> /dev/null; then
                    echo "Git installed successfully via yum"
                    cd ~
                    rm -rf /tmp/git_install
                    return 0
                fi
            fi
        elif command -v dnf &> /dev/null; then
            echo "Trying dnf installation..."
            if sudo dnf install -y git; then
                if command -v git &> /dev/null; then
                    echo "Git installed successfully via dnf"
                    cd ~
                    rm -rf /tmp/git_install
                    return 0
                fi
            fi
        fi
        
        echo "Package manager installation failed, trying binary download..."
        
        # Try downloading Git binary with better error handling
        echo "Downloading Git binary for $ARCH..."
        
        # Method 1: Try static binary from GitHub (most reliable)
        if curl -L -H "Accept: application/octet-stream" -o git.tar.gz "https://github.com/git/git/releases/download/v2.44.0/git-2.44.0-linux-$GIT_ARCH.tar.gz"; then
            # Verify file size and type
            FILE_SIZE=$(stat -c%s git.tar.gz 2>/dev/null || stat -f%z git.tar.gz 2>/dev/null || echo "0")
            if [ "$FILE_SIZE" -gt 1000000 ]; then  # Should be > 1MB
                if file git.tar.gz 2>/dev/null | grep -q "gzip compressed data" || file git.tar.gz 2>/dev/null | grep -q "tar archive"; then
                    echo "Git binary downloaded successfully from GitHub (size: ${FILE_SIZE} bytes)"
                else
                    echo "Downloaded file is not a valid archive, trying alternative..."
                    rm -f git.tar.gz
                    # Fall through to next method
                fi
            else
                echo "Downloaded file too small (${FILE_SIZE} bytes), trying alternative..."
                rm -f git.tar.gz
                # Fall through to next method
            fi
        else
            echo "GitHub download failed, trying alternative..."
        fi
        
        # Method 2: Try kernel.org mirror
        if [ ! -f git.tar.gz ]; then
            echo "Trying kernel.org mirror..."
            if curl -L -o git.tar.gz "https://mirrors.edge.kernel.org/pub/software/scm/git/git-2.44.0-linux-$GIT_ARCH.tar.gz"; then
                FILE_SIZE=$(stat -c%s git.tar.gz 2>/dev/null || stat -f%z git.tar.gz 2>/dev/null || echo "0")
                if [ "$FILE_SIZE" -gt 1000000 ]; then
                    if file git.tar.gz 2>/dev/null | grep -q "gzip compressed data" || file git.tar.gz 2>/dev/null | grep -q "tar archive"; then
                        echo "Git binary downloaded successfully from kernel.org (size: ${FILE_SIZE} bytes)"
                    else
                        echo "Kernel.org file invalid, trying alternative..."
                        rm -f git.tar.gz
                    fi
                else
                    echo "Kernel.org file too small, trying alternative..."
                    rm -f git.tar.gz
                fi
            fi
        fi
        
        # Method 3: Try alternative GitHub URL format
        if [ ! -f git.tar.gz ]; then
            echo "Trying alternative GitHub format..."
            if curl -L -H "Accept: application/octet-stream" -o git.tar.gz "https://github.com/git/git/releases/download/v2.44.0/git-2.44.0-linux-$GIT_ARCH.tar.gz"; then
                FILE_SIZE=$(stat -c%s git.tar.gz 2>/dev/null || stat -f%z git.tar.gz 2>/dev/null || echo "0")
                if [ "$FILE_SIZE" -gt 1000000 ]; then
                    if file git.tar.gz 2>/dev/null | grep -q "gzip compressed data" || file git.tar.gz 2>/dev/null | grep -q "tar archive"; then
                        echo "Git binary downloaded successfully from alternative GitHub (size: ${FILE_SIZE} bytes)"
                    else
                        echo "Alternative GitHub file invalid"
                        rm -f git.tar.gz
                    fi
                else
                    echo "Alternative GitHub file too small"
                    rm -f git.tar.gz
                fi
            fi
        fi
        
        # If all downloads failed, show error and exit
        if [ ! -f git.tar.gz ] || [ ! -s git.tar.gz ]; then
            echo "Error: All Git download methods failed"
            echo "Please install Git manually:"
            echo "  sudo yum install -y git"
            echo "  or"
            echo "  sudo dnf install -y git"
            exit 1
        fi
        
        # Extract the binary
        echo "Extracting Git binary..."
        if ! tar -xzf git.tar.gz; then
            echo "Error: Failed to extract Git binary"
            echo "File contents (first 10 lines):"
            head -10 git.tar.gz
            echo "File type:"
            file git.tar.gz
            exit 1
        fi
        
        # Find the extracted directory
        GIT_DIR=$(find . -maxdepth 1 -type d -name "git-*" | head -1)
        if [ -z "$GIT_DIR" ]; then
            echo "Error: Could not find extracted Git directory"
            echo "Contents of current directory:"
            ls -la
            exit 1
        fi
        
        echo "Found Git directory: $GIT_DIR"
        
        # Install Git binary
        echo "Installing Git binary..."
        sudo cp -r "$GIT_DIR"/* /usr/local/
        
        # Create symlinks
        sudo ln -sf /usr/local/bin/git /usr/bin/git
        
        # Clean up
        cd ~
        rm -rf /tmp/git_install
        
        echo "Git binary installed successfully!"
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
echo "  1. Git (pre-compiled binary)"
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

# Install Git from pre-compiled binary (no package managers needed)
echo "Installing Git from pre-compiled binary (no build required)..."
install_git_from_binary

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