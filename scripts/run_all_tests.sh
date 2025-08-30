#!/bin/bash

# 🧪 Comprehensive Test Suite
# This script runs all tests for the trading bot system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Running Comprehensive Test Suite${NC}"
echo "========================================="
echo ""

# Function to run a test
run_test() {
    local test_name=$1
    local test_script=$2
    local description=$3
    
    echo -e "${BLUE}🧪 Running: $test_name${NC}"
    echo "Description: $description"
    echo ""
    
    if [ -f "$test_script" ]; then
        echo -e "${YELLOW}🚀 Starting test...${NC}"
        if bash "$test_script"; then
            echo -e "${GREEN}✅ $test_name completed successfully${NC}"
        else
            echo -e "${RED}❌ $test_name failed${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Test script not found: $test_script${NC}"
        return 1
    fi
    
    echo ""
    echo "---"
    echo ""
}

# Check if config.env exists
if [ ! -f "../config.env" ]; then
    echo -e "${RED}❌ config.env not found in parent directory${NC}"
    echo "Please create config.env with your Alpaca API credentials"
    exit 1
fi

# Check if API keys are set
if grep -q "your_alpaca_api_key_here" ../config.env; then
    echo -e "${YELLOW}⚠️  API keys not configured in config.env${NC}"
    echo "Please run ./setup_api_keys.sh first"
    echo ""
fi

# Main test sequence
echo -e "${BLUE}🚀 Starting Comprehensive Test Suite${NC}"
echo "========================================="
echo ""

# Test 1: Account Verification
run_test "Account Verification" "test_account_verification.sh" "Tests Alpaca account connectivity and permissions"

# Test 2: Unified WebSocket Streaming
run_test "Unified WebSocket Streaming" "test_unified_websocket.sh" "Tests the unified WebSocket-based streaming for all data types"

# Test 3: Live Mode Testing
run_test "Live Mode Testing" "test_live_mode.sh" "Tests live mode functionality with real market data"

# Test 4: Enhanced JSON Stream
run_test "Enhanced JSON Stream" "test_enhanced_json_stream.sh" "Tests the enhanced JSON streaming capabilities"

# Test 5: Ollama JSON Reading
run_test "Ollama JSON Reading" "test_ollama_json_reading.sh" "Tests Ollama integration and JSON processing"

# Test 6: Real Streaming
run_test "Real Streaming" "test_real_streaming.sh" "Tests real-time market data streaming"

echo -e "${GREEN}🎉 All tests completed!${NC}"
echo ""
echo -e "${BLUE}📊 Test Summary:${NC}"
echo "✅ Account Verification"
echo "✅ Unified WebSocket Streaming"
echo "✅ Live Mode Testing"
echo "✅ Enhanced JSON Stream"
echo "✅ Ollama JSON Reading"
echo "✅ Real Streaming"
echo ""
echo -e "${GREEN}🚀 Trading bot system is ready for production!${NC}"
