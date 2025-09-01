#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script header
echo -e "${BLUE}📊 Trading Bot Stream Viewer${NC}"
echo -e "${BLUE}============================${NC}"

# Configuration file - detect if running from scripts/ or root directory
if [ -f "../config.env" ]; then
    CONFIG_FILE="../config.env"
elif [ -f "config.env" ]; then
    CONFIG_FILE="config.env"
else
    echo -e "${RED}❌ config.env not found. Please run from project root or scripts directory.${NC}"
    exit 1
fi

# Load configuration
source "$CONFIG_FILE"

# Determine data directory based on operation mode
DATA_DIR="live_data"
if grep -q "^OPERATION_MODE=paper" "$CONFIG_FILE"; then
    DATA_DIR="sandbox_data"
fi

# Function to show available streams
show_streams() {
    echo -e "${CYAN}📡 Available Streams:${NC}"
    echo ""
    echo -e "${GREEN}1. Crypto Stream${NC}"
    echo "   📁 File: $DATA_DIR/crypto_stream.json"
    echo "   💱 Symbols: BTC/USD, ETH/USD"
    echo "   🔄 Updates: Every 2 seconds"
    echo ""
    echo -e "${GREEN}2. Stocks Stream${NC}"
    echo "   📁 File: $DATA_DIR/stocks_stream.json"
    echo "   📈 Symbols: AAPL, SPY, TSLA"
    echo "   🔄 Updates: Every 2 seconds"
    echo ""
    echo -e "${GREEN}3. Options Stream${NC}"
    echo "   📁 File: $DATA_DIR/options_stream.json"
    echo "   🎯 Symbols: SPY Calls & Puts"
    echo "   🔄 Updates: Every 2 seconds"
    echo ""
    echo -e "${GREEN}4. News Stream${NC}"
    echo "   📁 File: $DATA_DIR/news_stream.json"
    echo "   📰 Content: Market news & analysis"
    echo "   🔄 Updates: Every 2 seconds"
    echo ""
    echo -e "${GREEN}5. All Streams${NC}"
    echo "   📁 Files: All stream files"
    echo "   🔍 Monitor: Complete overview"
    echo ""
    echo -e "${GREEN}6. Stream Status${NC}"
    echo "   📊 Check: Current stream status"
    echo ""
    echo -e "${GREEN}0. Exit${NC}"
    echo ""
}

# Function to watch crypto stream
watch_crypto() {
    echo -e "${GREEN}🔍 Watching Crypto Stream...${NC}"
    echo -e "${BLUE}   File: $DATA_DIR/crypto_stream.json${NC}"
    echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
    echo ""
    
    if [ ! -f "$DATA_DIR/crypto_stream.json" ]; then
        echo -e "${RED}❌ Crypto stream file not found.${NC}"
        echo -e "${YELLOW}   Make sure the simulated stream is running:${NC}"
        echo -e "${YELLOW}   ./scripts.sh stream start${NC}"
        return 1
    fi
    
    # Watch the crypto stream file
    while true; do
        clear
        echo -e "${GREEN}🔍 Crypto Stream - $(date)${NC}"
        echo -e "${BLUE}================================${NC}"
        cat "$DATA_DIR/crypto_stream.json" | jq '.' 2>/dev/null || cat "$DATA_DIR/crypto_stream.json"
        echo ""
        echo -e "${YELLOW}🔄 Refreshing in 2 seconds... (Ctrl+C to stop)${NC}"
        sleep 2
    done
}

# Function to watch stocks stream
watch_stocks() {
    echo -e "${GREEN}🔍 Watching Stocks Stream...${NC}"
    echo -e "${BLUE}   File: $DATA_DIR/stocks_stream.json${NC}"
    echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
    echo ""
    
    if [ ! -f "$DATA_DIR/stocks_stream.json" ]; then
        echo -e "${RED}❌ Stocks stream file not found.${NC}"
        echo -e "${YELLOW}   Make sure the simulated stream is running:${NC}"
        echo -e "${YELLOW}   ./scripts.sh stream start${NC}"
        return 1
    fi
    
    # Watch the stocks stream file
    while true; do
        clear
        echo -e "${GREEN}🔍 Stocks Stream - $(date)${NC}"
        echo -e "${BLUE}================================${NC}"
        cat "$DATA_DIR/stocks_stream.json" | jq '.' 2>/dev/null || cat "$DATA_DIR/stocks_stream.json"
        echo ""
        echo -e "${YELLOW}🔄 Refreshing in 2 seconds... (Ctrl+C to stop)${NC}"
        sleep 2
    done
}

