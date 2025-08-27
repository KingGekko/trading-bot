#!/bin/bash

# Protobuf Fix Script
# This script fixes common protobuf installation issues that cause Rust builds to fail

set -e

echo "🔧 Protobuf Fix Script"
echo "======================"
echo "This script will fix protobuf installation issues that commonly cause:"
echo "  • 'Could not find protoc' errors"
echo "  • Build script failures"
echo "  • Missing PROTOC environment variable"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command_exists apt-get; then
            echo "ubuntu"
        elif command_exists yum; then
            echo "centos"
        elif command_exists dnf; then
            echo "fedora"
        elif command_exists pacman; then
            echo "arch"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to install protobuf on Ubuntu/Debian
install_protobuf_ubuntu() {
    echo -e "${BLUE}📦 Installing protobuf on Ubuntu/Debian...${NC}"
    
    # Update package list
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
        return 1
    fi
    
    return 0
}

# Function to install protobuf on CentOS/RHEL
install_protobuf_centos() {
    echo -e "${BLUE}📦 Installing protobuf on CentOS/RHEL...${NC}"
    
    if command_exists yum; then
        sudo yum install -y protobuf-compiler
    elif command_exists dnf; then
        sudo dnf install -y protobuf-compiler
    else
        echo "No package manager found"
        return 1
    fi
    
    return 0
}

# Function to install protobuf on macOS
install_protobuf_macos() {
    echo -e "${BLUE}📦 Installing protobuf on macOS...${NC}"
    
    if command_exists brew; then
        brew install protobuf
    else
        echo "Homebrew not found. Please install Homebrew first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        return 1
    fi
    
    return 0
}

# Function to install protobuf from source
install_protobuf_source() {
    echo -e "${BLUE}🔄 Installing protobuf from source...${NC}"
    
    # Install build dependencies
    if [[ "$(detect_os)" == "ubuntu" ]]; then
        sudo apt-get install -y build-essential cmake pkg-config jq
    elif [[ "$(detect_os)" == "centos" ]]; then
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y cmake pkg-config jq
    fi
    
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
        
        echo -e "${GREEN}✅ Protobuf built and installed from source${NC}"
    else
        echo -e "${RED}❌ Failed to download protobuf source${NC}"
        return 1
    fi
    
    cd - > /dev/null
    rm -rf "$PROTOC_DIR"
    
    return 0
}

# Function to download pre-built protobuf binary
download_protobuf_binary() {
    echo -e "${BLUE}📥 Downloading pre-built protobuf binary...${NC}"
    
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
            if [[ "$(detect_os)" == "ubuntu" ]]; then
                sudo apt-get install -y unzip
            elif [[ "$(detect_os)" == "centos" ]]; then
                sudo yum install -y unzip
            fi
        fi
        
        # Extract and install
        unzip protoc.zip
        sudo mv bin/protoc /usr/local/bin/
        sudo chmod +x /usr/local/bin/protoc
        
        # Clean up
        rm -rf bin include protoc.zip
        
        echo -e "${GREEN}✅ Protobuf binary installed${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to download protobuf binary${NC}"
        return 1
    fi
}

# Function to verify protobuf installation
verify_protobuf() {
    echo -e "${BLUE}🔍 Verifying protobuf installation...${NC}"
    
    if command_exists protoc; then
        protoc_version=$(protoc --version)
        echo -e "${GREEN}✅ $protoc_version is installed${NC}"
        
        # Check if PROTOC environment variable is set
        if [ -z "$PROTOC" ]; then
            PROTOC_PATH=$(which protoc)
            if [ -n "$PROTOC_PATH" ]; then
                echo "🔧 Setting PROTOC environment variable..."
                export PROTOC="$PROTOC_PATH"
                echo "export PROTOC=\"$PROTOC_PATH\"" >> ~/.bashrc
                echo "export PROTOC=\"$PROTOC_PATH\"" >> ~/.profile
                echo -e "${GREEN}✅ PROTOC environment variable set${NC}"
            fi
        else
            echo "✅ PROTOC environment variable is already set: $PROTOC"
        fi
        
        # Test protobuf compilation
        echo "🧪 Testing protobuf compilation..."
        if [ -f "proto/receipt.proto" ]; then
            if protoc --cpp_out=/tmp proto/receipt.proto 2>/dev/null; then
                echo -e "${GREEN}✅ Protobuf compilation test passed${NC}"
                rm -f /tmp/receipt.pb.h /tmp/receipt.pb.cc
                return 0
            else
                echo -e "${RED}❌ Protobuf compilation test failed${NC}"
                return 1
            fi
        else
            echo "⚠️ No proto files found to test compilation"
            return 0
        fi
    else
        echo -e "${RED}❌ protoc command not found${NC}"
        return 1
    fi
}

