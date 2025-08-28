#!/bin/bash

# 🔗 Integration Test Runner
# Tests API endpoints, JSON streaming, and WebSocket functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_BASE_URL="http://localhost:3000"
WS_BASE_URL="ws://localhost:3000"
TEST_RESULTS=()

echo -e "${BLUE}🔗 Running Integration Tests...${NC}"
echo "================================="
echo ""

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_script="$2"
    local description="$3"
    
    echo -e "${BLUE}🧪 Running: $test_name${NC}"
    echo "Description: $description"
    echo ""
    
    if [ -f "$test_script" ]; then
        if bash "$test_script"; then
            echo -e "${GREEN}✅ $test_name: PASSED${NC}"
            TEST_RESULTS+=("PASS:$test_name")
        else
            echo -e "${RED}❌ $test_name: FAILED${NC}"
            TEST_RESULTS+=("FAIL:$test_name")
        fi
    else
        echo -e "${YELLOW}⚠️ $test_name: SKIPPED (script not found)${NC}"
        TEST_RESULTS+=("SKIP:$test_name")
    fi
    
    echo ""
    echo "----------------------------------------"
    echo ""
}

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

# Function to check Ollama
check_ollama() {
    echo -e "${BLUE}🔍 Checking Ollama...${NC}"
    
    if curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Ollama is running${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️ Ollama is not running${NC}"
        echo "Some tests may be skipped. Start Ollama with:"
        echo "  ollama serve"
        return 1
    fi
}

# Function to check wscat
check_wscat() {
    echo -e "${BLUE}🔍 Checking wscat...${NC}"
    
    if command -v wscat >/dev/null 2>&1; then
        echo -e "${GREEN}✅ wscat is available${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️ wscat not found${NC}"
        echo "WebSocket tests may be skipped. Install with:"
        echo "  npm install -g wscat@5.1.1"
        return 1
    fi
}

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Integration Test Summary${NC}"
    echo "================================"
    
    local passed=0
    local failed=0
    local skipped=0
    
    for result in "${TEST_RESULTS[@]}"; do
        local status="${result%%:*}"
        local test_name="${result#*:}"
        
        case $status in
            "PASS")
                echo -e "${GREEN}✅ PASS${NC}: $test_name"
                ((passed++))
                ;;
            "FAIL")
                echo -e "${RED}❌ FAIL${NC}: $test_name"
                ((failed++))
                ;;
            "SKIP")
                echo -e "${YELLOW}⚠️ SKIP${NC}: $test_name"
                ((skipped++))
                ;;
        esac
    done
    
    echo ""
    echo "Total Tests: ${#TEST_RESULTS[@]}"
    echo -e "${GREEN}Passed: $passed${NC}"
    echo -e "${RED}Failed: $failed${NC}"
    echo -e "${YELLOW}Skipped: $skipped${NC}"
    echo ""
    
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}🎉 All integration tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some integration tests failed${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Integration Test Suite${NC}"
    echo ""
    
    # Check prerequisites
    check_api_health || exit 1
    check_ollama
    check_wscat
    echo ""
    
    # Run tests
    run_test "API Endpoints" "test_api_endpoints.sh" "Tests all REST API endpoints including health, file watching, and Ollama processing"
    
    run_test "JSON Stream" "test_json_stream.sh" "Tests complete JSON streaming functionality including file watching, updates, and real-time notifications"
    
    run_test "WebSocket" "test_websocket.sh" "Tests WebSocket real-time streaming and connection management"
    
    # Show summary
    show_summary
}

# Run main function
main "$@"
