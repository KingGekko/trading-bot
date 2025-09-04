#!/bin/bash

# ========================================
# API KEY UPDATE SCRIPT
# ========================================
# This script helps you update your Alpaca API keys

set -e

echo "🔑 ALPACA API KEY UPDATE SCRIPT"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✅ SUCCESS${NC}: $message"
            ;;
        "ERROR")
            echo -e "${RED}❌ ERROR${NC}: $message"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠️  WARNING${NC}: $message"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  INFO${NC}: $message"
            ;;
    esac
}

# Check if config.env exists
if [ ! -f "config.env" ]; then
    print_status "ERROR" "config.env file not found!"
    print_status "INFO" "Please run this script from the project root directory"
    exit 1
fi

print_status "INFO" "Found config.env file"

# Get current API keys
CURRENT_API_KEY=$(grep "^APCA_API_KEY_ID=" config.env | cut -d'=' -f2)
CURRENT_SECRET_KEY=$(grep "^APCA_API_SECRET_KEY=" config.env | cut -d'=' -f2)

echo ""
echo "📋 CURRENT API KEYS:"
echo "==================="
echo "API Key: ${CURRENT_API_KEY:0:8}..."
echo "Secret: ${CURRENT_SECRET_KEY:0:8}..."
echo ""

# Ask user if they want to update
read -p "Do you want to update your API keys? (y/n): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_status "INFO" "API key update cancelled"
    exit 0
fi

echo ""
echo "🔑 ENTER YOUR NEW API KEYS"
echo "=========================="

# Get new API key
read -p "Enter your new Alpaca API Key: " NEW_API_KEY
if [ -z "$NEW_API_KEY" ]; then
    print_status "ERROR" "API key cannot be empty"
    exit 1
fi

# Get new secret key
read -p "Enter your new Alpaca Secret Key: " NEW_SECRET_KEY
if [ -z "$NEW_SECRET_KEY" ]; then
    print_status "ERROR" "Secret key cannot be empty"
    exit 1
fi

echo ""
print_status "INFO" "Updating config.env with new API keys..."

# Create backup
cp config.env config.env.backup.$(date +%Y%m%d_%H%M%S)
print_status "SUCCESS" "Created backup: config.env.backup"

# Update the config file
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # Windows (Git Bash)
    sed -i "s/APCA_API_KEY_ID=.*/APCA_API_KEY_ID=$NEW_API_KEY/" config.env
    sed -i "s/APCA_API_SECRET_KEY=.*/APCA_API_SECRET_KEY=$NEW_SECRET_KEY/" config.env
else
    # Unix/Linux
    sed -i "s/APCA_API_KEY_ID=.*/APCA_API_KEY_ID=$NEW_API_KEY/" config.env
    sed -i "s/APCA_API_SECRET_KEY=.*/APCA_API_SECRET_KEY=$NEW_SECRET_KEY/" config.env
fi

print_status "SUCCESS" "API keys updated successfully!"

echo ""
echo "📋 NEW API KEYS:"
echo "==============="
echo "API Key: ${NEW_API_KEY:0:8}..."
echo "Secret: ${NEW_SECRET_KEY:0:8}..."
echo ""

# Test the new API keys
print_status "INFO" "Testing new API keys..."

# Test API connection (basic test)
if curl -s -H "APCA-API-KEY-ID: $NEW_API_KEY" -H "APCA-API-SECRET-KEY: $NEW_SECRET_KEY" \
    "https://paper-api.alpaca.markets/v2/account" > /dev/null 2>&1; then
    print_status "SUCCESS" "API key test successful! Connection to Alpaca established."
else
    print_status "WARNING" "API key test failed. Please verify your keys are correct."
    print_status "INFO" "You can still use the trading bot, but some features may not work."
fi

echo ""
print_status "SUCCESS" "API key update complete!"
print_status "INFO" "You can now run the trading bot with your new API keys"
echo ""
echo "💡 Next steps:"
echo "   • Run: ./target/release/trading_bot.exe --test-orders"
echo "   • Run: ./target/release/trading_bot.exe --enhanced-strategy"
echo "   • Run: ./target/release/trading_bot.exe --execute-orders (when market opens)"
echo ""