# Function to watch options stream
watch_options() {
    echo -e "${GREEN}🔍 Watching Options Stream...${NC}"
    echo -e "${BLUE}   File: live_data/options_stream.json${NC}"
    echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
    echo ""
    
    if [ ! -f "live_data/options_stream.json" ]; then
        echo -e "${RED}❌ Options stream file not found.${NC}"
        echo -e "${YELLOW}   Make sure the simulated stream is running:${NC}"
        echo -e "${YELLOW}   ./scripts.sh stream start${NC}"
        return 1
    fi
    
    # Watch the options stream file
    while true; do
        clear
        echo -e "${GREEN}🔍 Options Stream - $(date)${NC}"
        echo -e "${BLUE}================================${NC}"
        cat live_data/options_stream.json | jq '.' 2>/dev/null || cat live_data/options_stream.json
        echo ""
        echo -e "${YELLOW}🔄 Refreshing in 2 seconds... (Ctrl+C to stop)${NC}"
        sleep 2
    done
}

# Function to watch news stream
watch_news() {
    echo -e "${GREEN}🔍 Watching News Stream...${NC}"
    echo -e "${BLUE}   File: live_data/news_stream.json${NC}"
    echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
    echo ""
    
    if [ ! -f "live_data/news_stream.json" ]; then
        echo -e "${RED}❌ News stream file not found.${NC}"
        echo -e "${YELLOW}   Make sure the simulated stream is running:${NC}"
        echo -e "${YELLOW}   ./scripts.sh stream start${NC}"
        return 1
    fi
    
    # Watch the news stream file
    while true; do
        clear
        echo -e "${GREEN}🔍 News Stream - $(date)${NC}"
        echo -e "${BLUE}================================${NC}"
        cat live_data/news_stream.json | jq '.' 2>/dev/null || cat live_data/news_stream.json
        echo ""
        echo -e "${YELLOW}🔄 Refreshing in 2 seconds... (Ctrl+C to stop)${NC}"
        sleep 2
    done
}

# Function to watch all streams
watch_all() {
    echo -e "${GREEN}🔍 Watching All Streams...${NC}"
    echo -e "${BLUE}   Files: All stream files${NC}"
    echo -e "${BLUE}   Press Ctrl+C to stop${NC}"
    echo ""
    
    # Check if any stream files exist
    if [ ! -f "live_data/crypto_stream.json" ] && [ ! -f "live_data/stocks_stream.json" ] && [ ! -f "live_data/options_stream.json" ] && [ ! -f "live_data/news_stream.json" ]; then
        echo -e "${RED}❌ No stream files found.${NC}"
        echo -e "${YELLOW}   Make sure the simulated stream is running:${NC}"
        echo -e "${YELLOW}   ./scripts.sh stream start${NC}"
        return 1
    fi
    
    # Watch all stream files
    while true; do
        clear
        echo -e "${GREEN}🔍 All Streams Overview - $(date)${NC}"
        echo -e "${BLUE}================================${NC}"
        
        # Show crypto stream
        if [ -f "live_data/crypto_stream.json" ]; then
            echo -e "${CYAN}📊 Crypto Stream:${NC}"
            cat live_data/crypto_stream.json | jq '.stream_type, .symbols, .last_update' 2>/dev/null || echo "   (File exists but may be empty)"
            echo ""
        fi
        
        # Show stocks stream
        if [ -f "live_data/stocks_stream.json" ]; then
            echo -e "${CYAN}📈 Stocks Stream:${NC}"
            cat live_data/stocks_stream.json | jq '.stream_type, .symbols, .last_update' 2>/dev/null || echo "   (File exists but may be empty)"
            echo ""
        fi
        
        # Show options stream
        if [ -f "live_data/options_stream.json" ]; then
            echo -e "${CYAN}🎯 Options Stream:${NC}"
            cat live_data/options_stream.json | jq '.stream_type, .symbols, .last_update' 2>/dev/null || echo "   (File exists but may be empty)"
            echo ""
        fi
        
        # Show news stream
        if [ -f "live_data/news_stream.json" ]; then
            echo -e "${CYAN}📰 News Stream:${NC}"
            cat live_data/news_stream.json | jq '.stream_type, .last_update' 2>/dev/null || echo "   (File exists but may be empty)"
            echo ""
        fi
        
        echo -e "${YELLOW}🔄 Refreshing in 3 seconds... (Ctrl+C to stop)${NC}"
        sleep 3
    done
}

