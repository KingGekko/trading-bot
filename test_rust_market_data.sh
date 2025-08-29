#!/bin/bash

# 🧪 Rust Market Data Testing Script
# This script tests the new Rust-based market data streaming system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Testing Rust-Based Market Data System${NC}"
echo "============================================="
echo ""

# Function to check if Rust is available
check_rust() {
    echo -e "${BLUE}🔍 Checking Rust installation...${NC}"
    
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Rust/Cargo found${NC}"
        cargo --version
    else
        echo -e "${RED}❌ Rust/Cargo not found${NC}"
        echo "Please install Rust toolchain first"
        exit 1
    fi
    
    echo ""
}

# Function to build the project
build_project() {
    echo -e "${BLUE}🔨 Building project with market data support...${NC}"
    
    # Clean build to ensure all modules are included
    cargo clean
    cargo build
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
    
    echo ""
}

# Function to test market data module compilation
test_module_compilation() {
    echo -e "${BLUE}🧪 Testing market data module compilation...${NC}"
    
    # Test if the market data module compiles
    if cargo check --lib 2>/dev/null; then
        echo -e "${GREEN}✅ Market data module compiles successfully${NC}"
    else
        echo -e "${RED}❌ Market data module compilation failed${NC}"
        echo "Checking for compilation errors..."
        cargo check --lib
        exit 1
    fi
    
    echo ""
}

# Function to run market data tests
run_market_data_tests() {
    echo -e "${BLUE}🧪 Running market data tests...${NC}"
    
    # Run tests for market data module
    if cargo test market_data 2>/dev/null; then
        echo -e "${GREEN}✅ Market data tests passed${NC}"
    else
        echo -e "${YELLOW}⚠️ Some market data tests failed or no tests found${NC}"
        echo "Running all tests to check..."
        cargo test
    fi
    
    echo ""
}

# Function to check configuration
check_configuration() {
    echo -e "${BLUE}🔧 Checking market data configuration...${NC}"
    
    if [ -f "live_config.env" ]; then
        echo -e "${GREEN}✅ Live mode configuration found${NC}"
        
        # Check if API keys are configured
        if grep -q "your_alpaca_api_key_here" live_config.env; then
            echo -e "${YELLOW}⚠️ API keys not configured in live_config.env${NC}"
            echo "Please update live_config.env with your Alpaca API credentials"
        else
            echo -e "${GREEN}✅ API keys appear to be configured${NC}"
        fi
    else
        echo -e "${RED}❌ Live mode configuration not found${NC}"
        echo "Please create live_config.env with your Alpaca API credentials"
    fi
    
    echo ""
}

# Function to show next steps
show_next_steps() {
    echo -e "${BLUE}📋 Next Steps${NC}"
    echo "============="
    echo ""
    echo "1. Configure Alpaca API credentials in live_config.env"
    echo "2. Test the system:"
    echo "   - Test mode: ./start_test_mode.sh"
    echo "   - Live mode: ./start_live_mode.sh"
    echo "3. Monitor market data files in live_data/ directory"
    echo "4. Check logs in logs/ directory"
    echo ""
    echo -e "${GREEN}🎉 Rust-based market data system is ready!${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}🧪 Testing Rust-Based Market Data System${NC}"
    echo ""
    
    # Run all checks
    check_rust
    build_project
    test_module_compilation
    run_market_data_tests
    check_configuration
    
    # Show next steps
    show_next_steps
}

# Run main function
main "$@"
