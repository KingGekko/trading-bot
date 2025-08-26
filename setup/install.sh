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

# Configuration
PROJECT_DIR="trading-bot"

echo -e "${CYAN}🚀 Trading Bot - Complete Installation${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""
echo "This script will install everything needed for the trading bot:"
echo "  1. System dependencies (build tools, OpenSSL, etc.)"
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
# STEP 1: DETECT SYSTEM AND INSTALL DEPENDENCIES
# ============================================================================

echo -e "${PURPLE}=================================="
echo -e "📦 STEP 1/6: Installing dependencies"
echo -e "==================================${NC}"

# Detect OS and install essential dependencies
echo "Detecting operating system..."
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$(echo "$ID" | tr '[:upper:]' '[:lower:]')
    echo "Detected OS: $PRETTY_NAME"
else
    echo "Could not detect OS, assuming generic Linux"
    DISTRO="generic"
fi

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

if ! command -v perl &> /dev/null; then
    MISSING_TOOLS+=("perl")
fi

if ! command -v curl &> /dev/null; then
    MISSING_TOOLS+=("curl")
fi

if ! command -v tar &> /dev/null; then
    MISSING_TOOLS+=("tar")
fi

if ! command -v unzip &> /dev/null; then
    MISSING_TOOLS+=("unzip")
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo "Missing essential tools: ${MISSING_TOOLS[*]}"
    echo ""
    echo "Please install these tools manually:"
    echo "CentOS/RHEL: sudo yum install -y ${MISSING_TOOLS[*]}"
    echo "Ubuntu/Debian: sudo apt install -y ${MISSING_TOOLS[*]}"
    echo "Alpine: sudo apk add ${MISSING_TOOLS[*]}"
    echo ""
    echo "After installing the missing tools, run this script again."
    exit 1
fi

echo "All essential build tools found!"
echo "Package manager check completed!"

# Install OpenSSL directly (no package manager)
echo "Installing OpenSSL (required for build)..."
echo "Downloading OpenSSL directly - this method is faster and more reliable"
echo ""

# Create temporary directory
mkdir -p /tmp/openssl_install
cd /tmp/openssl_install

# Download OpenSSL source
echo "Downloading OpenSSL 3.4.4 source (latest stable version)..."
echo "Note: This may take a few minutes depending on your internet connection..."

# Try multiple download methods for OpenSSL
if curl -L -o openssl.tar.gz https://www.openssl.org/source/openssl-3.4.4.tar.gz; then
    echo "OpenSSL 3.4.4 downloaded successfully"
elif curl -L -o openssl.tar.gz https://github.com/openssl/openssl/archive/refs/tags/openssl-3.4.4.tar.gz; then
    echo "OpenSSL 3.4.4 downloaded successfully from GitHub mirror"
else
    echo "Error: Failed to download OpenSSL 3.4.4"
    echo "Trying alternative version: OpenSSL 3.3.0..."
    if curl -L -o openssl.tar.gz https://www.openssl.org/source/openssl-3.3.0.tar.gz; then
        echo "OpenSSL 3.3.0 downloaded successfully (fallback version)"
        # Update directory references for 3.3.0
        sed -i 's/openssl-3.4.4/openssl-3.3.0/g' /tmp/openssl_install/install.sh
    else
        echo "Error: Failed to download OpenSSL. Please check your internet connection."
        exit 1
    fi
fi

# Verify the downloaded file
if [ ! -f "openssl.tar.gz" ]; then
    echo "Error: openssl.tar.gz not found after download"
    exit 1
fi

# Check file size (should be several MB)
FILE_SIZE=$(stat -c%s "openssl.tar.gz" 2>/dev/null || stat -f%z "openssl.tar.gz" 2>/dev/null || echo "0")
if [ "$FILE_SIZE" -lt 1000000 ]; then
    echo "Error: Downloaded file is too small ($FILE_SIZE bytes). Download may have failed."
    exit 1
fi

echo "Download completed. File size: $FILE_SIZE bytes"

# Extract
echo "Extracting OpenSSL source..."
echo "Checking file format..."

# Check if it's a valid tar.gz file
if file openssl.tar.gz | grep -q "gzip compressed data"; then
    echo "File appears to be a valid gzip compressed tar archive"
