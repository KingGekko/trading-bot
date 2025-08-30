#!/bin/bash

# 🛑 Stop Live Mode Script
# This script stops the trading bot live mode and cleans up processes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PID_FILE="live_mode.pid"
LOG_DIR="logs"

echo -e "${BLUE}🛑 Stopping Trading Bot Live Mode${NC}"
echo "========================================="
echo ""

# Function to stop processes
stop_processes() {
    echo -e "${BLUE}🔄 Stopping processes...${NC}"
    
    if [ -f "$PID_FILE" ]; then
        echo "Found PID file: $PID_FILE"
        
        while read -r pid; do
            if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
                echo "Stopping process $pid..."
                kill "$pid" 2>/dev/null || true
                
                # Wait for process to stop
                for i in {1..10}; do
                    if ! kill -0 "$pid" 2>/dev/null; then
                        echo -e "${GREEN}✅ Process $pid stopped${NC}"
                        break
                    fi
                    sleep 1
                done
                
                # Force kill if still running
                if kill -0 "$pid" 2>/dev/null; then
                    echo -e "${YELLOW}⚠️ Force killing process $pid...${NC}"
                    kill -9 "$pid" 2>/dev/null || true
                fi
            else
                echo "Process $pid not running or invalid"
            fi
        done < "$PID_FILE"
        
        # Remove PID file
        rm -f "$PID_FILE"
        echo -e "${GREEN}✅ PID file removed${NC}"
    else
        echo -e "${YELLOW}⚠️ No PID file found${NC}"
    fi
    
    echo ""
}

# Function to check for remaining processes
check_remaining_processes() {
    echo -e "${BLUE}🔍 Checking for remaining processes...${NC}"
    
    # Check for trading bot processes
    trading_bot_pids=$(pgrep -f "trading_bot" 2>/dev/null || true)
    if [ -n "$trading_bot_pids" ]; then
        echo -e "${YELLOW}⚠️ Found remaining trading bot processes:${NC}"
        echo "$trading_bot_pids"
        echo "Stopping them..."
        echo "$trading_bot_pids" | xargs kill -9 2>/dev/null || true
    else
        echo -e "${GREEN}✅ No remaining trading bot processes${NC}"
    fi
    
    # Market data streaming is now integrated into Rust trading bot
    echo -e "${GREEN}✅ Market data streaming integrated into Rust trading bot${NC}"
    
    echo ""
}

# Function to check server status
check_server_status() {
    echo -e "${BLUE}🌐 Checking server status...${NC}"
    
    if curl -s "http://localhost:8080/health" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ Server still responding, may need manual stop${NC}"
        echo "You can manually stop it with: pkill -f 'trading_bot --api'"
    else
        echo -e "${GREEN}✅ Server stopped${NC}"
    fi
    
    echo ""
}

# Function to show cleanup summary
show_cleanup_summary() {
    echo -e "${BLUE}📊 Cleanup Summary${NC}"
    echo "=================="
    echo ""
    
    # Check if processes are still running
    trading_bot_running=$(pgrep -f "trading_bot" 2>/dev/null | wc -l)
    
    if [ "$trading_bot_running" -eq 0 ]; then
        echo -e "${GREEN}✅ All processes stopped successfully${NC}"
    else
        echo -e "${YELLOW}⚠️ Some processes may still be running${NC}"
        echo "Trading bot processes: $trading_bot_running"
    fi
    
    # Check if PID file is removed
    if [ ! -f "$PID_FILE" ]; then
        echo -e "${GREEN}✅ PID file cleaned up${NC}"
    else
        echo -e "${RED}❌ PID file still exists${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}📁 Log files are available in: $LOG_DIR${NC}"
    echo -e "${BLUE}🔄 To restart live mode: ./start_live_mode.sh${NC}"
    echo -e "${BLUE}🧪 To start test mode: ./start_test_mode.sh${NC}"
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🛑 Stopping Trading Bot Live Mode${NC}"
    echo ""
    
    # Stop all processes
    stop_processes
    
    # Check for remaining processes
    check_remaining_processes
    
    # Check server status
    check_server_status
    
    # Show cleanup summary
    show_cleanup_summary
    
    echo -e "${GREEN}🎉 Live mode stopped successfully!${NC}"
}

# Run main function
main "$@"
