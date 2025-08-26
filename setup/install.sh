#!/bin/bash

# Trading Bot Installation Script for Ubuntu/Canonical
# This script installs all necessary dependencies including Python, pip, Rust, and Ollama

set -e  # Exit on any error

echo "🚀 Trading Bot Installation Script"
echo "=================================="
echo "This script will install:"
echo "  • Python 3 and pip"
echo "  • Rust programming language"
echo "  • Ollama AI framework"
echo "  • Trading Bot application"
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ This script should not be run as root"
   echo "   Please run as a regular user with sudo privileges"
   exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install package with apt
install_package() {
    local package=$1
    if ! dpkg -l | grep -q "^ii  $package "; then
        echo "📦 Installing $package..."
        sudo apt-get install -y "$package"
    else
        echo "✅ $package is already installed"
    fi
}

echo "🔍 Checking system requirements..."

# Update package list
echo "📋 Updating package list..."
sudo apt-get update

# Install essential packages
echo "📦 Installing essential packages..."
install_package "curl"
install_package "wget"
install_package "git"
install_package "build-essential"
install_package "pkg-config"
install_package "libssl-dev"
install_package "python3"
install_package "python3-pip"
install_package "python3-venv"

# Verify Python installation
echo "🐍 Verifying Python installation..."
if command_exists python3; then
    python3_version=$(python3 --version)
    echo "✅ $python3_version is installed"
else
    echo "❌ Python 3 installation failed"
    exit 1
fi

# Verify and upgrade pip
echo "📦 Verifying pip installation..."
if command_exists pip3; then
    pip3_version=$(pip3 --version)
    echo "✅ $pip3_version is installed"
    
    echo "🔄 Upgrading pip to latest version..."
    python3 -m pip install --upgrade pip --user
    
    # Verify pip upgrade
    new_pip_version=$(pip3 --version)
    echo "✅ Upgraded to $new_pip_version"
else
    echo "❌ pip3 installation failed"
    exit 1
fi

# Install Python development tools
echo "🔧 Installing Python development tools..."
install_package "python3-dev"
install_package "python3-setuptools"

# Install Rust
echo "🦀 Installing Rust programming language..."
if ! command_exists cargo; then
    echo "📥 Downloading Rust installer..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source Rust environment
    source "$HOME/.cargo/env"
    
    # Verify Rust installation
    if command_exists cargo; then
        rust_version=$(cargo --version)
        echo "✅ $rust_version is installed"
    else
        echo "❌ Rust installation failed"
        exit 1
    fi
else
    rust_version=$(cargo --version)
    echo "✅ $rust_version is already installed"
fi

# Update Rust to latest version
echo "🔄 Updating Rust to latest version..."
rustup update

# Install additional Rust components
echo "🔧 Installing additional Rust components..."
rustup component add rust-src
rustup component add rust-analysis
rustup component add rust-std

# Install useful Rust tools
echo "🛠️ Installing Rust development tools..."
cargo install cargo-update
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-tree
cargo install cargo-expand

# Verify Rust tools installation
echo "✅ Rust tools installed:"
cargo --version
rustc --version
rustup --version

# Install Ollama
echo "🤖 Installing Ollama AI framework..."
if ! command_exists ollama; then
    echo "📥 Downloading Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
    
    # Start Ollama service
    echo "🚀 Starting Ollama service..."
    ollama serve &
    
    # Wait for Ollama to start
    echo "⏳ Waiting for Ollama to start..."
    sleep 5
    
    # Verify Ollama installation
    if command_exists ollama; then
        ollama_version=$(ollama --version)
        echo "✅ $ollama_version is installed and running"
    else
        echo "❌ Ollama installation failed"
        exit 1
    fi
else
    ollama_version=$(ollama --version)
    echo "✅ $ollama_version is already installed"
    
    # Ensure Ollama is running
    if ! pgrep -x "ollama" > /dev/null; then
        echo "🚀 Starting Ollama service..."
        ollama serve &
        sleep 5
    fi
fi

# Install protobuf compiler
echo "📋 Installing Protocol Buffers compiler..."
install_package "protobuf-compiler"

# Verify protobuf installation
if command_exists protoc; then
    protoc_version=$(protoc --version)
    echo "✅ $protoc_version is installed"
else
    echo "❌ protobuf-compiler installation failed"
    exit 1
fi

# Install additional Python packages for development
echo "🐍 Installing Python development packages..."
python3 -m pip install --user protobuf grpcio-tools

# Clone or update trading bot repository
echo "📁 Setting up trading bot repository..."
if [ -d "trading_bot" ]; then
    echo "🔄 Updating existing repository..."
    cd trading_bot
    git pull origin main
else
    echo "📥 Cloning repository..."
    git clone https://github.com/yourusername/trading_bot.git
    cd trading_bot
fi

# Build the trading bot
echo "🔨 Building trading bot..."
cargo build --release

# Create log directories
echo "📁 Creating log directories..."
mkdir -p logs
mkdir -p ollama_logs

# Set proper permissions
echo "🔐 Setting permissions..."
chmod 755 logs
chmod 755 ollama_logs

# Create configuration file if it doesn't exist
if [ ! -f "config.env" ]; then
    echo "⚙️ Creating configuration file..."
    cat > config.env << EOF
# Trading Bot Configuration
BOT_NAME=TradingBot
LOG_LEVEL=info
LOG_TO_FILE=true
LOG_TO_CONSOLE=true
LOG_DIRECTORY=./logs

# Ollama Configuration
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=auto

# Performance Settings
MAX_TIMEOUT_SECONDS=300
MAX_RESPONSE_LENGTH=500
EOF
    echo "✅ Configuration file created: config.env"
fi

# Test the installation
echo "🧪 Testing installation..."
if [ -f "target/release/trading_bot" ]; then
    echo "✅ Trading bot binary built successfully"
    
    # Test Ollama connection
    echo "🔗 Testing Ollama connection..."
    if curl -s http://localhost:11434/api/tags > /dev/null; then
        echo "✅ Ollama is accessible"
    else
        echo "⚠️  Ollama might not be running. You can start it with: ollama serve"
    fi
else
    echo "❌ Trading bot build failed"
    exit 1
fi

echo ""
echo "🎉 Installation completed successfully!"
echo ""
echo "📋 What was installed:"
echo "  ✅ Python 3 and latest pip"
echo "  ✅ Rust programming language"
echo "  ✅ Ollama AI framework"
echo "  ✅ Protocol Buffers compiler"
echo "  ✅ Trading Bot application"
echo "  ✅ All necessary dependencies"
echo ""
echo "🚀 To start using the trading bot:"
echo "  1. Start Ollama: ollama serve"
echo "  2. Run the bot: ./target/release/trading_bot --help"
echo "  3. Test with: ./target/release/trading_bot -b 'Hello, world!'"
echo ""
echo "📚 For more information, see the README.md file"
echo "🔄 To update later, run: ./setup/update.sh" 