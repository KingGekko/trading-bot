#!/bin/bash

# 🚀 Real JSON Streaming Test
# Tests the actual JSON streaming functionality, not just Ollama API calls

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:8080"
SAMPLE_DATA_FILE="sample_data.json"
TEST_FILE="test_streaming.json"

echo -e "${BLUE}🚀 Real JSON Streaming Test${NC}"
echo "================================="
echo "Testing actual JSON streaming functionality"
echo ""

# Function to check if API is running
check_api_health() {
    echo -e "${BLUE}🔍 Checking API health...${NC}"
    
    if curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ API is running at $API_BASE_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ API is not running at $API_BASE_URL${NC}"
        echo "Please start the API server first:"
        echo "  cargo run -- --api"
        return 1
    fi
}

# Function to create test data file
create_test_file() {
    echo -e "${BLUE}📁 Creating test data file...${NC}"
    
    cat > "$TEST_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "test": "streaming_test",
  "value": 100,
  "status": "initial"
}
EOF
    
    echo -e "${GREEN}✅ Created test file: $TEST_FILE${NC}"
    echo "Content:"
    cat "$TEST_FILE" | jq '.' 2>/dev/null || cat "$TEST_FILE"
    echo ""
}

# Function to test file watching
test_file_watching() {
    echo -e "${BLUE}🧪 Test 1: File Watching System${NC}"
    echo "Testing if the API can watch files for changes..."
    echo ""
    
    # Start watching the test file
    echo "📡 Starting file watch..."
    watch_response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$TEST_FILE\"}" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$watch_response" ]; then
        echo -e "${GREEN}✅ File watch started${NC}"
        echo "Response: $watch_response"
    else
        echo -e "${RED}❌ Failed to start file watch${NC}"
        echo "This indicates the file watching system is broken"
        return 1
    fi
    
    # Check watched files
    echo ""
    echo "📋 Checking watched files..."
    watched_files=$(curl -s "$API_BASE_URL/api/files" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$watched_files" ]; then
        echo -e "${GREEN}✅ Watched files retrieved${NC}"
        echo "Response: $watched_files"
    else
        echo -e "${RED}❌ Failed to get watched files${NC}"
        echo "File watching system may be broken"
        return 1
    fi
    
    echo ""
}

# Function to test WebSocket streaming
test_websocket_streaming() {
    echo -e "${BLUE}🧪 Test 2: WebSocket Streaming${NC}"
    echo "Testing WebSocket endpoint for real-time updates..."
    echo ""
    
    # Check if wscat is available
    if ! command -v wscat >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ wscat not available, skipping WebSocket test${NC}"
        echo "Install with: npm install -g wscat@5.1.1"
        return 0
    fi
    
    echo "🔌 Testing WebSocket connection..."
    echo "WebSocket URL: ws://localhost:8080/api/stream/$TEST_FILE"
    
    # Test WebSocket connection (non-blocking)
    timeout 5s wscat -c "ws://localhost:8080/api/stream/$TEST_FILE" 2>/dev/null || {
        echo -e "${RED}❌ WebSocket connection failed${NC}"
        echo "This indicates the streaming system is broken"
        return 1
    }
    
    echo -e "${GREEN}✅ WebSocket connection successful${NC}"
    echo ""
}

# Function to test real-time updates
test_realtime_updates() {
    echo -e "${BLUE}🧪 Test 3: Real-Time File Updates${NC}"
    echo "Testing if file changes trigger real-time updates..."
    echo ""
    
    # Update the test file
    echo "📝 Updating test file..."
    cat > "$TEST_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "test": "streaming_test",
  "value": 200,
  "status": "updated",
  "change": "real_time_update"
}
EOF
    
    echo -e "${GREEN}✅ File updated${NC}"
    echo "New content:"
    cat "$TEST_FILE" | jq '.' 2>/dev/null || cat "$TEST_FILE"
    echo ""
    
    # Wait a moment for the system to detect changes
    echo "⏳ Waiting for system to detect changes..."
    sleep 2
    
    # Check if the API detected the change
    echo "🔍 Checking if API detected the change..."
    current_content=$(curl -s "$API_BASE_URL/api/content/$TEST_FILE" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$current_content" ]; then
        echo -e "${GREEN}✅ API detected file change${NC}"
        echo "Current content from API:"
        echo "$current_content" | jq '.' 2>/dev/null || echo "$current_content"
    else
        echo -e "${RED}❌ API did not detect file change${NC}"
        echo "Real-time monitoring system may be broken"
        return 1
    fi
    
    echo ""
}

# Function to test Ollama integration with streaming
test_ollama_streaming() {
    echo -e "${BLUE}🧪 Test 4: Ollama Integration with Streaming${NC}"
    echo "Testing if Ollama can process streaming data..."
    echo ""
    
    # Process the updated file with Ollama
    echo "🤖 Processing updated file with Ollama..."
    ollama_response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d "{
            \"file_path\": \"$TEST_FILE\",
            \"prompt\": \"Analyze this real-time trading data update and provide insights about the changes.\"
        }" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$ollama_response" ]; then
        echo -e "${GREEN}✅ Ollama processing successful${NC}"
        echo "Response:"
        echo "$ollama_response" | jq '.' 2>/dev/null || echo "$ollama_response"
    else
        echo -e "${RED}❌ Ollama processing failed${NC}"
        echo "Integration between streaming and Ollama may be broken"
        return 1
    fi
    
    echo ""
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    
    # Stop watching the test file
    echo "🛑 Stopping file watch..."
    curl -s -X GET "$API_BASE_URL/api/watch/$TEST_FILE" >/dev/null 2>&1 || true
    
    # Remove test file
    if [ -f "$TEST_FILE" ]; then
        rm -f "$TEST_FILE"
        echo -e "${GREEN}✅ Removed test file${NC}"
    fi
    
    echo ""
}

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 JSON Streaming Test Summary${NC}"
    echo "====================================="
    echo ""
    
    echo "💡 What we tested:"
    echo "  1. File watching system - Can API monitor files?"
    echo "  2. WebSocket streaming - Are real-time connections working?"
    echo "  3. Real-time updates - Does system detect file changes?"
    echo "  4. Ollama integration - Can AI process streaming data?"
    echo ""
    
    echo "🚨 If any tests failed, the JSON streaming system is broken and needs fixing."
    echo "✅ If all tests passed, the streaming system is working correctly."
    echo ""
    
    echo "🔧 Next steps:"
    echo "  - Fix any broken components identified above"
    echo "  - Test with actual trading data files"
    echo "  - Implement real-time data feeds"
    echo "  - Set up continuous monitoring"
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Real JSON Streaming Test${NC}"
    echo ""
    
    # Check prerequisites
    check_api_health || exit 1
    echo ""
    
    # Create test environment
    create_test_file
    
    # Run tests
    test_file_watching
    test_websocket_streaming
    test_realtime_updates
    test_ollama_streaming
    
    # Cleanup
    cleanup
    
    # Show summary
    show_summary
}

# Run main function
main "$@"
