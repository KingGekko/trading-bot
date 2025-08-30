#!/bin/bash

# 🐛 Debug File Watching System
# This script helps debug the broken file watching functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:8080"
TEST_FILE="debug_test.json"

echo -e "${BLUE}🐛 Debug File Watching System${NC}"
echo "====================================="
echo "This script will help identify exactly where the file watching fails"
echo ""

# Function to check if API is running
check_api_health() {
    echo -e "${BLUE}🔍 Step 1: Check API Health${NC}"
    
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

# Function to create test file
create_test_file() {
    echo -e "${BLUE}🔍 Step 2: Create Test File${NC}"
    
    cat > "$TEST_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "test": "debug_test",
  "value": 100,
  "status": "initial"
}
EOF
    
    echo -e "${GREEN}✅ Created test file: $TEST_FILE${NC}"
    echo "File path: $(pwd)/$TEST_FILE"
    echo "File exists: $(test -f "$TEST_FILE" && echo "Yes" || echo "No")"
    echo "File size: $(wc -c < "$TEST_FILE") bytes"
    echo ""
}

# Function to test file existence check
test_file_existence() {
    echo -e "${BLUE}🔍 Step 3: Test File Existence${NC}"
    
    # Test with absolute path
    ABSOLUTE_PATH="$(pwd)/$TEST_FILE"
    echo "Testing with absolute path: $ABSOLUTE_PATH"
    
    if [ -f "$ABSOLUTE_PATH" ]; then
        echo -e "${GREEN}✅ File exists at absolute path${NC}"
    else
        echo -e "${RED}❌ File does not exist at absolute path${NC}"
    fi
    
    # Test with relative path
    echo "Testing with relative path: $TEST_FILE"
    if [ -f "$TEST_FILE" ]; then
        echo -e "${GREEN}✅ File exists at relative path${NC}"
    else
        echo -e "${RED}❌ File does not exist at relative path${NC}"
    fi
    
    echo ""
}

# Function to test basic API endpoints
test_basic_endpoints() {
    echo -e "${BLUE}🔍 Step 4: Test Basic API Endpoints${NC}"
    
    # Test health endpoint
    echo "Testing /health endpoint..."
    health_response=$(curl -s "$API_BASE_URL/health" 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$health_response" ]; then
        echo -e "${GREEN}✅ Health endpoint working${NC}"
        echo "Response: $health_response"
    else
        echo -e "${RED}❌ Health endpoint failed${NC}"
    fi
    
    # Test available files endpoint
    echo ""
    echo "Testing /api/available-files endpoint..."
    files_response=$(curl -s "$API_BASE_URL/api/available-files" 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$files_response" ]; then
        echo -e "${GREEN}✅ Available files endpoint working${NC}"
        echo "Response: $files_response"
    else
        echo -e "${RED}❌ Available files endpoint failed${NC}"
    fi
    
    echo ""
}

# Function to test file watching with detailed output
test_file_watching() {
    echo -e "${BLUE}🔍 Step 5: Test File Watching (Detailed)${NC}"
    
    echo "Attempting to start file watch..."
    echo "File path: $TEST_FILE"
    echo "Current directory: $(pwd)"
    echo ""
    
    # Test with relative path
    echo "Testing with relative path: $TEST_FILE"
    watch_response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d "{\"file_path\": \"$TEST_FILE\"}" 2>/dev/null)
    
    echo "cURL exit code: $?"
    echo "Response: $watch_response"
    
    if [ $? -eq 0 ] && [ -n "$watch_response" ]; then
        echo -e "${GREEN}✅ File watch started successfully${NC}"
        echo "Response: $watch_response"
    else
        echo -e "${RED}❌ File watch failed${NC}"
        echo "This indicates the file watching system is broken"
        echo ""
        echo "Let's try with absolute path..."
        
        # Test with absolute path
        ABSOLUTE_PATH="$(pwd)/$TEST_FILE"
        echo "Testing with absolute path: $ABSOLUTE_PATH"
        
        watch_response_abs=$(curl -s -X POST "$API_BASE_URL/api/watch" \
            -H "Content-Type: application/json" \
            -d "{\"file_path\": \"$ABSOLUTE_PATH\"}" 2>/dev/null)
        
        echo "cURL exit code: $?"
        echo "Response: $watch_response_abs"
        
        if [ $? -eq 0 ] && [ -n "$watch_response_abs" ]; then
            echo -e "${GREEN}✅ File watch started with absolute path${NC}"
            echo "Response: $watch_response_abs"
        else
            echo -e "${RED}❌ File watch failed with absolute path too${NC}"
            echo "The issue is deeper in the file watching system"
        fi
    fi
    
    echo ""
}

# Function to check server logs
check_server_logs() {
    echo -e "${BLUE}🔍 Step 6: Check Server Logs${NC}"
    
    echo "The server should have logged detailed information about the file watching attempt."
    echo "Look for these log messages in your server output:"
    echo ""
    echo "  - 'Starting to watch file: ...'"
    echo "  - 'File exists, attempting to start watch...'"
    echo "  - 'JsonStreamManager: Attempting to watch file: ...'"
    echo "  - 'JsonStreamManager: Resolved path: ...'"
    echo "  - 'JsonStreamManager: File exists, checking if already watching...'"
    echo "  - 'JsonStreamManager: Creating new broadcast channel...'"
    echo "  - 'JsonStreamManager: Starting file watcher...'"
    echo ""
    echo "If you see any error messages, they will help identify the exact issue."
    echo ""
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}🧹 Cleanup${NC}"
    
    if [ -f "$TEST_FILE" ]; then
        rm -f "$TEST_FILE"
        echo -e "${GREEN}✅ Removed test file${NC}"
    fi
    
    echo ""
}

# Function to show next steps
show_next_steps() {
    echo -e "${BLUE}📋 Next Steps${NC}"
    echo "=============="
    echo ""
    
    if [ -f "$TEST_FILE" ]; then
        echo "1. Check your server logs for the detailed debug information"
        echo "2. Look for any error messages or warnings"
        echo "3. The debug logs will show exactly where the failure occurs"
        echo "4. Common issues:"
        echo "   - File permissions"
        echo "   - Path resolution problems"
        echo "   - notify crate compatibility issues"
        echo "   - Async runtime issues"
        echo ""
        echo "5. Run this script again after fixing any issues"
    else
        echo "1. The test file was cleaned up"
        echo "2. Run this script again to test the file watching system"
    fi
    
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🐛 Starting File Watching Debug${NC}"
    echo ""
    
    # Run all debug steps
    check_api_health || exit 1
    create_test_file
    test_file_existence
    test_basic_endpoints
    test_file_watching
    check_server_logs
    
    # Cleanup and show next steps
    cleanup
    show_next_steps
    
    echo -e "${BLUE}🐛 Debug Complete${NC}"
    echo "Check your server logs for detailed information about what went wrong."
}

# Run main function
main "$@"
