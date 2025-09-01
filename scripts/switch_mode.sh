#!/bin/bash

# 🔄 Mode Switching Script
# This script helps switch between test and live modes using the unified config.env

set -e

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration file - detect if running from scripts/ or root directory
if [ -f "../config.env" ]; then
    CONFIG_FILE="../config.env"
elif [ -f "config.env" ]; then
    CONFIG_FILE="config.env"
else
    echo -e "${RED}❌ config.env not found. Please run from project root or scripts directory.${NC}"
    exit 1
fi

# Function to show current mode
show_current_mode() {
    if [ -f "$CONFIG_FILE" ]; then
        local mode=$(grep "^OPERATION_MODE=" "$CONFIG_FILE" | cut -d'=' -f2)
        local feed=$(grep "^ALPACA_FEED=" "$CONFIG_FILE" | cut -d'=' -f2)
        local paper=$(grep "^ALPACA_PAPER_TRADING=" "$CONFIG_FILE" | cut -d'=' -f2)
        
        echo -e "${BLUE}📊 Current Configuration:${NC}"
        echo "  Mode: $mode"
        echo "  Feed: $feed"
        echo "  Paper Trading: $paper"
        echo ""
    else
        echo -e "${RED}❌ Configuration file not found: $CONFIG_FILE${NC}"
        exit 1
    fi
}

# Function to switch to paper trading mode
switch_to_paper() {
    echo -e "${BLUE}🧪 Switching to PAPER TRADING mode...${NC}"
    
    # Update operation mode
    sed -i 's/^OPERATION_MODE=.*/OPERATION_MODE=paper/' "$CONFIG_FILE"
    
    # Update feed to test (paper trading uses test feed)
    sed -i 's/^ALPACA_FEED=.*/ALPACA_FEED=test/' "$CONFIG_FILE"
    
    # Update market data WebSocket URL
    sed -i 's|^ALPACA_MARKET_DATA_WEBSOCKET_URL=.*|ALPACA_MARKET_DATA_WEBSOCKET_URL=wss://stream.data.alpaca.markets/v2/test|' "$CONFIG_FILE"
    
    # Enable paper trading
    sed -i 's/^ALPACA_PAPER_TRADING=.*/ALPACA_PAPER_TRADING=true/' "$CONFIG_FILE"
    
    echo -e "${GREEN}✅ Switched to PAPER TRADING mode${NC}"
    echo "  - Using paper trading feed (no real money)"
    echo "  - Paper trading enabled"
    echo "  - Safe for development and testing"
    echo "  - Uses APCA-API-KEY-ID and APCA-API-SECRET-KEY"
    echo ""
}

# Function to switch to live mode
switch_to_live() {
    echo -e "${BLUE}🚀 Switching to LIVE mode...${NC}"
    
    # Check if API keys are configured
    if grep -q "your_alpaca_api_key_here" "$CONFIG_FILE"; then
        echo -e "${RED}❌ Please configure your API keys first!${NC}"
        echo "  Edit $CONFIG_FILE and set your real Alpaca API credentials"
        exit 1
    fi
    
    # Update operation mode
    sed -i 's/^OPERATION_MODE=.*/OPERATION_MODE=live/' "$CONFIG_FILE"
    
    # Update feed to iex (live data)
    sed -i 's/^ALPACA_FEED=.*/ALPACA_FEED=iex/' "$CONFIG_FILE"
    
    # Update market data WebSocket URL
    sed -i 's|^ALPACA_MARKET_DATA_WEBSOCKET_URL=.*|ALPACA_MARKET_DATA_WEBSOCKET_URL=wss://stream.data.alpaca.markets/v2/iex|' "$CONFIG_FILE"
    
    # Disable paper trading for live mode
    sed -i 's/^ALPACA_PAPER_TRADING=.*/ALPACA_PAPER_TRADING=false/' "$CONFIG_FILE"
    
    echo -e "${GREEN}✅ Switched to LIVE mode${NC}"
    echo "  - Using live feed (real market data)"
    echo "  - Paper trading disabled"
    echo "  - ⚠️  Use with caution - real trading environment"
    echo ""
}

# Function to show usage
show_usage() {
    echo -e "${BLUE}🔄 Trading Bot Mode Switcher${NC}"
    echo "================================="
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
echo "  paper         - Switch to paper trading mode (safe for development)"
echo "  live          - Switch to live trading mode (real trading)"
    echo "  status        - Show current mode configuration"
    echo "  help          - Show this help message"
    echo ""
    echo "Examples:"
echo "  $0 paper      # Switch to paper trading mode"
echo "  $0 live       # Switch to live trading mode"
    echo "  $0 status     # Show current configuration"
    echo ""
}

# Main script logic
case "${1:-help}" in
    paper)
        switch_to_paper
        show_current_mode
        ;;
    live)
        switch_to_live
        show_current_mode
        ;;
    status)
        show_current_mode
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo -e "${YELLOW}💡 Current Status:${NC}"
        echo ""
        show_current_mode
        echo -e "${BLUE}📖 For help: $0 help${NC}"
        echo ""
        show_usage
        ;;
esac