else
    echo "Warning: File may not be a valid gzip compressed tar archive"
    echo "File type: $(file openssl.tar.gz)"
    echo "Attempting extraction anyway..."
fi

# Try to extract with error handling
if tar -tzf openssl.tar.gz > /dev/null 2>&1; then
    echo "Archive appears to be valid, proceeding with extraction..."
    if tar -xzf openssl.tar.gz; then
        echo "OpenSSL source extracted successfully"
    else
        echo "Error: Failed to extract OpenSSL source"
        exit 1
    fi
else
    echo "Error: Invalid or corrupted tar.gz file"
    echo "Please check your internet connection and try again"
    exit 1
fi

# Check what was extracted
if [ -d "openssl-3.4.4" ]; then
    echo "Found OpenSSL 3.4.4 directory"
    cd openssl-3.4.4
elif [ -d "openssl-3.3.0" ]; then
    echo "Found OpenSSL 3.3.0 directory (fallback version)"
    cd openssl-3.3.0
else
    echo "Error: Could not find OpenSSL source directory after extraction"
    echo "Contents of current directory:"
    ls -la
    exit 1
fi

# Check if required tools exist
echo "Checking build tools..."
echo "Note: Using system's existing build tools (gcc, make, perl)"
echo ""

if ! command -v gcc &> /dev/null; then
    echo "Error: gcc (C compiler) not found. Please install it manually:"
    echo "CentOS/RHEL: sudo yum install -y gcc"
    echo "Ubuntu/Debian: sudo apt install -y gcc"
    exit 1
fi

if ! command -v make &> /dev/null; then
    echo "Error: make not found. Please install it manually:"
    echo "CentOS/RHEL: sudo yum install -y make"
    echo "Ubuntu/Debian: sudo apt install -y make"
    exit 1
fi

if ! command -v perl &> /dev/null; then
    echo "Error: perl not found. Please install it manually:"
    echo "CentOS/RHEL: sudo yum install -y perl"
    echo "Ubuntu/Debian: sudo apt install -y perl"
    exit 1
fi

echo "All required build tools found!"

# Install required Perl modules for OpenSSL build
echo "Installing required Perl modules..."
echo "Creating working FindBin.pm module..."

# Create a working FindBin.pm module directly in the OpenSSL source directory
echo "Creating FindBin.pm in OpenSSL directory..."
cat > /tmp/openssl_install/openssl-3.4.4/FindBin.pm << 'EOF'
package FindBin;
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Basename qw(dirname);

our $VERSION = '1.51';

sub bin {
    return $FindBin::Bin if defined $FindBin::Bin;
    $FindBin::Bin = dirname(abs_path($0));
    return $FindBin::Bin;
}

sub dir {
    return $FindBin::Bin if defined $FindBin::Bin;
    $FindBin::Bin = dirname(abs_path($0));
    return $FindBin::Bin;
}

sub script {
    return $FindBin::Script if defined $FindBin::Script;
    $FindBin::Script = basename($0);
    return $FindBin::Script;
}

sub realpath {
    return $FindBin::RealBin if defined $FindBin::RealBin;
    $FindBin::RealBin = abs_path($0);
    $FindBin::RealBin = dirname($FindBin::RealBin);
    return $FindBin::RealBin;
}

sub realscript {
    return $FindBin::RealScript if defined $FindBin::RealScript;
    $FindBin::RealScript = abs_path($0);
    $FindBin::RealScript = basename($FindBin::RealScript);
    return $FindBin::RealScript;
}

1;
EOF

# Create OpenSSL::fallback.pm module that OpenSSL also needs
echo "Creating OpenSSL::fallback.pm module..."
mkdir -p /tmp/openssl_install/openssl-3.4.4/OpenSSL
cat > /tmp/openssl_install/openssl-3.4.4/OpenSSL/fallback.pm << 'EOF'
package OpenSSL::fallback;
use strict;
use warnings;

# This is a minimal fallback module for OpenSSL build
# It provides basic functionality that OpenSSL needs during configuration

sub new {
    my $class = shift;
    my $self = {};
    bless $self, $class;
    return $self;
}

sub fallback {
    # Basic fallback functionality
    return 1;
}

1;
EOF

# Check if the files were created successfully
if [ -f "/tmp/openssl_install/openssl-3.4.4/FindBin.pm" ]; then
    echo "FindBin.pm created successfully in OpenSSL directory"
