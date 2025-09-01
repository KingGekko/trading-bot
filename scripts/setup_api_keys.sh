#!/bin/bash

# 🚀 Simple API Key Setup Script
# Run this in your VM to easily configure multiple API keys

echo "🚀 Trading Bot API Key Setup"
echo "=============================="

# Check if config.env exists - detect if running from scripts/ or root directory
if [ -f "../config.env" ]; then
    CONFIG_FILE="../config.env"
elif [ -f "config.env" ]; then
    CONFIG_FILE="config.env"
else
    echo "❌ config.env not found. Please run from project root or scripts directory."
    exit 1
fi

# Function to add API key
add_api_key() {
    local key_name=$1
    local env_var=$2
    local current_value=$(grep "^${env_var}=" "$CONFIG_FILE" | cut -d'=' -f2)
    
    echo ""
    echo "🔑 Setting up ${key_name}..."
    
    if [ "$current_value" != "your_${key_name}_here" ] && [ -n "$current_value" ]; then
        echo "Current value: ${current_value}"
        read -p "Replace? (y/n): " replace
        if [ "$replace" != "y" ]; then
            echo "Keeping existing value."
            return
        fi
    fi
    
    read -p "Enter your ${key_name}: " api_key
    
    if [ -n "$api_key" ]; then
        # Replace the line in config.env
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            sed -i '' "s/^${env_var}=.*/${env_var}=${api_key}/" "$CONFIG_FILE"
        else
            # Linux
            sed -i "s/^${env_var}=.*/${env_var}=${api_key}/" "$CONFIG_FILE"
        fi
        echo "✅ ${key_name} updated successfully!"
    else
        echo "❌ ${key_name} cannot be empty. Skipping..."
    fi
}

# Function to add multiple API keys
add_multiple_keys() {
    echo ""
    echo "🔑 Adding Multiple API Keys"
    echo "============================"
    
    # Add Alpaca API keys
    add_api_key "Alpaca API Key" "ALPACA_API_KEY"
    add_api_key "Alpaca Secret Key" "ALPACA_SECRET_KEY"
    
    # Add other optional keys
    echo ""
    echo "🔑 Optional: Additional API Keys"
    echo "================================"
    
    read -p "Add additional API keys? (y/n): " add_more
    if [ "$add_more" = "y" ]; then
        while true; do
            echo ""
            read -p "Enter API key name (or 'done' to finish): " key_name
            if [ "$key_name" = "done" ]; then
                break
            fi
            
            read -p "Enter API key value: " key_value
            if [ -n "$key_value" ]; then
                # Add to config.env
                echo "${key_name^^}=${key_value}" >> "$CONFIG_FILE"
                echo "✅ Added ${key_name} successfully!"
            fi
        done
    fi
}

# Function to validate configuration
validate_config() {
    echo ""
    echo "🔍 Validating Configuration"
    echo "==========================="
    
    # Check for either paper trading or live trading credentials
    local paper_key=$(grep "^APCA_API_KEY_ID=" "$CONFIG_FILE" | cut -d'=' -f2)
    local paper_secret=$(grep "^APCA_API_SECRET_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    local live_key=$(grep "^ALPACA_API_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    local live_secret=$(grep "^ALPACA_SECRET_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    
    # Check if paper trading credentials are set
    if [ "$paper_key" != "your_paper_api_key_here" ] && [ -n "$paper_key" ] && 
       [ "$paper_secret" != "your_paper_secret_key_here" ] && [ -n "$paper_secret" ]; then
        echo "✅ Paper Trading credentials configured"
        echo "📝 Mode: Paper Trading (TEST)"
        return 0
    fi
    
    # Check if live trading credentials are set
    if [ "$live_key" != "your_live_api_key_here" ] && [ -n "$live_key" ] && 
       [ "$live_secret" != "your_live_secret_key_here" ] && [ -n "$live_secret" ]; then
        echo "✅ Live Trading credentials configured"
        echo "⚠️  Mode: Live Trading (REAL MONEY)"
        return 0
    fi
    
    echo "❌ No valid API credentials found"
    echo "   Set either APCA_API_KEY_ID/APCA_API_SECRET_KEY for paper trading"
    echo "   or ALPACA_API_KEY/ALPACA_SECRET_KEY for live trading"
    return 1
}

# Function to show current configuration
show_config() {
    echo ""
    echo "📋 Current Configuration"
    echo "========================"
    
    # Show Paper Trading credentials (masked)
    local paper_key=$(grep "^APCA_API_KEY_ID=" "$CONFIG_FILE" | cut -d'=' -f2)
    local paper_secret=$(grep "^APCA_API_SECRET_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    
    echo "📝 Paper Trading Credentials:"
    if [ "$paper_key" != "your_paper_api_key_here" ] && [ -n "$paper_key" ]; then
        echo "  APCA_API_KEY_ID: ${paper_key:0:8}...${paper_key: -4}"
    else
        echo "  APCA_API_KEY_ID: Not set"
    fi
    
    if [ "$paper_secret" != "your_paper_secret_key_here" ] && [ -n "$paper_secret" ]; then
        echo "  APCA_API_SECRET_KEY: ${paper_secret:0:8}...${paper_key: -4}"
    else
        echo "  APCA_API_SECRET_KEY: Not set"
    fi
    
    # Show Live Trading credentials (masked)
    local live_key=$(grep "^ALPACA_API_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    local live_secret=$(grep "^ALPACA_SECRET_KEY=" "$CONFIG_FILE" | cut -d'=' -f2)
    
    echo ""
    echo "💰 Live Trading Credentials:"
    if [ "$live_key" != "your_live_api_key_here" ] && [ -n "$live_key" ]; then
        echo "  ALPACA_API_KEY: ${live_key:0:8}...${live_key: -4}"
    else
        echo "  ALPACA_API_KEY: Not set"
    fi
    
    if [ "$live_secret" != "your_live_secret_key_here" ] && [ -n "$live_secret" ]; then
        echo "  ALPACA_SECRET_KEY: ${live_secret:0:8}...${live_secret: -4}"
    else
        echo "  ALPACA_SECRET_KEY: Not set"
    fi
    
    # Show other configuration
    echo ""
    echo "Other Settings:"
    grep -E "^(STREAM_TYPES|TRADING_SYMBOLS|UPDATE_INTERVAL_MS|MARKET_DATA_DIR|API_PORT|ALPACA_PAPER_TRADING)=" "$CONFIG_FILE"
}

# Main menu
while true; do
    echo ""
    echo "🚀 Trading Bot API Key Manager"
    echo "=============================="
    echo "1. Add/Update API Keys"
    echo "2. Add Multiple API Keys"
    echo "3. Show Current Configuration"
    echo "4. Validate Configuration"
    echo "5. Exit"
    echo ""
    read -p "Choose an option (1-5): " choice
    
            case $choice in
        1)
            echo ""
            echo "📝 Setting up Paper Trading credentials (recommended for testing)..."
            add_api_key "Alpaca Paper Trading API Key ID" "APCA-API-KEY-ID"
            add_api_key "Alpaca Paper Trading Secret Key" "APCA-API-SECRET-KEY"
            ;;
        2)
            add_multiple_keys
            ;;
        3)
            show_config
            ;;
        4)
            if validate_config; then
                echo ""
                echo "🎉 Configuration is ready! You can now run the trading bot."
            else
                echo ""
                echo "⚠️  Please complete the configuration before running the bot."
            fi
            ;;
        5)
            echo "👋 Goodbye!"
            exit 0
            ;;
        *)
            echo "❌ Invalid option. Please choose 1-5."
            ;;
    esac
done