# Function to test Rust build
test_rust_build() {
    echo -e "${BLUE}🧪 Testing Rust build...${NC}"
    
    if [ -f "Cargo.toml" ]; then
        echo "🔨 Testing cargo build..."
        if cargo check; then
            echo -e "${GREEN}✅ Rust build test passed${NC}"
            return 0
        else
            echo -e "${RED}❌ Rust build test failed${NC}"
            return 1
        fi
    else
        echo "⚠️ No Cargo.toml found, skipping build test"
        return 0
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting protobuf fix process...${NC}"
    echo ""
    
    # Check if we're in a Rust project
    check_rust_project
    
    # Detect OS
    OS=$(detect_os)
    echo "🌍 Detected OS: $OS"
    echo ""
    
    # Check current protobuf status
    if verify_protobuf; then
        echo -e "${GREEN}✅ Protobuf is already working correctly!${NC}"
        echo ""
        
        # Test Rust build
        if test_rust_build; then
            echo -e "${GREEN}🎉 Everything is working correctly!${NC}"
            exit 0
        else
            echo -e "${YELLOW}⚠️ Protobuf works but Rust build failed${NC}"
            echo "This might be a different issue. Check the error messages above."
            exit 1
        fi
    fi
    
    echo -e "${YELLOW}⚠️ Protobuf needs to be installed or fixed${NC}"
    echo ""
    
    # Try different installation methods
    echo "🔄 Attempting to install/fix protobuf..."
    
    # Method 1: OS-specific package manager
    case $OS in
        ubuntu)
            if install_protobuf_ubuntu; then
                echo "✅ Package manager installation successful"
            else
                echo "❌ Package manager installation failed"
            fi
            ;;
        centos)
            if install_protobuf_centos; then
                echo "✅ Package manager installation successful"
            else
                echo "❌ Package manager installation failed"
            fi
            ;;
        macos)
            if install_protobuf_macos; then
                echo "✅ Homebrew installation successful"
            else
                echo "❌ Homebrew installation failed"
            fi
            ;;
        *)
            echo "⚠️ Unsupported OS for package manager installation"
            ;;
    esac
    
    # Method 2: Download pre-built binary
    if ! verify_protobuf; then
        echo "🔄 Trying pre-built binary download..."
        if download_protobuf_binary; then
            echo "✅ Binary download successful"
        else
            echo "❌ Binary download failed"
        fi
    fi
    
    # Method 3: Build from source
    if ! verify_protobuf; then
        echo "🔄 Trying source build..."
        if install_protobuf_source; then
            echo "✅ Source build successful"
        else
            echo "❌ Source build failed"
        fi
    fi
    
    # Final verification
    echo ""
    echo "🔍 Final verification..."
    if verify_protobuf; then
        echo -e "${GREEN}✅ Protobuf installation successful!${NC}"
        
        # Test Rust build
        if test_rust_build; then
            echo -e "${GREEN}🎉 Everything is now working correctly!${NC}"
            echo ""
            echo "📋 What was fixed:"
            echo "  ✅ Protobuf compiler installed"
            echo "  ✅ PROTOC environment variable set"
            echo "  ✅ Protobuf compilation test passed"
            echo "  ✅ Rust build test passed"
            echo ""
            echo "💡 You can now build your Rust project successfully!"
        else
            echo -e "${YELLOW}⚠️ Protobuf works but Rust build still fails${NC}"
            echo "This might be a different issue. Check the error messages above."
            exit 1
        fi
    else
        echo -e "${RED}❌ All protobuf installation methods failed${NC}"
        echo ""
        echo "🔧 Manual installation required:"
        echo "  1. Download from: https://github.com/protocolbuffers/protobuf/releases"
        echo "  2. Extract and add to PATH"
        echo "  3. Set PROTOC environment variable"
        echo ""
        echo "📚 For more help, see: https://docs.rs/prost-build/#sourcing-protoc"
        exit 1
    fi
}

# Run main function
main "$@" 