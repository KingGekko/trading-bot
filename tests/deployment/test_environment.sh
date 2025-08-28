#!/bin/bash

# 🧪 Environment Test
# Validates system requirements and basic environment setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Testing Environment...${NC}"
echo "============================="
echo ""

# Test OS compatibility
test_os() {
    echo -e "${BLUE}🧪 Testing OS compatibility...${NC}"
    
    local os_type=$(uname -s)
    local os_version=$(uname -r)
    
    echo "OS Type: $os_type"
    echo "OS Version: $os_version"
    
    # Check if it's a supported OS
    case $os_type in
        "Linux"|"Darwin")
            echo -e "${GREEN}✅ OS is supported${NC}"
            ;;
        *)
            echo -e "${YELLOW}⚠️ OS may not be fully supported${NC}"
            ;;
    esac
    
    echo ""
}

# Test system resources
test_resources() {
    echo -e "${BLUE}🧪 Testing system resources...${NC}"
    
    # Check available memory
    if command -v free >/dev/null 2>&1; then
        local mem_total=$(free -m | awk 'NR==2{printf "%.0f", $2}')
        echo "Total Memory: ${mem_total}MB"
        
        if [ $mem_total -gt 1024 ]; then
            echo -e "${GREEN}✅ Sufficient memory available${NC}"
        else
            echo -e "${YELLOW}⚠️ Low memory, may affect performance${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️ Cannot check memory (free command not available)${NC}"
    fi
    
    # Check available disk space
    local disk_space=$(df -h . | awk 'NR==2{print $4}')
    echo "Available Disk Space: $disk_space"
    
    echo ""
}

# Test network connectivity
test_network() {
    echo -e "${BLUE}🧪 Testing network connectivity...${NC}"
    
    # Test internet connectivity
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Internet connectivity available${NC}"
    else
        echo -e "${YELLOW}⚠️ Internet connectivity may be limited${NC}"
    fi
    
    # Test localhost
    if ping -c 1 localhost >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Localhost connectivity available${NC}"
    else
        echo -e "${RED}❌ Localhost connectivity failed${NC}"
        return 1
    fi
    
    echo ""
}

# Test file system
test_filesystem() {
    echo -e "${BLUE}🧪 Testing file system...${NC}"
    
    # Check if we can write to current directory
    local test_file="test_write_$(date +%s).tmp"
    
    if echo "test" > "$test_file" 2>/dev/null; then
        echo -e "${GREEN}✅ Write permissions available${NC}"
        rm -f "$test_file"
    else
        echo -e "${RED}❌ Write permissions not available${NC}"
        return 1
    fi
    
    # Check if we can create directories
    local test_dir="test_dir_$(date +%s)"
    
    if mkdir "$test_dir" 2>/dev/null; then
        echo -e "${GREEN}✅ Directory creation available${NC}"
        rmdir "$test_dir"
    else
        echo -e "${RED}❌ Directory creation not available${NC}"
        return 1
    fi
    
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Environment Tests${NC}"
    echo ""
    
    test_os
    test_resources
    test_network
    test_filesystem
    
    echo -e "${GREEN}🎉 Environment tests completed successfully!${NC}"
    echo "System appears ready for trading bot deployment."
}

# Run main function
main "$@"
