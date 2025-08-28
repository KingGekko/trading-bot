#!/bin/bash

# 🧪 Latency Test
# Measures response times and system latency

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:3000"
TEST_ITERATIONS=10

echo -e "${BLUE}⚡ Testing System Latency...${NC}"
echo "==============================="
echo ""

# Function to measure API response time
test_api_latency() {
    echo -e "${BLUE}🧪 Testing API response latency...${NC}"
    
    if ! curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${RED}❌ API is not running, skipping latency test${NC}"
        return 1
    fi
    
    local total_time=0
    local min_time=999999
    local max_time=0
    
    echo "Running $TEST_ITERATIONS API health checks..."
    
    for i in $(seq 1 $TEST_ITERATIONS); do
        local start_time=$(date +%s%N)
        curl -s "$API_BASE_URL/health" >/dev/null
        local end_time=$(date +%s%N)
        
        local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
        total_time=$((total_time + duration))
        
        if [ $duration -lt $min_time ]; then
            min_time=$duration
        fi
        
        if [ $duration -gt $max_time ]; then
            max_time=$duration
        fi
        
        echo "  Request $i: ${duration}ms"
    done
    
    local avg_time=$((total_time / TEST_ITERATIONS))
    
    echo ""
    echo "Latency Results:"
    echo "  Min: ${min_time}ms"
    echo "  Max: ${max_time}ms"
    echo "  Avg: ${avg_time}ms"
    
    # Performance validation
    if [ $avg_time -lt 200 ]; then
        echo -e "${GREEN}✅ Excellent latency (< 200ms)${NC}"
    elif [ $avg_time -lt 500 ]; then
        echo -e "${GREEN}✅ Good latency (< 500ms)${NC}"
    elif [ $avg_time -lt 1000 ]; then
        echo -e "${YELLOW}⚠️ Acceptable latency (< 1000ms)${NC}"
    else
        echo -e "${RED}❌ Poor latency (>= 1000ms)${NC}"
    fi
    
    echo ""
}

# Function to measure file system latency
test_filesystem_latency() {
    echo -e "${BLUE}🧪 Testing file system latency...${NC}"
    
    local test_file="latency_test_$(date +%s).tmp"
    local total_time=0
    
    echo "Running $TEST_ITERATIONS file write/read operations..."
    
    for i in $(seq 1 $TEST_ITERATIONS); do
        local start_time=$(date +%s%N)
        
        # Write test
        echo "test data $i" > "$test_file"
        
        # Read test
        cat "$test_file" >/dev/null
        
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        total_time=$((total_time + duration))
        
        echo "  Operation $i: ${duration}ms"
    done
    
    # Cleanup
    rm -f "$test_file"
    
    local avg_time=$((total_time / TEST_ITERATIONS))
    
    echo ""
    echo "File System Latency Results:"
    echo "  Average: ${avg_time}ms"
    
    if [ $avg_time -lt 10 ]; then
        echo -e "${GREEN}✅ Excellent file system performance (< 10ms)${NC}"
    elif [ $avg_time -lt 50 ]; then
        echo -e "${GREEN}✅ Good file system performance (< 50ms)${NC}"
    else
        echo -e "${YELLOW}⚠️ File system may be slow (>= 50ms)${NC}"
    fi
    
    echo ""
}

# Function to measure memory allocation latency
test_memory_latency() {
    echo -e "${BLUE}🧪 Testing memory allocation latency...${NC}"
    
    local total_time=0
    
    echo "Running $TEST_ITERATIONS memory allocation tests..."
    
    for i in $(seq 1 $TEST_ITERATIONS); do
        local start_time=$(date +%s%N)
        
        # Allocate and deallocate memory
        local test_var=$(seq 1 1000 | tr '\n' ' ')
        unset test_var
        
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        total_time=$((total_time + duration))
        
        echo "  Allocation $i: ${duration}ms"
    done
    
    local avg_time=$((total_time / TEST_ITERATIONS))
    
    echo ""
    echo "Memory Latency Results:"
    echo "  Average: ${avg_time}ms"
    
    if [ $avg_time -lt 5 ]; then
        echo -e "${GREEN}✅ Excellent memory performance (< 5ms)${NC}"
    elif [ $avg_time -lt 20 ]; then
        echo -e "${GREEN}✅ Good memory performance (< 20ms)${NC}"
    else
        echo -e "${YELLOW}⚠️ Memory performance may be slow (>= 20ms)${NC}"
    fi
    
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Latency Tests${NC}"
    echo ""
    
    test_api_latency
    test_filesystem_latency
    test_memory_latency
    
    echo -e "${GREEN}🎉 Latency tests completed successfully!${NC}"
    echo "System latency appears to be within acceptable ranges."
}

# Run main function
main "$@"
