#!/bin/bash

# 🚀 Quick Ollama Sample Data Test
# Simple test to see if Ollama can read and process the sample trading data

echo "🧪 Quick Ollama Sample Data Test"
echo "================================"
echo ""

# Check if sample data exists
if [ ! -f "sample_data.json" ]; then
    echo "❌ Sample data file not found: sample_data.json"
    exit 1
fi

echo "✅ Sample data file found"
echo ""

# Show sample data
echo "📊 Sample data content:"
cat sample_data.json | jq '.' 2>/dev/null || cat sample_data.json
echo ""

# Check if Ollama is running
echo "🔍 Checking if Ollama is running..."
if curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
    echo "✅ Ollama is running"
    
    # Get available models
    echo "🔍 Available models:"
    curl -s "http://localhost:11434/api/tags" | jq -r '.models[]?.name // empty' 2>/dev/null || curl -s "http://localhost:11434/api/tags"
    echo ""
    
    # Test basic Ollama processing
    echo "🧪 Testing basic Ollama processing..."
    echo "Prompt: Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities."
    echo ""
    
    # Get first available model
    MODEL=$(curl -s "http://localhost:11434/api/tags" | jq -r '.models[0]?.name // "llama3.2"' 2>/dev/null)
    echo "Using model: $MODEL"
    echo ""
    
    # Test direct Ollama API call
    echo "📝 Ollama Response:"
    
    # Extract key data points and create a clean prompt
    BTC_PRICE=$(cat sample_data.json | jq -r '.price // "unknown"' 2>/dev/null || echo "unknown")
    BTC_SYMBOL=$(cat sample_data.json | jq -r '.symbol // "unknown"' 2>/dev/null || echo "unknown")
    RSI_VALUE=$(cat sample_data.json | jq -r '.indicators.rsi // "unknown"' 2>/dev/null || echo "unknown")
    MACD_VALUE=$(cat sample_data.json | jq -r '.indicators.macd // "unknown"' 2>/dev/null || echo "unknown")
    SENTIMENT=$(cat sample_data.json | jq -r '.sentiment // "unknown"' 2>/dev/null || echo "unknown")
    
    echo "Extracted data: $BTC_SYMBOL at $BTC_PRICE, RSI: $RSI_VALUE, MACD: $MACD_VALUE, Sentiment: $SENTIMENT"
    echo ""
    
    # Create a clean prompt with extracted values
    clean_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities. Here is the data: $BTC_SYMBOL price is $BTC_PRICE, RSI is $RSI_VALUE, MACD is $MACD_VALUE, sentiment is $SENTIMENT."
    
    # Test with clean prompt
    curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL\",
            \"prompt\": \"$clean_prompt\",
            \"stream\": false
        }" | jq -r '.response // .error // "Unknown response"' 2>/dev/null || echo "Failed to get response"
    
else
    echo "❌ Ollama is not running"
    echo "Please start Ollama with: ollama serve"
    exit 1
fi

echo ""
echo "🎉 Quick test completed!"
echo ""
echo "To test the full Trading Bot API:"
echo "1. Start the server: cargo run -- --api"
echo "2. Run: ./test_ollama_sample_data.sh"
