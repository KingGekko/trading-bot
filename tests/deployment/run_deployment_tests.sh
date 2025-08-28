#!/bin/bash

# 🚀 Deployment Test Runner
# Validates deployment environment, dependencies, and service health

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_RESULTS=()

echo -e "${BLUE}🚀 Running Deployment Tests...${NC}"
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

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Deployment Test Summary${NC}"
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
        echo -e "${GREEN}🎉 Deployment environment is ready!${NC}"
        return 0
    else
        echo -e "${RED}❌ Deployment environment has issues${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Deployment Test Suite${NC}"
    echo ""
    
    # Run tests
    run_test "Environment" "test_environment.sh" "Validates system requirements, OS compatibility, and basic environment setup"
    
    run_test "Dependencies" "test_dependencies.sh" "Checks availability of required packages, tools, and runtime dependencies"
    
    run_test "Services" "test_services.sh" "Validates service health, network connectivity, and configuration"
    
    run_test "Configuration" "test_configuration.sh" "Tests environment variables, configuration files, and security settings"
    
    # Show summary
    show_summary
}

# Run main function
main "$@"
