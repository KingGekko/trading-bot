#!/bin/bash
# Setup script for Linux VM

echo "🐧 Setting up Trading Bot on Linux..."

# Detect Linux distribution
if [ -f /etc/debian_version ]; then
    DISTRO="debian"
elif [ -f /etc/redhat-release ]; then
    DISTRO="redhat"
elif [ -f /etc/alpine-release ]; then
    DISTRO="alpine"
else
    DISTRO="unknown"
fi

echo "📦 Installing dependencies for $DISTRO..."

# Install build dependencies based on distribution
case $DISTRO in
    "debian")
        sudo apt update
        sudo apt install -y build-essential pkg-config libssl-dev curl
        ;;
    "redhat")
        sudo yum install -y gcc openssl-devel pkg-config curl
        ;;
    "alpine")
        sudo apk add build-base openssl-dev pkgconfig curl
        ;;
    *)
        echo "⚠️  Unknown distribution. Please install: gcc, openssl-dev, pkg-config manually"
        ;;
esac

# Install Rust if not already installed
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    echo "✅ Rust already installed"
fi

# Extract source if archive exists
if [ -f "trading_bot_source.tar.gz" ]; then
    echo "📁 Extracting source code..."
    tar -xzf trading_bot_source.tar.gz
fi

echo "🔨 Building release binary..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📍 Binary location: $(pwd)/target/release/trading_bot"
    echo "📏 Binary size: $(du -h target/release/trading_bot | cut -f1)"
    
    # Make binary executable
    chmod +x target/release/trading_bot
    
    echo ""
    echo "🧪 Testing binary..."
    ./target/release/trading_bot --help
    
    echo ""
    echo "🎯 Setup complete! Usage examples:"
    echo "  ./target/release/trading_bot -t 'Test prompt'"
    echo "  ./target/release/trading_bot -i"
    echo ""
    echo "⚠️  Make sure Ollama is installed and running:"
    echo "  curl -fsSL https://ollama.ai/install.sh | sh"
    echo "  ollama serve &"
    echo "  ollama pull tinyllama"
else
    echo "❌ Build failed!"
    exit 1
fi