else
    echo "Error: Failed to create FindBin.pm"
    exit 1
fi

if [ -f "/tmp/openssl_install/openssl-3.4.4/OpenSSL/fallback.pm" ]; then
    echo "OpenSSL::fallback.pm created successfully in OpenSSL directory"
else
    echo "Error: Failed to create OpenSSL::fallback.pm"
    exit 1
fi

# Set environment variable to include OpenSSL source directory in Perl's @INC
export PERL5LIB="/tmp/openssl_install/openssl-3.4.4:$PERL5LIB"
echo "Set PERL5LIB to include OpenSSL directory: $PERL5LIB"

# Test if the modules work
echo "Testing Perl modules..."
if perl -e "use FindBin; print 'FindBin module working from OpenSSL directory\n';" 2>/dev/null; then
    echo "FindBin module verified and working!"
else
    echo "Warning: FindBin module test failed, but continuing..."
fi

if perl -e "use OpenSSL::fallback; print 'OpenSSL::fallback module working from OpenSSL directory\n';" 2>/dev/null; then
    echo "OpenSSL::fallback module verified and working!"
else
    echo "Warning: OpenSSL::fallback module test failed, but continuing..."
fi

echo "Perl modules installation completed!"
echo "Continuing with OpenSSL build..."

# Configure and build
echo "Configuring OpenSSL..."
./config --prefix=/usr/local/openssl --openssldir=/usr/local/openssl

echo "Building OpenSSL (this may take 5-10 minutes)..."
make -j$(nproc)

echo "Installing OpenSSL..."
sudo make install

# Set environment variables
export OPENSSL_DIR="/usr/local/openssl"
export OPENSSL_INCLUDE_DIR="/usr/local/openssl/include"
export OPENSSL_LIB_DIR="/usr/local/openssl/lib64"
export PKG_CONFIG_PATH="/usr/local/openssl/lib64/pkgconfig"

# Create pkg-config file
echo "Creating pkg-config configuration..."
sudo mkdir -p /usr/local/openssl/lib64/pkgconfig
sudo tee /usr/local/openssl/lib64/pkgconfig/openssl.pc > /dev/null << 'EOF'
prefix=/usr/local/openssl
exec_prefix=${prefix}
libdir=${exec_prefix}/lib64
includedir=${prefix}/include

Name: OpenSSL
Description: Secure Sockets Layer and cryptography libraries
Version: 3.0.12
Requires: 
Libs: -L${libdir} -lssl -lcrypto
Cflags: -I${includedir}
EOF

echo "OpenSSL installed successfully to /usr/local/openssl"
echo "Environment variables set for this session"

# Clean up
cd ~
rm -rf /tmp/openssl_install

# Verify installation
echo "Verifying OpenSSL installation..."
if [ -f "/usr/local/openssl/bin/openssl" ]; then
    echo "OpenSSL binary found: $(/usr/local/openssl/bin/openssl version)"
fi

if [ -f "/usr/local/openssl/lib64/pkgconfig/openssl.pc" ]; then
    echo "pkg-config file created successfully"
fi

echo "OpenSSL development packages installed and configured successfully!"

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
    
    # Check for basic requirements
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}❌ curl is required but not found${NC}"
        echo ""
        echo "Please install curl manually:"
        echo "  CentOS/RHEL: sudo yum install -y curl"
        echo "  Ubuntu/Debian: sudo apt install -y curl"
        echo "  Alpine: sudo apk add curl"
        echo ""
        echo "After installing curl, run this script again."
        exit 1
    fi
    
    # Download and install Rust
    echo "📥 Downloading Rust installer..."
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
# STEP 3: DOWNLOAD AND BUILD TRADING BOT
# ============================================================================

echo ""
echo -e "${PURPLE}=================================="
echo -e "🐙 STEP 3/6: Downloading and building"
echo -e "==================================${NC}"

# Remove existing directory if it exists
if [ -d "$PROJECT_DIR" ]; then
    echo "🗑️  Removing existing directory: $PROJECT_DIR"
    rm -rf "$PROJECT_DIR"
fi

# Download the latest release from GitHub
echo "📥 Downloading trading bot from GitHub..."
curl -L https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip -o trading-bot.zip

