#!/bin/bash
# Trading Bot - Dependencies Installation Script
# This script installs all required dependencies for the trading bot

set -e  # Exit on any error

echo "🔧 Trading Bot - Dependencies Installation"
echo "=========================================="

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

echo "📋 Detected OS: $DISTRO_NAME"

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
        echo "⚠️  Unknown distribution. Please install dependencies manually."
        echo "Required: git, curl, gcc, openssl-dev, pkg-config"
        exit 1
        ;;
esac

# Install build dependencies
echo "🛠️  Installing build dependencies..."
case $DISTRO in
    "debian")
        sudo apt install -y \
            git \
            curl \
            build-essential \
            pkg-config \
            libssl-dev \
            ca-certificates
        ;;
    "redhat")
        if command -v dnf &> /dev/null; then
            sudo dnf install -y \
                git \
                curl \
                gcc \
                gcc-c++ \
                openssl-devel \
                pkg-config \
                ca-certificates
        else
            sudo yum install -y \
                git \
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
            git \
            curl \
            build-base \
            openssl-dev \
            pkgconfig \
            ca-certificates
        ;;
esac

echo "✅ Dependencies installed successfully!"
echo ""
echo "📋 Installed packages:"
echo "   • Git (version control)"
echo "   • Curl (file downloads)"
echo "   • Build tools (gcc, make, etc.)"
echo "   • OpenSSL development libraries"
echo "   • Package config tools"
echo "   • CA certificates"
echo ""
echo "🎯 Next step: Run ./install_rust.sh"