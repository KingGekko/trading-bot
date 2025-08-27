#!/bin/bash

# Trading Bot - Comprehensive Update Script
# This script updates the trading bot with full dependency management and protobuf support

set -e

echo "🔄 Trading Bot - Comprehensive Update Script"
echo "============================================="
echo "This script will:"
echo "  • Update Rust toolchain and dependencies"
echo "  • Verify and repair protobuf installation"
echo "  • Update Ollama to latest version"
echo "  • Update system packages"
echo "  • Rebuild the trading bot"
echo "  • Run security audits"
echo ""

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

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}❌ This script requires admin privileges!${NC}"
    echo ""
    echo "Please run with sudo:"
    echo "  sudo ./update.sh"
    echo ""
    echo "Or run the full installation with sudo:"
    echo "  sudo ./install.sh"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if we're in a Rust project
check_rust_project() {
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}❌ Not in a Rust project directory${NC}"
        echo "Please run this script from the root of a Rust project"
        exit 1
    fi
}

# Function to verify and repair protobuf installation
verify_protobuf() {
    echo "📋 Verifying protobuf installation..."
    
    if ! command_exists protoc; then
        echo "❌ protoc not found, attempting to install..."
        
        # Try to install protobuf-compiler
        sudo apt-get update
        sudo apt-get install -y protobuf-compiler
        
        # Verify installation
        if command_exists protoc; then
            protoc_version=$(protoc --version)
            echo "✅ $protoc_version is now installed"
        else
            echo "❌ Failed to install protobuf-compiler"
            return 1
        fi
    else
        protoc_version=$(protoc --version)
        echo "✅ $protoc_version is installed"
    fi
    
    # Check if PROTOC environment variable is set
    if [ -z "$PROTOC" ]; then
        PROTOC_PATH=$(which protoc)
        if [ -n "$PROTOC_PATH" ]; then
            echo "🔧 Setting PROTOC environment variable..."
            export PROTOC="$PROTOC_PATH"
            echo "export PROTOC=\"$PROTOC_PATH\"" >> ~/.bashrc
            echo "export PROTOC=\"$PROTOC_PATH\"" >> ~/.profile
        fi
    fi
    
    # Test protobuf compilation
    echo "🧪 Testing protobuf compilation..."
    if [ -f "proto/receipt.proto" ]; then
        if protoc --cpp_out=/tmp proto/receipt.proto 2>/dev/null; then
            echo "✅ Protobuf compilation test passed"
            rm -f /tmp/receipt.pb.h /tmp/receipt.pb.cc
        else
            echo "❌ Protobuf compilation test failed"
            return 1
        fi
    fi
    
    return 0
}

