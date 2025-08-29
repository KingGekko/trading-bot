#!/bin/bash

# 🧪 Live Mode Testing Script
# This script tests the live mode functionality with real market data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:8080"
LIVE_DATA_DIR="live_data"
# Data files for each stream type
CRYPTO_BTC_FILE="crypto_data_btc.json"
CRYPTO_ETH_FILE="crypto_data_eth.json"
STOCK_AAPL_FILE="stock_data_aapl.json"
OPTIONS_SPY_FILE="options_data_spy.json"
NEWS_AAPL_FILE="news_data_aapl.json"
NEWS_SPY_FILE="news_data_spy.json"

echo -e "${BLUE}🧪 Live Mode Testing (Rust-Based Market Data)${NC}"
echo "=================================================="
echo ""

# Function to check if server is running
check_server() {
    echo -e "${BLUE}🔍 Checking server status...${NC}"
    
    if curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Server is running at $API_BASE_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ Server is not running at $API_BASE_URL${NC}"
        echo "Please start the server first with: ./start_live_mode.sh"
        return 1
    fi
}

# Function to check live data files
check_live_data_files() {
    echo -e "${BLUE}📁 Checking live data files for all stream types...${NC}"
    
    # Check Crypto data files
    echo -e "${BLUE}🔐 Checking Crypto Data Files:${NC}"
    if [ -f "$LIVE_DATA_DIR/$CRYPTO_BTC_FILE" ]; then
        echo -e "${GREEN}✅ BTC crypto data file found: $LIVE_DATA_DIR/$CRYPTO_BTC_FILE${NC}"
        echo "BTC Crypto Data Content:"
        cat "$LIVE_DATA_DIR/$CRYPTO_BTC_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$CRYPTO_BTC_FILE"
        echo ""
    else
        echo -e "${RED}❌ BTC crypto data file not found: $LIVE_DATA_DIR/$CRYPTO_BTC_FILE${NC}"
    fi
    
    if [ -f "$LIVE_DATA_DIR/$CRYPTO_ETH_FILE" ]; then
        echo -e "${GREEN}✅ ETH crypto data file found: $LIVE_DATA_DIR/$CRYPTO_ETH_FILE${NC}"
        echo "ETH Crypto Data Content:"
        cat "$LIVE_DATA_DIR/$CRYPTO_ETH_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$CRYPTO_ETH_FILE"
        echo ""
    else
        echo -e "${RED}❌ ETH crypto data file not found: $LIVE_DATA_DIR/$CRYPTO_ETH_FILE${NC}"
    fi
    
    # Check Stock data files
    echo -e "${BLUE}📈 Checking Stock Data Files:${NC}"
    if [ -f "$LIVE_DATA_DIR/$STOCK_AAPL_FILE" ]; then
        echo -e "${GREEN}✅ AAPL stock data file found: $LIVE_DATA_DIR/$STOCK_AAPL_FILE${NC}"
        echo "AAPL Stock Data Content:"
        cat "$LIVE_DATA_DIR/$STOCK_AAPL_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$STOCK_AAPL_FILE"
        echo ""
    else
        echo -e "${RED}❌ AAPL stock data file not found: $LIVE_DATA_DIR/$STOCK_AAPL_FILE${NC}"
    fi
    
    # Check Options data files
    echo -e "${BLUE}📊 Checking Options Data Files:${NC}"
    if [ -f "$LIVE_DATA_DIR/$OPTIONS_SPY_FILE" ]; then
        echo -e "${GREEN}✅ SPY options data file found: $LIVE_DATA_DIR/$OPTIONS_SPY_FILE${NC}"
        echo "SPY Options Data Content:"
        cat "$LIVE_DATA_DIR/$OPTIONS_SPY_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$OPTIONS_SPY_FILE"
        echo ""
    else
        echo -e "${RED}❌ SPY options data file not found: $LIVE_DATA_DIR/$OPTIONS_SPY_FILE${NC}"
    fi
    
    # Check News data files
    echo -e "${BLUE}📰 Checking News Data Files:${NC}"
    if [ -f "$LIVE_DATA_DIR/$NEWS_AAPL_FILE" ]; then
        echo -e "${GREEN}✅ AAPL news data file found: $LIVE_DATA_DIR/$NEWS_AAPL_FILE${NC}"
        echo "AAPL News Data Content:"
        cat "$LIVE_DATA_DIR/$NEWS_AAPL_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$NEWS_AAPL_FILE"
        echo ""
    else
        echo -e "${RED}❌ AAPL news data file not found: $LIVE_DATA_DIR/$NEWS_AAPL_FILE${NC}"
    fi
    
    if [ -f "$LIVE_DATA_DIR/$NEWS_SPY_FILE" ]; then
        echo -e "${GREEN}✅ SPY news data file found: $LIVE_DATA_DIR/$NEWS_SPY_FILE${NC}"
        echo "SPY News Data Content:"
        cat "$LIVE_DATA_DIR/$NEWS_SPY_FILE" | jq '.' 2>/dev/null || cat "$LIVE_DATA_DIR/$NEWS_SPY_FILE"
        echo ""
    else
        echo -e "${RED}❌ SPY news data file not found: $LIVE_DATA_DIR/$NEWS_SPY_FILE${NC}"
    fi
}

