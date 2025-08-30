#!/bin/bash

# 🎮 Trading Bot Control Script
# This script provides unified control for starting/stopping the trading bot

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CONFIG_FILE="../live_config.env"
LIVE_DATA_DIR="../live_data"
LOG_DIR="../logs"
PID_FILE="../live_mode.pid"

# Function to show usage
show_usage() {
    echo -e "${BLUE}🎮 Trading Bot Control Script${NC}"
    echo "================================"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start-live    - Start trading bot in live mode"
    echo "  start-test    - Start trading bot in test mode"
    echo "  stop          - Stop the trading bot"
    echo "  status        - Show current status"
    echo "  restart       - Restart the trading bot"
    echo "  logs          - Show recent logs"
    echo "  help          - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start-live"
    echo "  $0 stop"
    echo "  $0 status"
    echo ""
}

# Function to check if bot is running
is_bot_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        else
            # Remove stale PID file
            rm -f "$PID_FILE"
            return 1
        fi
    fi
    return 1
}

# Function to start live mode
start_live_mode() {
    echo -e "${BLUE}🚀 Starting Trading Bot in Live Mode${NC}"
    echo "============================================="
    echo ""
    
    if is_bot_running; then
        echo -e "${YELLOW}⚠️  Trading bot is already running${NC}"
        return 1
    fi
    
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}❌ Configuration file $CONFIG_FILE not found${NC}"
        echo "Please create $CONFIG_FILE with your Alpaca API credentials"
        return 1
    fi
    
    echo -e "${YELLOW}🚀 Starting live mode...${NC}"
    bash start_live_mode.sh
    
    if is_bot_running; then
        echo -e "${GREEN}✅ Trading bot started successfully in live mode${NC}"
    else
        echo -e "${RED}❌ Failed to start trading bot${NC}"
        return 1
    fi
}

# Function to start test mode
start_test_mode() {
    echo -e "${BLUE}🧪 Starting Trading Bot in Test Mode${NC}"
    echo "============================================="
    echo ""
    
    if is_bot_running; then
        echo -e "${YELLOW}⚠️  Trading bot is already running${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}🧪 Starting test mode...${NC}"
    bash start_test_mode.sh
    
    if is_bot_running; then
        echo -e "${GREEN}✅ Trading bot started successfully in test mode${NC}"
    else
        echo -e "${RED}❌ Failed to start trading bot${NC}"
        return 1
    fi
}

# Function to stop bot
stop_bot() {
    echo -e "${BLUE}🛑 Stopping Trading Bot${NC}"
    echo "============================="
    echo ""
    
    if ! is_bot_running; then
        echo -e "${YELLOW}⚠️  Trading bot is not running${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🛑 Stopping trading bot...${NC}"
    bash stop_live_mode.sh
    
    if ! is_bot_running; then
        echo -e "${GREEN}✅ Trading bot stopped successfully${NC}"
    else
        echo -e "${RED}❌ Failed to stop trading bot${NC}"
        return 1
    fi
}

# Function to show status
show_status() {
    echo -e "${BLUE}📊 Trading Bot Status${NC}"
    echo "======================="
    echo ""
    
    if is_bot_running; then
        local pid=$(cat "$PID_FILE")
        echo -e "${GREEN}✅ Trading bot is RUNNING${NC}"
        echo "PID: $pid"
        echo "Started: $(ps -o lstart= -p "$pid" 2>/dev/null || echo "Unknown")"
        echo ""
        
        # Show data files
        if [ -d "$LIVE_DATA_DIR" ]; then
            echo -e "${BLUE}📁 Live Data Files:${NC}"
            for file in "$LIVE_DATA_DIR"/*.json; do
                if [ -f "$file" ]; then
                    filename=$(basename "$file")
                    size=$(ls -lh "$file" | awk '{print $5}')
                    modified=$(ls -lh "$file" | awk '{print $6, $7, $8}')
                    echo "  📄 $filename ($size) - Modified: $modified"
                fi
            done
        fi
    else
        echo -e "${RED}❌ Trading bot is NOT RUNNING${NC}"
    fi
    echo ""
}

# Function to restart bot
restart_bot() {
    echo -e "${BLUE}🔄 Restarting Trading Bot${NC}"
    echo "==============================="
    echo ""
    
    stop_bot
    sleep 2
    start_live_mode
}

# Function to show logs
show_logs() {
    echo -e "${BLUE}📋 Recent Logs${NC}"
    echo "==============="
    echo ""
    
    if [ -d "$LOG_DIR" ]; then
        for log_file in "$LOG_DIR"/*.log; do
            if [ -f "$log_file" ]; then
                echo -e "${BLUE}📄 $(basename "$log_file"):${NC}"
                tail -20 "$log_file" 2>/dev/null || echo "  (No recent logs)"
                echo ""
            fi
        done
    else
        echo -e "${YELLOW}⚠️  No log directory found${NC}"
    fi
}

# Main script logic
case "${1:-help}" in
    start-live)
        start_live_mode
        ;;
    start-test)
        start_test_mode
        ;;
    stop)
        stop_bot
        ;;
    status)
        show_status
        ;;
    restart)
        restart_bot
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_usage
        exit 1
        ;;
esac