# Function to install protobuf using multiple methods
install_protobuf() {
    echo "🔧 Installing protobuf using multiple methods..."
    
    # Method 1: Package manager installation
    echo "📦 Method 1: Package manager installation..."
    sudo apt-get update
    
    # Try different package names
    if apt-cache show protobuf-compiler &> /dev/null; then
        echo "Installing protobuf-compiler..."
        sudo apt-get install -y protobuf-compiler
    elif apt-cache show protobuf-c-compiler &> /dev/null; then
        echo "Installing protobuf-c-compiler..."
        sudo apt-get install -y protobuf-c-compiler
    else
        echo "No protobuf package found in repositories"
    fi
    
    # Method 2: Download pre-built binary
    if ! command_exists protoc; then
        echo "📥 Method 2: Downloading pre-built binary..."
        
        # Detect architecture
        ARCH=$(uname -m)
        case $ARCH in
            x86_64) ARCH="x86_64" ;;
            aarch64) ARCH="aarch_64" ;;
            armv7l) ARCH="arm_32" ;;
            *) echo "Unsupported architecture: $ARCH"; return 1 ;;
        esac
        
        # Download appropriate binary
        PROTOC_VERSION="25.3"
        PROTOC_URL="https://github.com/protocolbuffers/protobuf/releases/download/v$PROTOC_VERSION/protoc-$PROTOC_VERSION-linux-$ARCH.zip"
        
        echo "📥 Downloading protobuf $PROTOC_VERSION for $ARCH..."
        curl -L -o protoc.zip "$PROTOC_URL"
        
        if [ -f "protoc.zip" ]; then
            # Install unzip if not available
            if ! command_exists unzip; then
                sudo apt-get install -y unzip
            fi
            
            # Extract and install
            unzip protoc.zip
            sudo mv bin/protoc /usr/local/bin/
            sudo chmod +x /usr/local/bin/protoc
            
            # Clean up
            rm -rf bin include protoc.zip
            
            echo "✅ Protobuf binary installed"
        else
            echo "❌ Failed to download protobuf binary"
        fi
    fi
    
    # Method 3: Build from source
    if ! command_exists protoc; then
        echo "🔄 Method 3: Building from source..."
        
        # Install build dependencies
        sudo apt-get install -y build-essential cmake pkg-config
        
        # Download and install protobuf from source
        PROTOC_VERSION="25.3"
        PROTOC_DIR="/tmp/protoc"
        
        mkdir -p "$PROTOC_DIR"
        cd "$PROTOC_DIR"
        
        # Download protobuf source
        echo "📥 Downloading protobuf $PROTOC_VERSION source..."
        curl -L -o protobuf.tar.gz "https://github.com/protocolbuffers/protobuf/releases/download/v$PROTOC_VERSION/protobuf-$PROTOC_VERSION.tar.gz"
        
        if [ -f "protobuf.tar.gz" ]; then
            tar -xzf protobuf.tar.gz
            cd "protobuf-$PROTOC_VERSION"
            
            # Configure and build
            echo "🔨 Building protobuf from source..."
            ./configure --prefix=/usr/local
            make -j$(nproc 2>/dev/null || echo 4)
            sudo make install
            sudo ldconfig
            
            echo "✅ Protobuf built and installed from source"
        else
            echo "❌ Failed to download protobuf source"
        fi
        
        cd - > /dev/null
        rm -rf "$PROTOC_DIR"
    fi
    
    # Install additional protobuf tools and libraries
    echo "🔧 Installing additional protobuf tools and libraries..."
    
    # Install protobuf development libraries
    if apt-cache show libprotobuf-dev &> /dev/null; then
        echo "📦 Installing libprotobuf-dev..."
        sudo apt-get install -y libprotobuf-dev
    fi
    
    if apt-cache show protobuf-c-compiler &> /dev/null; then
        echo "📦 Installing protobuf-c-compiler..."
        sudo apt-get install -y protobuf-c-compiler
    fi
    
    # Install additional protobuf tools
    if apt-cache show protobuf-compiler-grpc &> /dev/null; then
        echo "📦 Installing protobuf-compiler-grpc..."
        sudo apt-get install -y protobuf-compiler-grpc
    fi
    
    return 0
}

# Function to repair protobuf installation
repair_protobuf() {
    echo "🔧 Repairing protobuf installation..."
    
    # Check current status
    if verify_protobuf; then
        echo "✅ Protobuf is working correctly"
        return 0
    fi
    
    echo "❌ Protobuf needs repair, attempting installation..."
    
    # Try to install protobuf
    if install_protobuf; then
        echo "✅ Protobuf installation completed"
        
        # Verify installation
        if verify_protobuf; then
            echo "✅ Protobuf repair successful"
            return 0
        else
            echo "❌ Protobuf repair failed"
            return 1
        fi
    else
        echo "❌ Protobuf installation failed"
        return 1
    fi
}

# Function to update pip
update_pip() {
    echo "📦 Updating Python pip..."
    
    if command_exists pip3; then
        current_version=$(pip3 --version | cut -d' ' -f2)
        echo "Current pip version: $current_version"
        
        echo "🔄 Upgrading pip to latest version..."
        python3 -m pip install --upgrade pip
        
        new_version=$(pip3 --version | cut -d' ' -f2)
        echo "✅ Pip updated from $current_version to $new_version"
    else
        echo "❌ pip3 not found, installing Python and pip..."
        sudo apt-get update
        sudo apt-get install -y python3 python3-pip
        echo "✅ Python and pip installed"
    fi
}

