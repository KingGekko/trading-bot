#!/bin/bash

# 🧪 Test Stream Updates
# Generates sample updates for testing the 4 JSON streams

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LIVE_DATA_DIR="live_data"
UPDATE_INTERVAL=3  # Update every 3 seconds

echo -e "${BLUE}🧪 Test Stream Updates${NC}"
echo "========================="
echo "Generating sample updates for 4 JSON streams"
echo "Update interval: ${UPDATE_INTERVAL} seconds"
echo "Press Ctrl+C to stop"
echo ""

# Function to update crypto data
update_crypto() {
    local symbol=$1
    local file_path="$LIVE_DATA_DIR/crypto_data_${symbol,,}.json"
    
    # Generate random price movement
    local current_price=$(jq -r '.price' "$file_path" 2>/dev/null || echo "45000")
    local change_percent=$((RANDOM % 10 - 5))  # -5% to +5%
    local new_price=$(echo "$current_price * (1 + $change_percent / 100)" | bc -l)
    local new_volume=$((RANDOM % 1000 + 500))
    
    # Update the file
    jq --arg price "$(printf "%.2f" $new_price)" \
       --arg volume "$new_volume" \
       --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
       '.price = ($price | tonumber) | .volume = ($volume | tonumber) | .timestamp = $timestamp' \
       "$file_path" > "${file_path}.tmp" && mv "${file_path}.tmp" "$file_path"
    
    echo -e "${GREEN}📝 Updated $symbol: $${new_price} (${change_percent:+${change_percent}}%)${NC}"
}

# Function to update stock data
update_stock() {
    local symbol=$1
    local file_path="$LIVE_DATA_DIR/stock_data_${symbol,,}.json"
    
    # Generate random price movement
    local current_price=$(jq -r '.price' "$file_path" 2>/dev/null || echo "150.00")
    local change_percent=$((RANDOM % 6 - 3))  # -3% to +3%
    local new_price=$(echo "$current_price * (1 + $change_percent / 100)" | bc -l)
    local new_volume=$((RANDOM % 10000 + 5000))
    
    # Update the file
    jq --arg price "$(printf "%.2f" $new_price)" \
       --arg volume "$new_volume" \
       --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
       '.price = ($price | tonumber) | .volume = ($volume | tonumber) | .timestamp = $timestamp' \
       "$file_path" > "${file_path}.tmp" && mv "${file_path}.tmp" "$file_path"
    
    echo -e "${GREEN}📝 Updated $symbol: $${new_price} (${change_percent:+${change_percent}}%)${NC}"
}

# Function to update options data
update_options() {
    local file_path="$LIVE_DATA_DIR/options_data_spy.json"
    
    # Generate random options data
    local current_price=$(jq -r '.price' "$file_path" 2>/dev/null || echo "450.00")
    local change_percent=$((RANDOM % 8 - 4))  # -4% to +4%
    local new_price=$(echo "$current_price * (1 + $change_percent / 100)" | bc -l)
    local new_volume=$((RANDOM % 500 + 100))
    
    # Update the file
    jq --arg price "$(printf "%.2f" $new_price)" \
       --arg volume "$new_volume" \
       --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
       '.price = ($price | tonumber) | .volume = ($volume | tonumber) | .timestamp = $timestamp' \
       "$file_path" > "${file_path}.tmp" && mv "${file_path}.tmp" "$file_path"
    
    echo -e "${GREEN}📝 Updated SPY Options: $${new_price} (${change_percent:+${change_percent}}%)${NC}"
}

# Function to update news data
update_news() {
    local symbol=$1
    local file_path="$LIVE_DATA_DIR/news_data_${symbol,,}.json"
    
    # Generate random news sentiment
    local current_sentiment=$(jq -r '.sentiment // 0' "$file_path" 2>/dev/null || echo "0")
    local new_sentiment=$((RANDOM % 200 - 100))  # -100 to +100
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    # Update the file
    jq --arg sentiment "$new_sentiment" \
       --arg timestamp "$timestamp" \
       '.sentiment = ($sentiment | tonumber) | .timestamp = $timestamp' \
       "$file_path" > "${file_path}.tmp" && mv "${file_path}.tmp" "$file_path"
    
    local sentiment_emoji=""
    if [ $new_sentiment -gt 50 ]; then
        sentiment_emoji="📈"
    elif [ $new_sentiment -lt -50 ]; then
        sentiment_emoji="📉"
    else
        sentiment_emoji="➡️"
    fi
    
    echo -e "${GREEN}📝 Updated $symbol News: Sentiment ${sentiment_emoji} $new_sentiment${NC}"
}

# Main update loop
echo -e "${YELLOW}🚀 Starting test updates...${NC}"
echo ""

while true; do
    echo -e "${BLUE}=== Update Cycle: $(date) ===${NC}"
    
    # Update all streams
    update_crypto "BTC"
    update_crypto "ETH"
    update_stock "AAPL"
    update_options
    update_news "AAPL"
    update_news "SPY"
    
    echo ""
    echo -e "${YELLOW}⏳ Waiting ${UPDATE_INTERVAL} seconds for next update...${NC}"
    echo ""
    
    sleep $UPDATE_INTERVAL
done
