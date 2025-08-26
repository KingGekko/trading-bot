#!/bin/bash
# Trading Bot - Installation without Git
# This script downloads and installs everything without requiring Git initially

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🚀 Trading Bot - No-Git Installation${NC}"
echo -e "${CYAN}====================================${NC}"
echo ""
echo "This script installs the trading bot without requiring Git upfront."
echo "It will download the source code directly and then install everything."
echo ""

# Check if curl is available
if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl is required but not installed.${NC}"
    echo "Please install curl first:"
    echo "  Ubuntu/Debian: sudo apt install curl"
    echo "  CentOS/RHEL:   sudo yum install curl"
    echo "  Alpine:        sudo apk add curl"
    exit 1
fi

# Download the bootstrap script directly
BOOTSTRAP_URL="https://raw.githubusercontent.com/KingGekko/trading-bot/main/setup/bootstrap.sh"

echo -e "${BLUE}📥 Downloading bootstrap script...${NC}"
curl -fsSL "$BOOTSTRAP_URL" -o bootstrap_temp.sh

echo -e "${GREEN}✅ Bootstrap script downloaded!${NC}"

# Make it executable and run it
chmod +x bootstrap_temp.sh

echo ""
echo -e "${CYAN}🚀 Running bootstrap installation...${NC}"
./bootstrap_temp.sh

# Clean up
rm -f bootstrap_temp.sh

echo -e "${GREEN}🎉 Installation complete!${NC}"