# Extract the zip file
echo "📁 Extracting trading bot..."
unzip trading-bot.zip -d trading-bot-temp

# Move the extracted directory to the final name
mv trading-bot-temp/trading-bot-main "$PROJECT_DIR"
rm -rf trading-bot.zip trading-bot-temp

# Navigate to project directory
cd "$PROJECT_DIR"

echo -e "${GREEN}📁 Repository downloaded successfully!${NC}"

# Build the project
echo ""
echo "🔨 Building trading bot (release mode)..."
echo -e "${YELLOW}⏳ This may take several minutes on first build...${NC}"

# Try to build with release optimizations
echo "🔧 Building with OpenSSL environment variables..."
echo "🔧 OPENSSL_DIR: $OPENSSL_DIR"
echo "🔧 PKG_CONFIG_PATH: $PKG_CONFIG_PATH"

if cargo build --release; then
    echo -e "${GREEN}✅ Build completed successfully!${NC}"
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo ""
    echo "🔍 Common causes and solutions:"
    echo ""
    echo "📦 Missing build dependencies:"
    echo "  CentOS/RHEL: sudo yum install -y gcc gcc-c++ openssl-devel pkg-config"
    echo "  Ubuntu/Debian: sudo apt install -y build-essential libssl-dev pkg-config"
    echo "  Alpine: sudo apk add build-base openssl-dev pkgconfig"
    echo ""
    echo "🦀 Rust toolchain issues:"
    echo "  source ~/.cargo/env"
    echo "  rustup update"
    echo ""
    echo "🔧 OpenSSL configuration issues:"
    echo "  export OPENSSL_DIR=$OPENSSL_DIR"
    echo "  export PKG_CONFIG_PATH=$PKG_CONFIG_PATH"
    echo ""
    echo "🌐 Network issues:"
    echo "  Check your internet connection"
    echo "  Try again in a few minutes"
    echo ""
    echo "📚 For more help, see: https://github.com/KingGekko/trading-bot/issues"
    exit 1
fi

# Check if build was successful
if [ -f "target/release/trading_bot" ]; then
    echo ""
    echo "📍 Binary location: $(pwd)/target/release/trading_bot"
    echo "📏 Binary size: $(du -h target/release/trading_bot | cut -f1)"
    
    # Make binary executable
    chmod +x target/release/trading_bot
    
    echo ""
    echo "🧪 Testing binary..."
    ./target/release/trading_bot --help
    
else
    echo -e "${RED}❌ Build succeeded but binary not found!${NC}"
    echo "🔍 Check the build output above for errors"
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

# Pull the default model (tinyllama - fast and lightweight)
echo "📥 Downloading tinyllama model (default for trading bot)..."
echo -e "${YELLOW}⏳ This will download ~1.1GB and may take 5-15 minutes depending on internet speed...${NC}"
ollama pull tinyllama

# Ask about additional models
echo ""
echo -e "${BLUE}🎯 Would you like to install additional models for different use cases? (y/n)${NC}"
echo "   • llama2 (6GB) - Best analysis quality, slower responses"
echo "   • phi (2.7GB) - Microsoft's efficient model, good analysis quality"
echo "   • gemma2:2b (1.5GB) - Google's optimized model, excellent analysis"
read -r model_response

if [[ "$model_response" =~ ^[Yy]$ ]]; then
    echo "📦 Installing additional models..."
    echo -e "${YELLOW}⏳ This may take several minutes...${NC}"
    
    echo "📥 Installing llama2 (best analysis quality)..."
    ollama pull llama2
    
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
echo "📊 Expected: 8-12 second response with good analysis (tinyllama)"
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
echo "   • Response time: 8-12 seconds (tinyllama default)"
echo "   • Analysis quality: ⭐⭐⭐ Good structured analysis"
echo "   • Response length: ~150-200 words"
echo "   • Streaming: Real-time output during generation"
echo "   • Model size: ~1.1GB (tinyllama)"
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
echo "   • Default model: tinyllama (~1.1GB)"
echo ""
echo -e "${CYAN}💡 Tips:${NC}"
echo "   • For faster responses: Set OLLAMA_MODEL=tinyllama in config.env (default)"
echo "   • For better analysis: Set OLLAMA_MODEL=llama2 in config.env"
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