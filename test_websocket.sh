#!/bin/bash

# WebSocket Testing Script for Trading Bot
# This script tests the WebSocket streaming functionality after deployment

set -e

echo "🌐 WebSocket Testing Script for Trading Bot"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:8080"
WS_BASE_URL="ws://localhost:8080"
SAMPLE_FILE="./sample_data.json"

# Function to check if service is running
check_service() {
    echo -e "${BLUE}🔍 Checking if trading bot API is running...${NC}"
    
    if curl -s "$API_BASE_URL/health" > /dev/null; then
        echo -e "${GREEN}✅ Trading bot API is running${NC}"
        return 0
    else
        echo -e "${RED}❌ Trading bot API is not running${NC}"
        echo "Please start the API server first:"
        echo "  cargo run -- --api --api-port 8080"
        return 1
    fi
}

# Function to check if wscat is available
check_wscat() {
    echo -e "${BLUE}🔍 Checking if wscat is available...${NC}"
    
    if command -v wscat >/dev/null 2>&1; then
        echo -e "${GREEN}✅ wscat is available${NC}"
        wscat --version
        return 0
    else
        echo -e "${RED}❌ wscat is not available${NC}"
        echo "Installing wscat..."
        npm install -g wscat
        if command -v wscat >/dev/null 2>&1; then
            echo -e "${GREEN}✅ wscat installed successfully${NC}"
            return 0
        else
            echo -e "${RED}❌ Failed to install wscat${NC}"
            return 1
        fi
    fi
}

# Function to test file watching
test_file_watching() {
    echo -e "${BLUE}📁 Testing file watching functionality...${NC}"
    
    # Start watching the sample file
    echo "Starting to watch $SAMPLE_FILE..."
    curl -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$SAMPLE_FILE\"}"
    
    echo ""
    echo -e "${GREEN}✅ File watching started${NC}"
}

# Function to test WebSocket connection
test_websocket() {
    echo -e "${BLUE}🔌 Testing WebSocket connection...${NC}"
    
    local ws_url="$WS_BASE_URL/api/stream/$SAMPLE_FILE"
    echo "Connecting to WebSocket: $ws_url"
    echo ""
    echo -e "${YELLOW}📡 WebSocket test instructions:${NC}"
    echo "1. The connection will be established"
    echo "2. You'll see real-time updates when the file changes"
    echo "3. Press Ctrl+C to stop the test"
    echo ""
    echo -e "${BLUE}🔗 Connecting to WebSocket...${NC}"
    
    # Test WebSocket connection
    wscat -c "$ws_url" || {
        echo -e "${RED}❌ WebSocket connection failed${NC}"
        return 1
    }
}

# Function to test file modification
test_file_modification() {
    echo -e "${BLUE}✏️ Testing file modification...${NC}"
    
    echo "Modifying $SAMPLE_FILE to trigger WebSocket updates..."
    
    # Create a backup
    cp "$SAMPLE_FILE" "${SAMPLE_FILE}.backup"
    
    # Add a timestamp to trigger change
    echo "{\"test_timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\", \"test_data\": \"WebSocket test\"}" >> "$SAMPLE_FILE"
    
    echo -e "${GREEN}✅ File modified, WebSocket should show updates${NC}"
    
    # Wait a moment for the change to be processed
    sleep 2
    
    # Restore the original file
    mv "${SAMPLE_FILE}.backup" "$SAMPLE_FILE"
    echo -e "${GREEN}✅ File restored to original state${NC}"
}

# Function to run comprehensive test
run_comprehensive_test() {
    echo -e "${BLUE}🚀 Running comprehensive WebSocket test...${NC}"
    echo ""
    
    # Check prerequisites
    check_service || exit 1
    check_wscat || exit 1
    
    echo ""
    echo -e "${GREEN}✅ All prerequisites met${NC}"
    echo ""
    
    # Test file watching
    test_file_watching
    
    echo ""
    echo -e "${YELLOW}⏳ Waiting 3 seconds before WebSocket test...${NC}"
    sleep 3
    
    # Test WebSocket
    test_websocket
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  --check-only     Only check prerequisites"
    echo "  --watch-only     Only test file watching"
    echo "  --websocket-only Only test WebSocket connection"
    echo "  --comprehensive  Run full test suite (default)"
    echo "  --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run comprehensive test"
    echo "  $0 --check-only      # Check if everything is ready"
    echo "  $0 --websocket-only  # Test only WebSocket functionality"
}

# Main execution
main() {
    case "${1:---comprehensive}" in
        --check-only)
            check_service
            check_wscat
            ;;
        --watch-only)
            check_service || exit 1
            test_file_watching
            ;;
        --websocket-only)
            check_service || exit 1
            check_wscat || exit 1
            test_websocket
            ;;
        --comprehensive)
            run_comprehensive_test
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