# Function to test file watching
test_file_watching() {
    echo -e "${BLUE}👀 Testing file watching for all stream types...${NC}"
    
    # Start watching Crypto files
    echo "Starting to watch BTC crypto file..."
    response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$CRYPTO_BTC_FILE\"}")
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ BTC crypto file watching started${NC}"
    else
        echo -e "${RED}❌ Failed to start BTC crypto file watching${NC}"
        echo "Response: $response"
    fi
    
    echo "Starting to watch ETH crypto file..."
    response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$CRYPTO_ETH_FILE\"}")
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ ETH crypto file watching started${NC}"
    else
        echo -e "${RED}❌ Failed to start ETH crypto file watching${NC}"
        echo "Response: $response"
    fi
    
    # Start watching Stock files
    echo "Starting to watch AAPL stock file..."
    response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$STOCK_AAPL_FILE\"}")
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ AAPL stock file watching started${NC}"
    else
        echo -e "${RED}❌ Failed to start AAPL stock file watching${NC}"
        echo "Response: $response"
    fi
    
    # Start watching Options files
    echo "Starting to watch SPY options file..."
    response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$OPTIONS_SPY_FILE\"}")
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ SPY options file watching started${NC}"
    else
        echo -e "${RED}❌ Failed to start SPY options file watching${NC}"
        echo "Response: $response"
    fi
    
    # Start watching News files
    echo "Starting to watch AAPL news file..."
    response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$NEWS_AAPL_FILE\"}")
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ AAPL news file watching started${NC}"
    else
        echo -e "${RED}❌ Failed to start AAPL news file watching${NC}"
        echo "Response: $response"
    fi
    
    # Check watched files
    echo "Checking watched files..."
    watched_files=$(curl -s "$API_BASE_URL/api/files")
    echo "Watched files: $watched_files"
    echo ""
}

# Function to test real-time streaming
test_streaming() {
    echo -e "${BLUE}📡 Testing real-time streaming...${NC}"
    
    echo "Testing WebSocket streaming for BTC data..."
    echo "WebSocket URL: ws://localhost:8080/api/stream/$LIVE_DATA_DIR/$BTC_FILE"
    echo ""
    echo "To test WebSocket streaming, use wscat:"
    echo "wscat -c ws://localhost:8080/api/stream/$LIVE_DATA_DIR/$BTC_FILE"
    echo ""
    
    echo "Testing WebSocket streaming for ETH data..."
    echo "WebSocket URL: ws://localhost:8080/api/stream/$LIVE_DATA_DIR/$ETH_FILE"
    echo ""
    echo "To test WebSocket streaming, use wscat:"
    echo "wscat -c ws://localhost:8080/api/stream/$LIVE_DATA_DIR/$ETH_FILE"
    echo ""
}

# Function to test AI processing with live data
test_ai_processing() {
    echo -e "${BLUE}🤖 Testing AI processing with live data...${NC}"
    
    # Test basic processing with BTC data
    echo "Testing AI processing with BTC data..."
    response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$BTC_FILE\", \"prompt\": \"Analyze this Bitcoin market data and provide insights\"}")
    
    if [ -n "$response" ] && [ "$response" != "null" ]; then
        echo -e "${GREEN}✅ AI processing successful${NC}"
        echo "Response: $response"
    else
        echo -e "${RED}❌ AI processing failed${NC}"
        echo "Response: $response"
    fi
    
    echo ""
    
    # Test threaded processing with ETH data
    echo "Testing threaded AI processing with ETH data..."
    response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process/threaded" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$LIVE_DATA_DIR/$ETH_FILE\", \"prompt\": \"What are the key metrics for Ethereum?\"}")
    
    if [ -n "$response" ] && [ "$response" != "null" ]; then
        echo -e "${GREEN}✅ Threaded AI processing successful${NC}"
        echo "Response: $response"
    else
        echo -e "${RED}❌ Threaded AI processing failed${NC}"
        echo "Response: $response"
    fi
    
    echo ""
}

