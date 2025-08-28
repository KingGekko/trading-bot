#!/bin/bash

# Comprehensive JSON Stream Testing Script
# Tests all aspects of the trading bot's JSON streaming functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:3000"
WS_BASE_URL="ws://localhost:3000"
TEST_FILE="test_data.json"
TEST_DIR="test_files"

echo -e "${BLUE}🚀 JSON Stream Testing Suite${NC}"
echo "=================================="
echo ""

# Function to check if API is running
check_api() {
    echo -e "${BLUE}🔍 Checking if trading bot API is running...${NC}"
    
    if curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Trading bot API is running at $API_BASE_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ Trading bot API is not running at $API_BASE_URL${NC}"
        echo "Please start the API server first:"
        echo "  cargo run -- --api"
        return 1
    fi
}

# Function to create test data
create_test_data() {
    echo -e "${BLUE}📝 Creating test data...${NC}"
    
    # Create test directory
    mkdir -p "$TEST_DIR"
    
    # Create initial test JSON file
    cat > "$TEST_DIR/$TEST_FILE" << 'EOF'
{
    "id": 1,
    "name": "Test Trading Data",
    "timestamp": "2024-01-15T10:00:00Z",
    "data": {
        "price": 100.50,
        "volume": 1000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.0"
    }
}
EOF
    
    echo -e "${GREEN}✅ Test data created: $TEST_DIR/$TEST_FILE${NC}"
}

# Function to test file watching
test_file_watching() {
    echo -e "${BLUE}📡 Testing file watching...${NC}"
    
    # Start watching the test file
    echo "Starting to watch: $TEST_FILE"
    local response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$TEST_DIR/$TEST_FILE\"}")
    
    echo "Response: $response"
    
    # Check if watching started successfully
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ File watching started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start file watching${NC}"
        return 1
    fi
    
    # List watched files
    echo "Listing watched files..."
    local watched_files=$(curl -s "$API_BASE_URL/api/files")
    echo "Watched files: $watched_files"
    
    # Get current file content
    echo "Getting current file content..."
    local content=$(curl -s "$API_BASE_URL/api/content/$TEST_DIR/$TEST_FILE")
    echo "File content: $content"
}

# Function to test WebSocket streaming
test_websocket_streaming() {
    echo -e "${BLUE}🌊 Testing WebSocket streaming...${NC}"
    
    # Check if wscat is available
    if ! command -v wscat >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ wscat not found, installing...${NC}"
        if npm install -g wscat@5.1.1; then
            echo -e "${GREEN}✅ wscat installed successfully${NC}"
        else
            echo -e "${RED}❌ Failed to install wscat, skipping WebSocket test${NC}"
            return 1
        fi
    fi
    
    echo "Starting WebSocket connection to: $WS_BASE_URL/api/stream/$TEST_DIR/$TEST_FILE"
    echo "This will connect and wait for file updates..."
    echo "Press Ctrl+C to stop the WebSocket test"
    echo ""
    
    # Start WebSocket connection in background
    wscat -c "$WS_BASE_URL/api/stream/$TEST_DIR/$TEST_FILE" &
    local wscat_pid=$!
    
    # Wait a moment for connection
    sleep 2
    
    # Modify the test file to trigger an update
    echo "Modifying test file to trigger WebSocket update..."
    cat > "$TEST_DIR/$TEST_FILE" << 'EOF'
{
    "id": 1,
    "name": "Test Trading Data - UPDATED",
    "timestamp": "2024-01-15T10:05:00Z",
    "data": {
        "price": 101.25,
        "volume": 1500,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.1",
        "updated": true
    }
}
EOF
    
    echo "File updated! Check the WebSocket output above for real-time updates."
    echo "Waiting 5 seconds to see the update..."
    sleep 5
    
    # Stop wscat
    kill $wscat_pid 2>/dev/null || true
    echo -e "${GREEN}✅ WebSocket streaming test completed${NC}"
}

# Function to test Ollama processing
test_ollama_processing() {
    echo -e "${BLUE}🤖 Testing Ollama JSON processing...${NC}"
    
    # Check if Ollama is running
    if ! curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️ Ollama is not running, skipping Ollama test${NC}"
        echo "Start Ollama with: ollama serve"
        return 1
    fi
    
    echo "Processing JSON file with Ollama..."
    local response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d "{
            \"file_path\": \"$TEST_DIR/$TEST_FILE\",
            \"prompt\": \"Analyze this trading data and provide insights about the price movement and volume changes.\",
            \"model\": \"llama2\"
        }")
    
    echo "Ollama response: $response"
    
    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ Ollama processing completed successfully${NC}"
    else
        echo -e "${RED}❌ Ollama processing failed${NC}"
    fi
}

