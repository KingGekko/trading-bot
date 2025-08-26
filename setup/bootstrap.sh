#!/bin/bash
# Trading Bot - Bootstrap Script
# This script installs Git first, then runs the complete setup

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🚀 Trading Bot - Bootstrap Setup${NC}"
echo -e "${CYAN}================================${NC}"
echo ""
echo "This script will:"
echo "  1. Install Git (if not present)"
echo "  2. Clone the trading bot repository"
echo "  3. Run the complete setup automatically"
echo ""

# Detect Linux distribution
if [ -f /etc/debian_version ]; then
    DISTRO="debian"
    DISTRO_NAME="Ubuntu/Debian"
    PKG_MANAGER="apt"
    INSTALL_CMD="sudo apt update && sudo apt install -y"
elif [ -f /etc/redhat-release ]; then
    DISTRO="redhat"
    DISTRO_NAME="CentOS/RHEL/Fedora"
    if command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
        INSTALL_CMD="sudo dnf install -y"
    else
        PKG_MANAGER="yum"
        INSTALL_CMD="sudo yum install -y"
    fi
elif [ -f /etc/alpine-release ]; then
    DISTRO="alpine"
    DISTRO_NAME="Alpine Linux"
    PKG_MANAGER="apk"
    INSTALL_CMD="sudo apk update && sudo apk add"
else
    DISTRO="unknown"
    DISTRO_NAME="Unknown"
fi

echo -e "${BLUE}📋 Detected OS: $DISTRO_NAME${NC}"

# Check if Git is installed
if command -v git &> /dev/null; then
    echo -e "${GREEN}✅ Git is already installed!${NC}"
    git --version
else
    echo -e "${YELLOW}📦 Git not found. Installing Git...${NC}"
    
    case $DISTRO in
        "debian")
            sudo apt update
            sudo apt install -y git curl
            ;;
        "redhat")
            if command -v dnf &> /dev/null; then
                sudo dnf install -y git curl
            else
                sudo yum install -y git curl
            fi
            ;;
        "alpine")
            sudo apk update
            sudo apk add git curl
            ;;
        *)
            echo -e "${RED}❌ Unknown distribution. Please install Git manually:${NC}"
            echo "Ubuntu/Debian: sudo apt install git"
            echo "CentOS/RHEL:   sudo yum install git"
            echo "Fedora:        sudo dnf install git"
            echo "Alpine:        sudo apk add git"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}✅ Git installed successfully!${NC}"
    git --version
fi

# Clone the repository
REPO_URL="https://github.com/KingGekko/trading-bot.git"
PROJECT_DIR="trading-bot"

echo ""
echo -e "${BLUE}📥 Cloning trading bot repository...${NC}"

# Remove existing directory if it exists
if [ -d "$PROJECT_DIR" ]; then
    echo "🗑️  Removing existing directory: $PROJECT_DIR"
    rm -rf "$PROJECT_DIR"
fi

# Clone the repository
git clone "$REPO_URL"

echo -e "${GREEN}✅ Repository cloned successfully!${NC}"

# Navigate to setup directory and run complete setup
cd "$PROJECT_DIR/setup"

echo ""
echo -e "${CYAN}🚀 Running complete setup...${NC}"
echo -e "${CYAN}============================${NC}"

# Make script executable and run it
chmod +x complete_setup.sh
./complete_setup.sh

echo ""
echo -e "${GREEN}🎉 BOOTSTRAP COMPLETE!${NC}"
echo -e "${GREEN}======================${NC}"
echo ""
echo -e "${GREEN}✅ Trading bot has been fully installed and is ready to use!${NC}"
echo ""
echo -e "${CYAN}📍 Your trading bot is located at: $(pwd)/../target/release/trading_bot${NC}"
echo ""
echo -e "${YELLOW}💡 Quick start commands:${NC}"
echo "   cd ../  # Go to trading-bot directory"
echo "   ./target/release/trading_bot -t 'Analyze Bitcoin'  # Test mode"
echo "   ./target/release/trading_bot -i                    # Interactive mode"
echo ""
echo -e "${GREEN}🚀 Happy trading!${NC}"