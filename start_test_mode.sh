#!/bin/bash

# 🧪 Test Mode Startup Script
# This script starts the trading bot in test mode for development and testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LOG_DIR="logs"
PID_FILE="test_mode.pid"

echo -e "${BLUE}🧪 Starting Trading Bot in Test Mode${NC}"
echo "=========================================="
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if Rust is available
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}❌ Rust/Cargo not found${NC}"
        echo "Please install Rust toolchain"
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
    
    mkdir -p "$LOG_DIR"
    
    echo -e "${GREEN}✅ Directories created${NC}"
    echo ""
}

# Function to build the project
build_project() {
    echo -e "${BLUE}🔨 Building trading bot project...${NC}"
    
    if [ ! -f "target/debug/trading_bot" ]; then
        echo "Building debug version..."
        cargo build
    else
        echo "Debug binary already exists, skipping build"
    fi
    
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
}

# Function to start the trading bot server
start_trading_bot() {
    echo -e "${BLUE}🤖 Starting trading bot server in test mode...${NC}"
    
    # Start server in foreground (for testing)
    echo "Starting server with debug logging..."
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    # Start server and capture output
    ./target/debug/trading_bot --api 2>&1 | tee "$LOG_DIR/test_server.log"
}

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 Shutting down test mode...${NC}"
    
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
    
    echo -e "${GREEN}✅ Test mode stopped${NC}"
}

# Set trap for cleanup
trap cleanup EXIT

# Main execution
main() {
    echo -e "${BLUE}🧪 Starting Trading Bot in Test Mode${NC}"
    echo ""
    
    # Run all startup steps
    check_prerequisites
    create_directories
    build_project
    
    echo -e "${GREEN}🎉 Test mode startup complete!${NC}"
    echo ""
    echo -e "${BLUE}🌐 Trading Bot API: http://localhost:8080${NC}"
    echo -e "${BLUE}📋 Logs Directory: $LOG_DIR${NC}"
    echo -e "${BLUE}🔍 Debug Mode: Enabled${NC}"
    echo ""
    echo -e "${BLUE}Press Ctrl+C to stop the server${NC}"
    echo ""
    
    # Start the server
    start_trading_bot
}

# Run main function
main "$@"