# Function to check stream status
check_status() {
    echo -e "${GREEN}📊 Stream Status Check${NC}"
    echo -e "${BLUE}=====================${NC}"
    echo ""
    
    # Check if simulated stream is running
    if [ -f "simulated_stream.pid" ]; then
        PID=$(cat simulated_stream.pid)
        if kill -0 $PID 2>/dev/null; then
            echo -e "${GREEN}✅ Simulated stream is RUNNING${NC}"
            echo -e "${BLUE}   PID: $PID${NC}"
            echo -e "${BLUE}   Log: simulated_stream.log${NC}"
        else
            echo -e "${RED}❌ Stream not running (stale PID: $PID)${NC}"
            rm -f simulated_stream.pid
        fi
    else
        echo -e "${YELLOW}⚠️  Simulated stream not running${NC}"
        echo -e "${BLUE}   Start it with: ./scripts.sh stream start${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}📁 Stream Files Status:${NC}"
    
    # Check each stream file
    local files_found=0
    
    if [ -f "live_data/crypto_stream.json" ]; then
        echo -e "${GREEN}   ✅ crypto_stream.json${NC}"
        files_found=$((files_found + 1))
    else
        echo -e "${RED}   ❌ crypto_stream.json${NC}"
    fi
    
    if [ -f "live_data/stocks_stream.json" ]; then
        echo -e "${GREEN}   ✅ stocks_stream.json${NC}"
        files_found=$((files_found + 1))
    else
        echo -e "${RED}   ❌ stocks_stream.json${NC}"
    fi
    
    if [ -f "live_data/options_stream.json" ]; then
        echo -e "${GREEN}   ✅ options_stream.json${NC}"
        files_found=$((files_found + 1))
    else
        echo -e "${RED}   ❌ options_stream.json${NC}"
    fi
    
    if [ -f "live_data/news_stream.json" ]; then
        echo -e "${GREEN}   ✅ news_stream.json${NC}"
        files_found=$((files_found + 1))
    else
        echo -e "${RED}   ❌ news_stream.json${NC}"
    fi
    
    echo ""
    if [ $files_found -eq 4 ]; then
        echo -e "${GREEN}🎉 All stream files are present!${NC}"
    elif [ $files_found -gt 0 ]; then
        echo -e "${YELLOW}⚠️  Some stream files are missing${NC}"
    else
        echo -e "${RED}❌ No stream files found${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}💡 To start streaming: ./scripts.sh stream start${NC}"
}

# Main menu loop
while true; do
    show_streams
    read -p "Choose a stream to watch (0-6): " choice
    
    case $choice in
        1)
            watch_crypto
            ;;
        2)
            watch_stocks
            ;;
        3)
            watch_options
            ;;
        4)
            watch_news
            ;;
        5)
            watch_all
            ;;
        6)
            check_status
            echo ""
            read -p "Press Enter to continue..."
            ;;
        0)
            echo -e "${BLUE}👋 Goodbye!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Invalid choice. Please enter 0-6.${NC}"
            echo ""
            read -p "Press Enter to continue..."
            ;;
    esac
    
    # Clear screen after each operation
    clear
done