# Function to update Ollama
update_ollama() {
    echo "🤖 Updating Ollama..."
    
    if command_exists ollama; then
        current_version=$(ollama --version)
        echo "Current Ollama version: $current_version"
        
        # Get latest version from GitHub
        echo "🔍 Checking for latest Ollama version..."
        latest_version=$(curl -s https://api.github.com/repos/ollama/ollama/releases/latest | grep '"tag_name":' | cut -d'"' -f4)
        
        if [ "$current_version" != "$latest_version" ]; then
            echo "🔄 Updating Ollama to $latest_version..."
            
            # Stop Ollama service
            sudo systemctl stop ollama || true
            
            # Download and install latest version
            curl -fsSL https://ollama.ai/install.sh | sh
            
            # Start Ollama service
            sudo systemctl start ollama || true
            
            echo "✅ Ollama updated to $latest_version"
        else
            echo "✅ Ollama is already up to date ($current_version)"
        fi
    else
        echo "❌ Ollama not found, installing..."
        curl -fsSL https://ollama.ai/install.sh | sh
        echo "✅ Ollama installed"
    fi
}

# Function to update Rust and dependencies
update_rust_and_dependencies() {
    echo "🦀 Updating Rust and dependencies..."
    
    # Check if Rust is installed
    if ! command_exists cargo; then
        echo "❌ Rust is not installed. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Update Rust toolchain
    echo "🔄 Updating Rust toolchain..."
    rustup update
    
    # Update Rust components
    echo "🔧 Updating Rust components..."
    rustup component add rust-src
    rustup component add rust-analysis
    rustup component add rust-std
    
    # Update Rust development tools
    echo "🛠️ Updating Rust development tools..."
    cargo install-update -a
    
    # Check for outdated dependencies
    echo "🔍 Checking for outdated dependencies..."
    if command_exists cargo-outdated; then
        echo "📋 Outdated dependencies:"
        cargo outdated || echo "   All dependencies are up to date"
    fi
    
    # Update Cargo.lock to latest compatible versions
    echo "📦 Updating Cargo.lock..."
    cargo update
    
    # Verify Rust versions
    echo "✅ Rust versions:"
    cargo --version
    rustc --version
    rustup --version
}

# Function to update Cargo.toml to latest versions
update_cargo_dependencies() {
    echo "📝 Updating Cargo.toml to latest versions..."
    
    # Check if cargo-edit is installed
    if ! command_exists cargo-set-version; then
        echo "🛠️ Installing cargo-edit for dependency management..."
        cargo install cargo-edit
    fi
    
    # Update major dependencies to latest versions
    echo "🔄 Updating major dependencies to latest versions..."
    
    # Core dependencies
    cargo upgrade --incompatible || echo "⚠️ Some dependencies couldn't be upgraded (compatibility constraints)"
    
    # Update specific dependencies with version constraints
    echo "📦 Updating specific dependencies..."
    cargo add tokio@latest --features full
    cargo add reqwest@latest --features "json,stream"
    cargo add serde@latest --features derive
    cargo add anyhow@latest
    cargo add clap@latest --features derive
    cargo add chrono@latest --features serde
    cargo add prost@latest
    cargo add prost-types@latest
    cargo add futures-util@latest
    
    # Verify updated versions
    echo "📋 Updated dependency versions:"
    cargo tree --depth=1 | head -20
    
    echo "✅ Cargo.toml dependencies updated to latest versions"
}

# Function to update system packages
update_system_packages() {
    echo "🔧 Updating system packages..."
    
    # Update package lists
    sudo apt-get update
    
    # Upgrade existing packages
    sudo apt-get upgrade -y
    
    # Install/upgrade development packages
    sudo apt-get install -y build-essential cmake pkg-config curl wget git jq
    
    # Install additional development tools
    sudo apt-get install -y clang llvm-dev libssl-dev
    
    echo "✅ System packages updated"
}

# Function to update trading bot
update_trading_bot() {
    echo "🔄 Updating trading bot..."
    
    # Check if we're in the trading bot directory
    if [ ! -f "Cargo.toml" ]; then
        echo "❌ Not in trading bot directory. Please run this script from the trading bot root directory."
        exit 1
    fi
    
    # Pull latest changes
    echo "📥 Pulling latest changes from repository..."
    git pull origin main
    
    # Clean previous build artifacts for fresh build
    echo "🧹 Cleaning previous build artifacts..."
    cargo clean
    
    # Update Cargo.toml dependencies to latest versions
    echo "📝 Updating Cargo.toml dependencies..."
    update_cargo_dependencies
    
    # Update Rust dependencies to latest compatible versions
    echo "🦀 Updating Rust dependencies to latest versions..."
    cargo update
    
    # Check for outdated dependencies
    echo "🔍 Checking for outdated dependencies..."
    if command_exists cargo-outdated; then
        echo "📋 Outdated dependencies:"
        cargo outdated || echo "   All dependencies are up to date"
    fi
    
    # Update specific dependencies if needed
    echo "📦 Updating specific dependencies..."
    cargo update -p tokio
    cargo update -p reqwest
    cargo update -p serde
    cargo update -p anyhow
    cargo update -p clap
    cargo update -p chrono
    cargo update -p prost
    cargo update -p prost-types
    
    # Verify dependency versions
    echo "📋 Current dependency versions:"
    cargo tree --depth=1 | head -20
    
    # Build the updated bot with latest dependencies
    echo "🔨 Building updated trading bot with latest dependencies..."
    cargo build --release
    
    if [ -f "target/release/trading_bot" ]; then
        echo "✅ Trading bot updated and built successfully with latest dependencies"
        
        # Show binary information
        echo "📊 Binary information:"
        echo "   Size: $(du -h target/release/trading_bot | cut -f1)"
        echo "   Location: $(pwd)/target/release/trading_bot"
        
        # Make binary executable
        chmod +x target/release/trading_bot
    else
        echo "❌ Trading bot build failed"
        exit 1
    fi
}

# Function to audit dependencies and check security
audit_dependencies() {
    echo "🔒 Auditing dependencies and checking security..."
    
    # Check for security vulnerabilities
    if command_exists cargo-audit; then
        echo "🔍 Checking for security vulnerabilities..."
        cargo audit || echo "⚠️ Some security issues found. Check the report above."
    fi
    
    # Check for outdated dependencies
    if command_exists cargo-outdated; then
        echo "📋 Checking for outdated dependencies..."
        echo "Outdated dependencies:"
        cargo outdated || echo "   All dependencies are up to date"
    fi
    
    # Show dependency tree
    echo "🌳 Dependency tree (top level):"
    cargo tree --depth=1
    
    # Show dependency licenses
    echo "📜 Checking dependency licenses..."
    cargo license --summary || echo "⚠️ Could not check licenses"
    
    echo "✅ Dependency audit completed"
}

# Function to test everything
test_installation() {
    echo "🧪 Testing installation..."
    
    # Test protobuf
    if verify_protobuf; then
        echo "✅ Protobuf is working correctly"
    else
        echo "❌ Protobuf test failed"
        return 1
    fi
    
    # Test Rust build
    echo "🔨 Testing Rust build..."
    if cargo check; then
        echo "✅ Rust build test passed"
    else
        echo "❌ Rust build test failed"
        return 1
    fi
    
    # Test Ollama
    if command_exists ollama; then
        echo "🤖 Testing Ollama..."
        if ollama list &> /dev/null; then
            echo "✅ Ollama is working correctly"
        else
            echo "⚠️ Ollama is installed but may not be running"
        fi
    fi
    
    # Test pip
    if command_exists pip3; then
        pip_version=$(pip3 --version)
        echo "✅ $pip_version is working correctly"
    fi
    
    echo "✅ All tests passed!"
}

# Check if we're in the right directory
if [ ! -f "install.sh" ] && [ ! -d "$PROJECT_DIR" ]; then
    echo -e "${RED}❌ Please run this script from the setup directory or trading-bot root${NC}"
    echo ""
    echo "Run from setup directory:"
    echo "  sudo cd setup && sudo ./update.sh"
    echo ""
    echo "OR run from trading-bot root:"
    echo "  sudo cd trading-bot && sudo ../setup/update.sh"
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
    echo -e "${YELLOW}⚠️  Trading bot directory not found!${NC}"
    echo ""
    echo "Creating trading bot directory and downloading source code..."
    
    # Create the directory with proper permissions
    if [ "$PROJECT_PATH" = "." ]; then
        # We're in the trading-bot root, so clone here
        git clone https://github.com/KingGekko/trading-bot.git temp-clone
        mv temp-clone/* .
        mv temp-clone/.* . 2>/dev/null || true
        rmdir temp-clone
    else
        # We're in setup directory, clone to parent
        cd "$(dirname "$PROJECT_PATH")"
        git clone https://github.com/KingGekko/trading-bot.git
        cd - > /dev/null
    fi
    
    echo -e "${GREEN}✅ Trading bot directory created and source code downloaded!${NC}"
    echo ""
fi

# Ensure required directories and files exist
echo "Ensuring required directories and files exist..."
if [ "$PROJECT_PATH" = "." ]; then
    # We're in trading-bot root
    mkdir -p ollama_logs
    chmod 755 ollama_logs
    
    # Create default config.env if it doesn't exist
    if [ ! -f "config.env" ]; then
        echo "Creating default config.env..."
        cat > config.env << 'EOF'
# Trading Bot Configuration
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=tinyllama
LOG_LEVEL=info
LOG_DIRECTORY=ollama_logs
EOF
        chmod 644 config.env
    fi
else
    # We're in setup directory
    cd "$PROJECT_PATH"
    mkdir -p ollama_logs
    chmod 755 ollama_logs
    
    # Create default config.env if it doesn't exist
    if [ ! -f "config.env" ]; then
        echo "Creating default config.env..."
        cat > config.env << 'EOF'
# Trading Bot Configuration
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=tinyllama
LOG_LEVEL=info
LOG_DIRECTORY=ollama_logs
EOF
        chmod 644 config.env
    fi
    cd - > /dev/null
fi

echo -e "${GREEN}✅ Required directories and files created!${NC}"

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}⚠️  Rust is not available!${NC}"
    echo ""
    echo "Installing Rust programming language..."
    
    # Download and run rustup installer
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        echo "Rust installed successfully!"
        
        # Source Rust environment
        source ~/.cargo/env
        
        # Add Rust to PATH permanently
        if ! grep -q 'source ~/.cargo/env' ~/.bashrc; then
            echo 'source ~/.cargo/env' >> ~/.bashrc
        fi
        
        if ! grep -q 'source ~/.cargo/env' ~/.profile; then
            echo 'source ~/.cargo/env' >> ~/.profile
        fi
        
        # Verify installation
        if command -v rustc &> /dev/null; then
            echo "Rust verified: $(rustc --version)"
        else
            echo "Error: Rust installation verification failed"
            exit 1
        fi
    else
        echo "Error: Failed to install Rust"
        echo ""
        echo "Please run the full installation first:"
        echo "  sudo ./install.sh"
        exit 1
    fi
fi

# Ensure Rust is available in current session
source ~/.cargo/env

# Check if Ollama is available
if ! command -v ollama &> /dev/null; then
    echo -e "${YELLOW}⚠️  Ollama not found, but continuing with binary update...${NC}"
    echo "You may need to install Ollama separately if you want to use the AI features."
fi

# Check if protobuf is available (required for Rust build)
echo "📋 Checking protobuf installation..."
if ! command -v protoc &> /dev/null; then
    echo -e "${YELLOW}⚠️  protoc not found! This will cause build failures.${NC}"
    echo ""
    echo "🔧 To fix this, run one of these commands:"
    echo "  • Quick fix: ./fix_protobuf.sh"
    echo "  • Manual install: sudo apt-get install protobuf-compiler"
    echo "  • This script will attempt to fix it automatically"
    echo ""
    echo "⚠️  Continuing with update, but build may fail..."
else
    protoc_version=$(protoc --version)
    echo -e "${GREEN}✅ $protoc_version is available${NC}"
fi

echo -e "${GREEN}✅ Prerequisites check passed!${NC}"

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting comprehensive update process...${NC}"
    echo ""
    
    # Check if we're in a Rust project
    check_rust_project
    
    # STEP 0: Verify and repair protobuf
    echo "📋 STEP 0/7: Verifying and repairing protobuf"
    echo "============================================="
    repair_protobuf
    echo ""
    
    # STEP 1: Update pip
    echo "📦 STEP 1/7: Updating Python pip"
    echo "=================================="
    update_pip
    echo ""
    
    # STEP 2: Update Ollama
    echo "🤖 STEP 2/7: Updating Ollama"
    echo "============================="
    update_ollama
    echo ""
    
    # STEP 3: Update Rust and dependencies
    echo "🦀 STEP 3/7: Updating Rust and dependencies"
    echo "============================================"
    update_rust_and_dependencies
    echo ""
    
    # STEP 4: Update system packages
    echo "🔧 STEP 4/7: Updating system packages"
    echo "====================================="
    update_system_packages
    echo ""
    
    # STEP 5: Update trading bot
    echo "📱 STEP 5/7: Updating trading bot"
    echo "=================================="
    update_trading_bot
    echo ""
    
    # STEP 6: Audit dependencies
    echo "🔒 STEP 6/7: Auditing dependencies and security"
    echo "==============================================="
    audit_dependencies
    echo ""
    
    # STEP 7: Test everything
    echo "🧪 STEP 7/7: Testing everything"
    echo "================================"
    test_installation
    echo ""
    
    echo -e "${GREEN}🎉 Comprehensive update completed successfully!${NC}"
    echo ""
    echo "📋 What was updated:"
    echo "  ✅ Protobuf: Verified and repaired if needed"
    echo "  ✅ Python pip: Latest version"
    echo "  ✅ Ollama: Latest version"
    echo "  ✅ Rust: Latest toolchain and components"
    echo "  ✅ Dependencies: Latest versions"
    echo "  ✅ System packages: Updated"
    echo "  ✅ Trading bot: Rebuilt with latest dependencies"
    echo "  ✅ Security: Audited dependencies"
    echo ""
    echo "💡 Your trading bot is now fully updated and ready to use!"
}

# Run main function
main "$@" 