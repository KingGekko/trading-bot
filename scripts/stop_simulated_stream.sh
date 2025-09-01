#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script header
echo -e "${BLUE}🛑 Trading Bot Simulated Stream Controller${NC}"
echo -e "${BLUE}==========================================${NC}"

# Function to stop the simulated stream
stop_stream() {
    if [ -f "simulated_stream.pid" ]; then
        PID=$(cat simulated_stream.pid)
        if kill -0 $PID 2>/dev/null; then
            echo -e "${YELLOW}🛑 Stopping simulated stream (PID: $PID)...${NC}"
            kill $PID
            
            # Wait a moment for graceful shutdown
            sleep 2
            
            # Check if it's still running
            if kill -0 $PID 2>/dev/null; then
                echo -e "${YELLOW}⚠️  Graceful shutdown failed, forcing termination...${NC}"
                kill -9 $PID
            fi
            
            rm -f simulated_stream.pid
            echo -e "${GREEN}✅ Stream stopped successfully${NC}"
        else
            echo -e "${YELLOW}⚠️  Stream not running (PID: $PID)${NC}"
            rm -f simulated_stream.pid
        fi
    else
        echo -e "${YELLOW}⚠️  No PID file found${NC}"
        echo -e "${BLUE}   The stream may not be running${NC}"
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
        else
            echo -e "${RED}❌ Stream not running (stale PID: $PID)${NC}"
            rm -f simulated_stream.pid
        fi
    else
        echo -e "${YELLOW}⚠️  Stream not running (no PID file)${NC}"
    fi
}

# Main script logic
case "${1:-stop}" in
    "stop")
        stop_stream
        ;;
    "status")
        show_status
        ;;
    "help"|"-h"|"--help")
        echo -e "${BLUE}Usage: $0 [stop|status|help]${NC}"
        echo ""
        echo -e "${BLUE}Commands:${NC}"
        echo -e "${BLUE}  stop    - Stop the simulated stream${NC}"
        echo -e "${BLUE}  status  - Show stream status${NC}"
        echo -e "${BLUE}  help    - Show this help message${NC}"
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo -e "${BLUE}Use: $0 help for usage information${NC}"
        exit 1
        ;;
esac
