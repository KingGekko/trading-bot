#!/bin/bash

# 🧪 Trading Bot Test Suite Runner
# Comprehensive testing framework for all test categories

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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTS_DIR="$SCRIPT_DIR/tests"
LOG_FILE="$SCRIPT_DIR/test_results.log"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

echo -e "${BLUE}🧪 Trading Bot Test Suite Runner${NC}"
echo "====================================="
echo "Timestamp: $TIMESTAMP"
echo "Tests Directory: $TESTS_DIR"
echo ""

# Function to log test results
log_result() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    case $status in
        "PASS")
            echo -e "${GREEN}✅ PASS${NC}: $test_name - $message"
            ((PASSED_TESTS++))
            ;;
        "FAIL")
            echo -e "${RED}❌ FAIL${NC}: $test_name - $message"
            ((FAILED_TESTS++))
            ;;
        "SKIP")
            echo -e "${YELLOW}⚠️ SKIP${NC}: $test_name - $message"
            ((SKIPPED_TESTS++))
            ;;
    esac
    
    # Log to file
    echo "[$TIMESTAMP] $status: $test_name - $message" >> "$LOG_FILE"
    ((TOTAL_TESTS++))
}

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if tests directory exists
    if [ ! -d "$TESTS_DIR" ]; then
        echo -e "${RED}❌ Tests directory not found: $TESTS_DIR${NC}"
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo >/dev/null 2>&1; then
        log_result "Prerequisites" "SKIP" "Cargo not found, skipping Rust tests"
    else
        log_result "Prerequisites" "PASS" "Cargo available"
    fi
    
    # Check if curl is available
    if ! command -v curl >/dev/null 2>&1; then
        log_result "Prerequisites" "SKIP" "curl not found, some tests may fail"
    else
        log_result "Prerequisites" "PASS" "curl available"
    fi
    
    echo ""
}

# Function to run unit tests
run_unit_tests() {
    echo -e "${BLUE}🔧 Running Unit Tests...${NC}"
    
    if ! command -v cargo >/dev/null 2>&1; then
        log_result "Unit Tests" "SKIP" "Cargo not available"
        return
    fi
    
    if [ -f "$TESTS_DIR/unit/run_unit_tests.sh" ]; then
        if "$TESTS_DIR/unit/run_unit_tests.sh"; then
            log_result "Unit Tests" "PASS" "All unit tests completed successfully"
        else
            log_result "Unit Tests" "FAIL" "Unit tests failed"
        fi
    else
        # Fallback to cargo test
        if cargo test; then
            log_result "Unit Tests" "PASS" "Cargo tests completed successfully"
        else
            log_result "Unit Tests" "FAIL" "Cargo tests failed"
        fi
    fi
    
    echo ""
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running Integration Tests...${NC}"
    
    if [ -f "$TESTS_DIR/integration/run_integration_tests.sh" ]; then
        if "$TESTS_DIR/integration/run_integration_tests.sh"; then
            log_result "Integration Tests" "PASS" "All integration tests completed successfully"
        else
            log_result "Integration Tests" "FAIL" "Integration tests failed"
        fi
    else
        log_result "Integration Tests" "SKIP" "Integration test runner not found"
    fi
    
    echo ""
}

# Function to run performance tests
run_performance_tests() {
    echo -e "${BLUE}⚡ Running Performance Tests...${NC}"
    
    if [ -f "$TESTS_DIR/performance/run_performance_tests.sh" ]; then
        if "$TESTS_DIR/performance/run_performance_tests.sh"; then
            log_result "Performance Tests" "PASS" "All performance tests completed successfully"
        else
            log_result "Performance Tests" "FAIL" "Performance tests failed"
        fi
    else
        log_result "Performance Tests" "SKIP" "Performance test runner not found"
    fi
    
    echo ""
}

# Function to run deployment tests
run_deployment_tests() {
    echo -e "${BLUE}🚀 Running Deployment Tests...${NC}"
    
    if [ -f "$TESTS_DIR/deployment/run_deployment_tests.sh" ]; then
        if "$TESTS_DIR/deployment/run_deployment_tests.sh"; then
            log_result "Deployment Tests" "PASS" "All deployment tests completed successfully"
        else
            log_result "Deployment Tests" "FAIL" "Deployment tests failed"
        fi
    else
        log_result "Deployment Tests" "SKIP" "Deployment test runner not found"
    fi
    
    echo ""
}

# Function to run all tests
run_all_tests() {
    echo -e "${BLUE}🚀 Running Complete Test Suite...${NC}"
    echo ""
    
    check_prerequisites
    run_unit_tests
    run_integration_tests
    run_performance_tests
    run_deployment_tests
}

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Test Summary${NC}"
    echo "================"
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo -e "${YELLOW}Skipped: $SKIPPED_TESTS${NC}"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Check the log: $LOG_FILE${NC}"
        exit 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  --unit-only        Run unit tests only"
    echo "  --integration-only Run integration tests only"
    echo "  --performance-only Run performance tests only"
    echo "  --deployment-only  Run deployment tests only"
    echo "  --all             Run complete test suite (default)"
    echo "  --summary         Show test summary only"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run complete test suite"
    echo "  $0 --unit-only       # Run unit tests only"
    echo "  $0 --integration-only # Run integration tests only"
    echo "  $0 --summary         # Show test summary"
    echo ""
    echo "Test Categories:"
    echo "  Unit Tests:      Rust component testing"
    echo "  Integration:     API and system integration"
    echo "  Performance:     Load and stress testing"
    echo "  Deployment:      Environment validation"
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up test environment...${NC}"
    
    # Remove test files if they exist
    if [ -d "test_files" ]; then
        rm -rf test_files
        echo "Removed test files"
    fi
    
    # Stop any test services
    pkill -f "test.*service" 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Main execution
main() {
    # Initialize log file
    echo "=== Trading Bot Test Suite Results ===" > "$LOG_FILE"
    echo "Timestamp: $TIMESTAMP" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
    
    case "${1:---all}" in
        --unit-only)
            echo -e "${BLUE}🔧 Running Unit Tests Only...${NC}"
            check_prerequisites
            run_unit_tests
            ;;
        --integration-only)
            echo -e "${BLUE}🔗 Running Integration Tests Only...${NC}"
            check_prerequisites
            run_integration_tests
            ;;
        --performance-only)
            echo -e "${BLUE}⚡ Running Performance Tests Only...${NC}"
            check_prerequisites
            run_performance_tests
            ;;
        --deployment-only)
            echo -e "${BLUE}🚀 Running Deployment Tests Only...${NC}"
            check_prerequisites
            run_deployment_tests
            ;;
        --all)
            run_all_tests
            ;;
        --summary)
            if [ -f "$LOG_FILE" ]; then
                echo "=== Test Results Summary ==="
                cat "$LOG_FILE"
            else
                echo "No test results found. Run tests first."
            fi
            exit 0
            ;;
        --help|-h)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            show_usage
            exit 1
            ;;
    esac
    
    # Show summary
    show_summary
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
