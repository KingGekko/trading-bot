#!/bin/bash

# 🚀 Simple API Key Setup Script
# Run this in your VM to easily configure multiple API keys

echo "🚀 Trading Bot API Key Setup"
echo "=============================="

# Check if config.env exists
if [ ! -f "config.env" ]; then
    echo "❌ config.env not found. Please run this script from the project directory."
    exit 1
fi

# Function to add API key
add_api_key() {
    local key_name=$1
    local env_var=$2
    local current_value=$(grep "^${env_var}=" config.env | cut -d'=' -f2)
    
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
            sed -i '' "s/^${env_var}=.*/${env_var}=${api_key}/" config.env
        else
            # Linux
            sed -i "s/^${env_var}=.*/${env_var}=${api_key}/" config.env
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
                echo "${key_name^^}=${key_value}" >> config.env
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
    
    # Check required fields
    local alpaca_key=$(grep "^ALPACA_API_KEY=" config.env | cut -d'=' -f2)
    local alpaca_secret=$(grep "^ALPACA_SECRET_KEY=" config.env | cut -d'=' -f2)
    
    if [ "$alpaca_key" = "your_alpaca_api_key_here" ] || [ -z "$alpaca_key" ]; then
        echo "❌ ALPACA_API_KEY not set"
        return 1
    fi
    
    if [ "$alpaca_secret" = "your_alpaca_secret_key_here" ] || [ -z "$alpaca_secret" ]; then
        echo "❌ ALPACA_SECRET_KEY not set"
        return 1
    fi
    
    echo "✅ All required API keys are configured!"
    echo "✅ Configuration is ready to use!"
    return 0
}

# Function to show current configuration
show_config() {
    echo ""
    echo "📋 Current Configuration"
    echo "========================"
    
    # Show API keys (masked)
    local alpaca_key=$(grep "^ALPACA_API_KEY=" config.env | cut -d'=' -f2)
    local alpaca_secret=$(grep "^ALPACA_SECRET_KEY=" config.env | cut -d'=' -f2)
    
    if [ "$alpaca_key" != "your_alpaca_api_key_here" ] && [ -n "$alpaca_key" ]; then
        echo "ALPACA_API_KEY: ${alpaca_key:0:8}...${alpaca_key: -4}"
    else
        echo "ALPACA_API_KEY: Not set"
    fi
    
    if [ "$alpaca_secret" != "your_alpaca_secret_key_here" ] && [ -n "$alpaca_secret" ]; then
        echo "ALPACA_SECRET_KEY: ${alpaca_secret:0:8}...${alpaca_secret: -4}"
    else
        echo "ALPACA_SECRET_KEY: Not set"
    fi
    
    # Show other configuration
    echo ""
    echo "Other Settings:"
    grep -E "^(STREAM_TYPES|TRADING_SYMBOLS|UPDATE_INTERVAL_MS|MARKET_DATA_DIR|API_PORT)=" config.env
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
            add_api_key "Alpaca API Key" "ALPACA_API_KEY"
            add_api_key "Alpaca Secret Key" "ALPACA_SECRET_KEY"
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
