#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script header
echo -e "${BLUE}🚀 Trading Bot Simulated Real-Time Stream${NC}"
echo -e "${BLUE}==========================================${NC}"

# Configuration file - detect if running from scripts/ or root directory
if [ -f "../config.env" ]; then
    CONFIG_FILE="../config.env"
elif [ -f "config.env" ]; then
    CONFIG_FILE="config.env"
else
    echo -e "${RED}❌ config.env not found. Please run from project root or scripts directory.${NC}"
    exit 1
fi

# Load configuration - export environment variables
set -a  # automatically export all variables
source "$CONFIG_FILE"
set +a  # stop automatically exporting

# Check if we're in paper trading mode
if [ "$OPERATION_MODE" != "paper" ]; then
    echo -e "${YELLOW}⚠️  Warning: Not in PAPER TRADING mode. Current mode: $OPERATION_MODE${NC}"
    echo -e "${YELLOW}   This script is designed for PAPER TRADING mode with simulated data.${NC}"
    read -p "Continue anyway? (y/n): " continue_anyway
    if [ "$continue_anyway" != "y" ]; then
        echo -e "${BLUE}👋 Exiting. Switch to PAPER TRADING mode first with: ./scripts.sh mode paper${NC}"
        exit 0
    fi
fi

# Function to check if stream is already running
check_stream_status() {
    if [ -f "simulated_stream.pid" ]; then
        PID=$(cat simulated_stream.pid)
        if kill -0 $PID 2>/dev/null; then
            return 0  # Running
        else
            return 1  # Not running
        fi
    else
        return 1  # Not running
    fi
}

# Function to start the simulated stream
start_stream() {
    echo -e "${GREEN}🔄 Starting paper trading real-time market data stream...${NC}"
    echo -e "${BLUE}   Mode: PAPER TRADING (real Alpaca data + simulated fallback)${NC}"
    echo -e "${BLUE}   Update Interval: 2 seconds${NC}"
    echo -e "${BLUE}   Symbols: BTC/USD, ETH/USD, AAPL, SPY${NC}"
    echo -e "${BLUE}   Data Types: Price, Volume, Options, News${NC}"
    
    # Start the Rust application with paper trading stream
    echo -e "${YELLOW}   Starting Rust application in paper trading mode...${NC}"
    
    # Run in background with --simulated flag and save PID
    nohup cargo run -- --simulated > simulated_stream.log 2>&1 &
    STREAM_PID=$!
    
    # Save PID to file for later use
    echo $STREAM_PID > simulated_stream.pid
    
    echo -e "${GREEN}✅ Simulated stream started with PID: $STREAM_PID${NC}"
    echo -e "${BLUE}   Log file: simulated_stream.log${NC}"
    echo -e "${BLUE}   PID file: simulated_stream.pid${NC}"
    
    # Wait a moment for startup
    sleep 3
    
    # Check if it's running
    if check_stream_status; then
        echo -e "${GREEN}✅ Stream is running successfully!${NC}"
        echo ""
        echo -e "${BLUE}📊 Monitor the stream:${NC}"
        echo -e "${BLUE}   watch -n 1 cat live_data/crypto_data_btc.json${NC}"
        echo -e "${BLUE}   watch -n 1 cat live_data/stock_data_aapl.json${NC}"
        echo -e "${BLUE}   watch -n 1 cat live_data/options_data_spy.json${NC}"
        echo ""
        echo -e "${BLUE}🛑 Stop the stream:${NC}"
        echo -e "${BLUE}   ./scripts/stop_simulated_stream.sh${NC}"
    else
        echo -e "${RED}❌ Failed to start stream. Check logs: simulated_stream.log${NC}"
        exit 1
    fi
}

# Function to stop the stream
stop_stream() {
    if [ -f "simulated_stream.pid" ]; then
        PID=$(cat simulated_stream.pid)
        if kill -0 $PID 2>/dev/null; then
            echo -e "${YELLOW}🛑 Stopping simulated stream (PID: $PID)...${NC}"
            kill $PID
            rm -f simulated_stream.pid
            echo -e "${GREEN}✅ Stream stopped${NC}"
        else
            echo -e "${YELLOW}⚠️  Stream not running (PID: $PID)${NC}"
            rm -f simulated_stream.pid
        fi
    else
        echo -e "${YELLOW}⚠️  No PID file found${NC}"
    fi
}

# Function to show stream status
show_status() {
    if [ -f "simulated_stream.pid" ]; then
        PID=$(cat simulated_stream.pid)
        if kill -0 $PID 2>/dev/null; then
            echo -e "${GREEN}✅ Simulated stream is RUNNING${NC}"
            echo -e "${BLUE}   PID: $PID${NC}"
            echo -e "${BLUE}   Log: simulated_stream.log${NC}"
            
            # Show recent log entries
            echo -e "${BLUE}   Recent logs:${NC}"
            tail -5 simulated_stream.log 2>/dev/null || echo "   No logs yet"
        else
            echo -e "${RED}❌ Stream not running (stale PID: $PID)${NC}"
            rm -f simulated_stream.pid
        fi
    else
        echo -e "${YELLOW}⚠️  Stream not running (no PID file)${NC}"
    fi
}

# Function to show help
show_help() {
    echo -e "${BLUE}Usage: $0 [start|stop|status|help]${NC}"
    echo ""
    echo -e "${BLUE}Commands:${NC}"
    echo -e "${BLUE}  start   - Start the simulated real-time stream${NC}"
    echo -e "${BLUE}  stop    - Stop the simulated stream${NC}"
    echo -e "${BLUE}  status  - Show stream status${NC}"
    echo -e "${BLUE}  help    - Show this help message${NC}"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo -e "${BLUE}  $0 start    # Start the stream${NC}"
    echo -e "${BLUE}  $0 stop     # Stop the stream${NC}"
    echo -e "${BLUE}  $0 status   # Check status${NC}"
}

# Main script logic
case "${1:-start}" in
    "start")
        if check_stream_status; then
            echo -e "${YELLOW}⚠️  Stream is already running${NC}"
            show_status
        else
            start_stream
        fi
        ;;
    "stop")
        stop_stream
        ;;
    "status")
        show_status
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac
