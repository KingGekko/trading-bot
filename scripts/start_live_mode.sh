#!/bin/bash

# 🚀 Live Mode Startup Script
# This script starts the trading bot in live mode with automatic market data streaming

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CONFIG_FILE="config.env"
LIVE_DATA_DIR="live_data"
LOG_DIR="logs"
PID_FILE="live_mode.pid"

echo -e "${BLUE}🚀 Starting Trading Bot in Live Mode${NC}"
echo "============================================="
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if config file exists
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}❌ Configuration file $CONFIG_FILE not found${NC}"
        echo "Please create $CONFIG_FILE with your Alpaca API credentials"
        exit 1
    fi
    
    # Market data streaming is now handled by Rust
    echo -e "${GREEN}✅ Market data streaming integrated into Rust trading bot${NC}"
    
    # Check if Rust is available (try multiple locations)
    RUST_FOUND=false
    
    # Try standard PATH first
    if command -v cargo >/dev/null 2>&1; then
        RUST_FOUND=true
        echo -e "${GREEN}✅ Rust/Cargo found in PATH${NC}"
    # Try common Rust installation locations
    elif [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        RUST_FOUND=true
        echo -e "${GREEN}✅ Rust/Cargo found in ~/.cargo/bin${NC}"
    elif [ -f "/root/.cargo/bin/cargo" ]; then
        export PATH="/root/.cargo/bin:$PATH"
        RUST_FOUND=true
        echo -e "${GREEN}✅ Rust/Cargo found in /root/.cargo/bin${NC}"
    elif [ -f "/usr/local/cargo/bin/cargo" ]; then
        export PATH="/usr/local/cargo/bin:$PATH"
        RUST_FOUND=true
        echo -e "${GREEN}✅ Rust/Cargo found in /usr/local/cargo/bin${NC}"
    elif [ -f "/opt/rust/bin/cargo" ]; then
        export PATH="/opt/rust/bin:$PATH"
        RUST_FOUND=true
        echo -e "${GREEN}✅ Rust/Cargo found in /opt/rust/bin${NC}"
    fi
    
    if [ "$RUST_FOUND" = false ]; then
        echo -e "${RED}❌ Rust/Cargo not found in standard locations${NC}"
        echo "Checking for Rust installation..."
        
        # Try to find Rust anywhere in the system
        RUST_LOCATION=$(find / -name "cargo" -type f 2>/dev/null | head -1)
        if [ -n "$RUST_LOCATION" ]; then
            export PATH="$(dirname "$RUST_LOCATION"):$PATH"
            RUST_FOUND=true
            echo -e "${GREEN}✅ Rust/Cargo found at: $RUST_LOCATION${NC}"
        else
            echo -e "${RED}❌ Rust/Cargo not found anywhere in the system${NC}"
            echo "Please install Rust toolchain or check installation"
            exit 1
        fi
    fi
    
    # Verify cargo is now accessible
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}❌ Cargo still not accessible after PATH update${NC}"
        exit 1
    fi
    
    # Check if Ollama is running
    if ! curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ Ollama not running, starting it...${NC}"
        ollama serve &
        sleep 3
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
    echo ""
}

# Function to create directories
create_directories() {
    echo -e "${BLUE}📁 Creating necessary directories...${NC}"
    
    mkdir -p "$LIVE_DATA_DIR"
    mkdir -p "$LOG_DIR"
    
    echo -e "${GREEN}✅ Directories created${NC}"
    echo ""
}

# Function to build the project
build_project() {
    echo -e "${BLUE}🔨 Building trading bot project...${NC}"
    
    if [ ! -f "target/release/trading_bot" ]; then
        echo "Building release version..."
        cargo build --release
    else
        echo "Release binary already exists, skipping build"
    fi
    
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
}

# Function to start market data streaming
start_market_data_streaming() {
    echo -e "${BLUE}📡 Starting market data streaming...${NC}"
    
    # Market data streaming is now integrated into the Rust trading bot
    # No need to start separate Python process
    echo -e "${GREEN}✅ Market data streaming integrated into Rust trading bot${NC}"
    echo ""
}

# Function to start the trading bot server
start_trading_bot() {
    echo -e "${BLUE}🤖 Starting trading bot server...${NC}"
    
    # Start server in background
    ./target/release/trading_bot --api > "$LOG_DIR/trading_bot.log" 2>&1 &
    SERVER_PID=$!
    
    # Save server PID
    echo "$SERVER_PID" >> "$PID_FILE"
    
    echo -e "${GREEN}✅ Trading bot server started (PID: $SERVER_PID)${NC}"
    echo ""
}

# Function to wait for services to be ready
wait_for_services() {
    echo -e "${BLUE}⏳ Waiting for services to be ready...${NC}"
    
    # Wait for trading bot server
    echo "Waiting for trading bot server..."
    for i in {1..30}; do
        if curl -s "http://localhost:8080/health" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Trading bot server ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Trading bot server failed to start${NC}"
            exit 1
        fi
        echo -n "."
        sleep 1
    done
    
    # Wait for market data files
    echo "Waiting for market data files..."
    for i in {1..30}; do
        if [ -f "$LIVE_DATA_DIR/crypto_data_btc.json" ] && [ -f "$LIVE_DATA_DIR/crypto_data_eth.json" ] && \
           [ -f "$LIVE_DATA_DIR/stock_data_aapl.json" ] && [ -f "$LIVE_DATA_DIR/options_data_spy.json" ]; then
            echo -e "${GREEN}✅ Market data files ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${YELLOW}⚠️ Market data files not ready, continuing anyway${NC}"
            break
        fi
        echo -n "."
        sleep 1
    done
    
    echo ""
}

# Function to start file watching
start_file_watching() {
    echo -e "${BLUE}👀 Starting file watching for live data...${NC}"
    
    # Start watching Crypto data files
    curl -s -X POST "http://localhost:8080/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/crypto_data_btc.json\"}" >/dev/null 2>&1 || true
    
    curl -s -X POST "http://localhost:8080/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/crypto_data_eth.json\"}" >/dev/null 2>&1 || true
    
    # Start watching Stock data files
    curl -s -X POST "http://localhost:8080/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/stock_data_aapl.json\"}" >/dev/null 2>&1 || true
    
    # Start watching Options data files
    curl -s -X POST "http://localhost:8080/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/options_data_spy.json\"}" >/dev/null 2>&1 || true
    
    echo -e "${GREEN}✅ File watching started${NC}"
    echo ""
}

# Function to show status
show_status() {
    echo -e "${BLUE}📊 Live Mode Status${NC}"
    echo "=================="
    echo ""
    
    # Check server status
    if curl -s "http://localhost:8080/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Trading Bot Server: RUNNING${NC}"
    else
        echo -e "${RED}❌ Trading Bot Server: STOPPED${NC}"
    fi
    
    # Check market data files for all stream types
    echo -e "${BLUE}🔐 Crypto Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/crypto_data_btc.json" ]; then
        echo -e "${GREEN}✅ BTC: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ BTC: NOT AVAILABLE${NC}"
    fi
    if [ -f "$LIVE_DATA_DIR/crypto_data_eth.json" ]; then
        echo -e "${GREEN}✅ ETH: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ ETH: NOT AVAILABLE${NC}"
    fi
    
    echo -e "${BLUE}📈 Stock Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/stock_data_aapl.json" ]; then
        echo -e "${GREEN}✅ AAPL: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ AAPL: NOT AVAILABLE${NC}"
    fi
    
    echo -e "${BLUE}📊 Options Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/options_data_spy.json" ]; then
        echo -e "${GREEN}✅ SPY: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ SPY: NOT AVAILABLE${NC}"
    fi
    
    # Check watched files
    watched_files=$(curl -s "http://localhost:8080/api/files" 2>/dev/null || echo "{}")
    if echo "$watched_files" | grep -q "crypto_data\|stock_data\|options_data"; then
        echo -e "${GREEN}✅ File Watching: ACTIVE${NC}"
    else
        echo -e "${YELLOW}⚠️ File Watching: NOT ACTIVE${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}🌐 Trading Bot API: http://localhost:8080${NC}"
    echo -e "${BLUE}📁 Live Data Directory: $LIVE_DATA_DIR${NC}"
    echo -e "${BLUE}📋 Logs Directory: $LOG_DIR${NC}"
    echo -e "${BLUE}🆔 Process IDs: $PID_FILE${NC}"
    echo ""
}

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 Shutting down live mode...${NC}"
    
    # Stop processes
    if [ -f "$PID_FILE" ]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                echo "Stopping process $pid..."
                kill "$pid" 2>/dev/null || true
            fi
        done < "$PID_FILE"
        rm -f "$PID_FILE"
    fi
    
    echo -e "${GREEN}✅ Live mode stopped${NC}"
}

# Set trap for cleanup
trap cleanup EXIT

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Trading Bot in Live Mode${NC}"
    echo ""
    
    # Run all startup steps
    check_prerequisites
    create_directories
    build_project
    start_market_data_streaming
    start_trading_bot
    wait_for_services
    start_file_watching
    
    # Show final status
    show_status
    
    echo -e "${GREEN}🎉 Live mode startup complete!${NC}"
    echo ""
    echo -e "${BLUE}Press Ctrl+C to stop live mode${NC}"
    
    # Keep script running
    while true; do
        sleep 10
        # Show periodic status
        echo ""
        show_status
    done
}

# Run main function
main "$@"