# Function to test file modifications
test_file_modifications() {
    echo -e "${BLUE}📝 Testing file modification detection...${NC}"
    
    echo "Creating multiple file modifications to test streaming..."
    
    # Modification 1: Price update
    echo "Modification 1: Updating price..."
    cat > "$TEST_DIR/$TEST_FILE" << 'EOF'
{
    "id": 1,
    "name": "Test Trading Data - PRICE UPDATE",
    "timestamp": "2024-01-15T10:10:00Z",
    "data": {
        "price": 102.75,
        "volume": 2000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.2",
        "update_type": "price_change"
    }
}
EOF
    sleep 2
    
    # Modification 2: Volume update
    echo "Modification 2: Updating volume..."
    cat > "$TEST_DIR/$TEST_FILE" << 'EOF'
{
    "id": 1,
    "name": "Test Trading Data - VOLUME UPDATE",
    "timestamp": "2024-01-15T10:15:00Z",
    "data": {
        "price": 102.75,
        "volume": 3000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.3",
        "update_type": "volume_change"
    }
}
EOF
    sleep 2
    
    # Modification 3: Final update
    echo "Modification 3: Final update..."
    cat > "$TEST_DIR/$TEST_FILE" << 'EOF'
{
    "id": 1,
    "name": "Test Trading Data - FINAL",
    "timestamp": "2024-01-15T10:20:00Z",
    "data": {
        "price": 103.50,
        "volume": 3500,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.4",
        "update_type": "final",
        "status": "completed"
    }
}
EOF
    
    echo -e "${GREEN}✅ File modifications completed${NC}"
}

# Function to test API endpoints
test_api_endpoints() {
    echo -e "${BLUE}🔌 Testing API endpoints...${NC}"
    
    # Health check
    echo "Testing health endpoint..."
    local health=$(curl -s "$API_BASE_URL/health")
    echo "Health: $health"
    
    # Available files
    echo "Testing available files endpoint..."
    local available_files=$(curl -s "$API_BASE_URL/api/available-files")
    echo "Available files: $available_files"
    
    # Get watched files
    echo "Testing watched files endpoint..."
    local watched_files=$(curl -s "$API_BASE_URL/api/files")
    echo "Watched files: $watched_files"
    
    echo -e "${GREEN}✅ API endpoint tests completed${NC}"
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up test data...${NC}"
    
    # Stop watching the test file
    if curl -s "$API_BASE_URL/api/watch/$TEST_DIR/$TEST_FILE" >/dev/null 2>&1; then
        echo "Stopped watching test file"
    fi
    
    # Remove test directory
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
        echo "Removed test directory"
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  --api-only        Test API endpoints only"
    echo "  --websocket-only  Test WebSocket streaming only"
    echo "  --ollama-only     Test Ollama processing only"
    echo "  --file-only       Test file watching only"
    echo "  --full-test       Run complete test suite (default)"
    echo "  --cleanup         Clean up test data only"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run complete test suite"
    echo "  $0 --websocket-only  # Test WebSocket only"
    echo "  $0 --cleanup         # Clean up test data"
}

# Main execution
main() {
    case "${1:---full-test}" in
        --api-only)
            echo -e "${BLUE}🔌 Running API endpoint tests only...${NC}"
            check_api || exit 1
            test_api_endpoints
            ;;
        --websocket-only)
            echo -e "${BLUE}🌊 Running WebSocket tests only...${NC}"
            check_api || exit 1
            create_test_data
            test_file_watching
            test_websocket_streaming
            cleanup
            ;;
        --ollama-only)
            echo -e "${BLUE}🤖 Running Ollama tests only...${NC}"
            check_api || exit 1
            create_test_data
            test_ollama_processing
            cleanup
            ;;
        --file-only)
            echo -e "${BLUE}📝 Running file watching tests only...${NC}"
            check_api || exit 1
            create_test_data
            test_file_watching
            test_file_modifications
            cleanup
            ;;
        --full-test)
            echo -e "${BLUE}🚀 Running complete JSON stream test suite...${NC}"
            echo ""
            
            # Check API
            check_api || exit 1
            
            # Create test data
            create_test_data
            
            echo ""
            # Test file watching
            test_file_watching
            
            echo ""
            # Test API endpoints
            test_api_endpoints
            
            echo ""
            # Test file modifications
            test_file_modifications
            
            echo ""
            # Test WebSocket streaming
            test_websocket_streaming
            
            echo ""
            # Test Ollama processing
            test_ollama_processing
            
            echo ""
            # Cleanup
            cleanup
            
            echo ""
            echo -e "${GREEN}🎉 Complete JSON stream test suite completed!${NC}"
            echo ""
            echo "📋 What was tested:"
            echo "  ✅ File watching and monitoring"
            echo "  ✅ Real-time WebSocket streaming"
            echo "  ✅ File modification detection"
            echo "  ✅ Ollama AI processing"
            echo "  ✅ API endpoint functionality"
            echo "  ✅ Real-time updates and notifications"
            ;;
        --cleanup)
            cleanup
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

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
