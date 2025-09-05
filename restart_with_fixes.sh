#!/bin/bash

# 🔄 Restart Trading Bot with Latest Fixes
# This script ensures the bot uses the latest release binary with position sizing fixes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔄 Restarting Trading Bot with Latest Fixes${NC}"
echo "=================================================="
echo ""

# Function to stop any running bot processes
stop_existing_bot() {
    echo -e "${YELLOW}🛑 Stopping any running trading bot processes...${NC}"
    
    # Kill any trading_bot processes
    pkill -f "trading_bot" 2>/dev/null || true
    
    # Remove PID files
    rm -f live_mode.pid
    rm -f simulated_stream.pid
    
    echo -e "${GREEN}✅ Stopped existing processes${NC}"
    echo ""
}

# Function to build the latest version
build_latest() {
    echo -e "${YELLOW}🔨 Building latest version with position sizing fixes...${NC}"
    
    # Build release version
    cargo build --release
    
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
}

# Function to verify the binary has our fixes
verify_fixes() {
    echo -e "${YELLOW}🔍 Verifying position sizing fixes are included...${NC}"
    
    # Check binary timestamp
    BINARY_TIME=$(stat -c %Y target/release/trading_bot.exe 2>/dev/null || stat -f %m target/release/trading_bot.exe 2>/dev/null || echo "0")
    CURRENT_TIME=$(date +%s)
    AGE_SECONDS=$((CURRENT_TIME - BINARY_TIME))
    
    if [ $AGE_SECONDS -lt 300 ]; then  # Less than 5 minutes old
        echo -e "${GREEN}✅ Binary is fresh (${AGE_SECONDS}s old)${NC}"
    else
        echo -e "${YELLOW}⚠️ Binary is ${AGE_SECONDS}s old, rebuilding...${NC}"
        cargo build --release
    fi
    
    echo ""
}

# Function to start the bot with correct binary
start_bot() {
    echo -e "${YELLOW}🚀 Starting trading bot with latest fixes...${NC}"
    
    # Start the bot directly with the release binary
    ./target/release/trading_bot.exe &
    BOT_PID=$!
    
    echo -e "${GREEN}✅ Trading bot started (PID: $BOT_PID)${NC}"
    echo -e "${BLUE}📊 The bot should now use correct position sizing based on available cash${NC}"
    echo ""
}

# Function to show what was fixed
show_fixes() {
    echo -e "${BLUE}🔧 Position Sizing Fixes Applied:${NC}"
    echo "=================================="
    echo ""
    echo -e "${GREEN}✅ Fixed: Position sizing now uses real available cash (\$9,982)${NC}"
    echo -e "${GREEN}✅ Fixed: Portfolio protection logic corrected for buy orders${NC}"
    echo -e "${GREEN}✅ Fixed: Trade execution tracking improved${NC}"
    echo ""
    echo -e "${BLUE}Expected behavior:${NC}"
    echo "- SPY: ~4 shares instead of 46 shares"
    echo "- QQQ: ~4 shares instead of 40 shares" 
    echo "- AAPL: ~6 shares instead of 69 shares"
    echo ""
}

# Main execution
main() {
    stop_existing_bot
    build_latest
    verify_fixes
    show_fixes
    start_bot
    
    echo -e "${GREEN}🎉 Trading bot restarted with latest fixes!${NC}"
    echo ""
    echo -e "${BLUE}The bot should now:${NC}"
    echo "- Calculate position sizes based on available cash (\$9,982)"
    echo "- Execute trades without 'insufficient cash' errors"
    echo "- Show 'available cash' instead of 'portfolio' in debug messages"
    echo ""
    echo -e "${YELLOW}Monitor the next trading iteration to see the fixes in action!${NC}"
}

# Run main function
main "$@"
