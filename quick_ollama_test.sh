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
    curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL\",
            \"prompt\": \"Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities. Here is the data: $(cat sample_data.json)\",
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