# Function to test file content endpoints
test_content_endpoints() {
    echo -e "${BLUE}📄 Testing content endpoints...${NC}"
    
    # Test BTC content
    echo "Testing BTC content endpoint..."
    btc_content=$(curl -s "$API_BASE_URL/api/content/$LIVE_DATA_DIR/$BTC_FILE")
    if [ -n "$btc_content" ] && [ "$btc_content" != "null" ]; then
        echo -e "${GREEN}✅ BTC content retrieved successfully${NC}"
    else
        echo -e "${RED}❌ Failed to retrieve BTC content${NC}"
    fi
    
    # Test ETH content
    echo "Testing ETH content endpoint..."
    eth_content=$(curl -s "$API_BASE_URL/api/content/$LIVE_DATA_DIR/$ETH_FILE")
    if [ -n "$eth_content" ] && [ "$eth_content" != "null" ]; then
        echo -e "${GREEN}✅ ETH content retrieved successfully${NC}"
    else
        echo -e "${RED}❌ Failed to retrieve ETH content${NC}"
    fi
    
    echo ""
}

# Function to show live mode status
show_live_mode_status() {
    echo -e "${BLUE}📊 Live Mode Status${NC}"
    echo "=================="
    echo ""
    
    # Check server status
    if curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Trading Bot Server: RUNNING${NC}"
    else
        echo -e "${RED}❌ Trading Bot Server: STOPPED${NC}"
    fi
    
    # Check market data files for all stream types
    echo -e "${BLUE}🔐 Crypto Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/$CRYPTO_BTC_FILE" ]; then
        echo -e "${GREEN}✅ BTC: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ BTC: NOT AVAILABLE${NC}"
    fi
    if [ -f "$LIVE_DATA_DIR/$CRYPTO_ETH_FILE" ]; then
        echo -e "${GREEN}✅ ETH: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ ETH: NOT AVAILABLE${NC}"
    fi
    
    echo -e "${BLUE}📈 Stock Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/$STOCK_AAPL_FILE" ]; then
        echo -e "${GREEN}✅ AAPL: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ AAPL: NOT AVAILABLE${NC}"
    fi
    
    echo -e "${BLUE}📊 Options Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/$OPTIONS_SPY_FILE" ]; then
        echo -e "${GREEN}✅ SPY: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ SPY: NOT AVAILABLE${NC}"
    fi
    
    echo -e "${BLUE}📰 News Data:${NC}"
    if [ -f "$LIVE_DATA_DIR/$NEWS_AAPL_FILE" ]; then
        echo -e "${GREEN}✅ AAPL: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ AAPL: NOT AVAILABLE${NC}"
    fi
    if [ -f "$LIVE_DATA_DIR/$NEWS_SPY_FILE" ]; then
        echo -e "${GREEN}✅ SPY: AVAILABLE${NC}"
    else
        echo -e "${RED}❌ SPY: NOT AVAILABLE${NC}"
    fi
    
    # Check watched files
    watched_files=$(curl -s "$API_BASE_URL/api/files" 2>/dev/null || echo "{}")
    if echo "$watched_files" | grep -q "crypto_data\|stock_data\|options_data"; then
        echo -e "${GREEN}✅ File Watching: ACTIVE${NC}"
    else
        echo -e "${YELLOW}⚠️ File Watching: NOT ACTIVE${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}🌐 Trading Bot API: $API_BASE_URL${NC}"
    echo -e "${BLUE}📁 Live Data Directory: $LIVE_DATA_DIR${NC}"
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🧪 Live Mode Testing${NC}"
    echo ""
    
    # Check server first
    if ! check_server; then
        exit 1
    fi
    
    echo ""
    
    # Run all tests
    check_live_data_files
    test_file_watching
    test_content_endpoints
    test_ai_processing
    test_streaming
    
    # Show final status
    show_live_mode_status
    
    echo -e "${GREEN}🎉 Live mode testing complete!${NC}"
    echo ""
    echo -e "${BLUE}💡 Next steps:${NC}"
    echo "1. Use wscat to test WebSocket streaming"
    echo "2. Monitor live data files for updates"
    echo "3. Test AI processing with different prompts"
    echo "4. Check logs in logs/ directory"
}

# Run main function
main "$@"
