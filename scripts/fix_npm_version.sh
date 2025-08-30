#!/bin/bash

# Quick Fix Script for npm Version Compatibility Issues
# This script resolves npm version conflicts during deployment

set -e

echo "🔧 Quick Fix for npm Version Compatibility"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check current versions
check_versions() {
    echo -e "${BLUE}🔍 Checking current versions...${NC}"
    
    if command -v node >/dev/null 2>&1; then
        local node_version=$(node --version)
        echo "📦 Node.js version: $node_version"
    else
        echo -e "${RED}❌ Node.js not found${NC}"
        return 1
    fi
    
    if command -v npm >/dev/null 2>&1; then
        local npm_version=$(npm --version)
        echo "📦 npm version: $npm_version"
    else
        echo -e "${RED}❌ npm not found${NC}"
        return 1
    fi
    
    return 0
}

# Function to fix npm version compatibility
fix_npm_compatibility() {
    echo -e "${BLUE}🔧 Fixing npm version compatibility...${NC}"
    
    # Check if we need to downgrade npm
    local npm_version=$(npm --version)
    if [[ "$npm_version" =~ ^[0-9]+\. ]] && (( $(echo "$npm_version" | cut -d. -f1) >= 11 )); then
        echo "⚠️ npm version $npm_version is too new, downgrading to compatible version..."
        
        # Downgrade to a compatible npm version
        if npm install -g npm@9.8.1; then
            echo -e "${GREEN}✅ npm downgraded to compatible version${NC}"
            npm --version
        else
            echo -e "${YELLOW}⚠️ npm downgrade failed, trying alternative...${NC}"
            if npm install -g npm@9.2.0; then
                echo -e "${GREEN}✅ npm downgraded to alternative version${NC}"
                npm --version
            else
                echo -e "${RED}❌ npm downgrade failed${NC}"
                return 1
            fi
        fi
    else
        echo -e "${GREEN}✅ npm version $npm_version is compatible${NC}"
    fi
    
    return 0
}

# Function to install wscat with compatible npm
install_wscat() {
    echo -e "${BLUE}📦 Installing wscat for WebSocket testing...${NC}"
    
    # Try installing wscat with compatible version first
    if npm install -g wscat@5.1.1; then
        echo -e "${GREEN}✅ wscat installed successfully (compatible version)${NC}"
        wscat --version
        return 0
    elif npm install -g wscat; then
        echo -e "${GREEN}✅ wscat installed successfully${NC}"
        wscat --version
        return 0
    else
        echo -e "${RED}❌ wscat installation failed${NC}"
        echo "You can try installing manually:"
        echo "  npm install -g wscat@5.1.1"
        echo "  or"
        echo "  npm install -g wscat"
        return 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  --check-only     Only check versions"
    echo "  --fix-npm        Fix npm version compatibility"
    echo "  --install-wscat  Install wscat with compatible npm"
    echo "  --full-fix       Run complete fix (default)"
    echo "  --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run complete fix"
    echo "  $0 --check-only      # Check versions only"
    echo "  $0 --fix-npm         # Fix npm version only"
}

# Main execution
main() {
    case "${1:---full-fix}" in
        --check-only)
            check_versions
            ;;
        --fix-npm)
            check_versions || exit 1
            fix_npm_compatibility
            ;;
        --install-wscat)
            check_versions || exit 1
            install_wscat
            ;;
        --full-fix)
            echo -e "${BLUE}🚀 Running complete npm compatibility fix...${NC}"
            echo ""
            
            # Check versions
            check_versions || exit 1
            
            echo ""
            # Fix npm compatibility
            fix_npm_compatibility
            
            echo ""
            # Install wscat
            install_wscat
            
            echo ""
            echo -e "${GREEN}🎉 npm compatibility fix completed!${NC}"
            echo ""
            echo "📋 What was fixed:"
            echo "  ✅ npm version compatibility checked"
            echo "  ✅ npm downgraded if needed"
            echo "  ✅ wscat installed with compatible npm"
            echo ""
            echo "💡 You can now continue with the deployment!"
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

# Run main function
main "$@"
