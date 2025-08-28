#!/bin/bash

# ⚡ Performance Test Runner
# Executes load testing, memory profiling, and latency testing

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
TEST_RESULTS=()

echo -e "${BLUE}⚡ Running Performance Tests...${NC}"
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

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Performance Test Summary${NC}"
    echo "================================="
    
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
        echo -e "${GREEN}🎉 All performance tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some performance tests failed${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Performance Test Suite${NC}"
    echo ""
    
    # Check prerequisites
    check_api_health || exit 1
    echo ""
    
    # Run tests
    run_test "Load Testing" "test_load.sh" "Tests system performance under high load with multiple concurrent requests and file updates"
    
    run_test "Memory Profiling" "test_memory.sh" "Monitors memory usage, identifies leaks, and validates resource consumption patterns"
    
    run_test "Latency Testing" "test_latency.sh" "Measures response times, WebSocket latency, and file update detection speed"
    
    run_test "Throughput Testing" "test_throughput.sh" "Tests data processing capacity, concurrent file watching, and system scalability"
    
    run_test "Stress Testing" "test_stress.sh" "Pushes system to limits with extreme load, rapid updates, and resource constraints"
    
    # Show summary
    show_summary
}

# Run main function
main "$@"
