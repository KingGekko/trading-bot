#!/bin/bash

# 🚀 Quick Ollama Sample Data Test
# Simple test to see if Ollama can read and process the sample trading data

# Parse command line arguments
SELECTED_MODEL=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--model)
            SELECTED_MODEL="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [-m|--model MODEL_NAME]"
            echo "  -m, --model MODEL_NAME  Specify model to use (e.g., llama3.2, mistral, tinyllama)"
            echo "  -h, --help              Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Interactive model selection"
            echo "  $0 -m llama3.2        # Use llama3.2 model"
            echo "  $0 --model tinyllama  # Use tinyllama model"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

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
    
    # Get available models and let user select
    echo "🔍 Available models:"
    local models_response=$(curl -s "http://localhost:11434/api/tags" 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$models_response" ]; then
        # Extract model names
        local model_names=($(echo "$models_response" | jq -r '.models[]?.name // empty' 2>/dev/null))
        
        if [ ${#model_names[@]} -eq 0 ]; then
            echo "❌ No models found"
            exit 1
        fi
        
        # Check if user specified a model via command line
        if [ -n "$SELECTED_MODEL" ]; then
            # Check if specified model exists
            if [[ " ${model_names[@]} " =~ " ${SELECTED_MODEL} " ]]; then
                MODEL="$SELECTED_MODEL"
                echo "✅ Using specified model: $MODEL"
            else
                echo "❌ Specified model '$SELECTED_MODEL' not found"
                echo "Available models:"
                for i in "${!model_names[@]}"; do
                    echo "  $((i+1)). ${model_names[$i]}"
                done
                echo ""
                echo "Please specify a valid model name or run without -m for interactive selection"
                exit 1
            fi
        else
            # Display models with numbers
            echo "Available models:"
            for i in "${!model_names[@]}"; do
                echo "  $((i+1)). ${model_names[$i]}"
            done
            echo ""
            
            # Let user select model
            if [ -t 0 ]; then
                # Interactive mode
                read -p "Select model (1-${#model_names[@]}): " selection
                if [[ "$selection" =~ ^[0-9]+$ ]] && [ "$selection" -ge 1 ] && [ "$selection" -le ${#model_names[@]} ]; then
                    MODEL="${model_names[$((selection-1))]}"
                    echo "✅ Selected model: $MODEL"
                else
                    echo "❌ Invalid selection, using first model: ${model_names[0]}"
                    MODEL="${model_names[0]}"
                fi
            else
                # Non-interactive mode, use first model
                MODEL="${model_names[0]}"
                echo "✅ Using first available model: $MODEL"
            fi
        fi
    else
        echo "❌ Failed to get models, using default: llama3.2"
        MODEL="llama3.2"
    fi
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
    
    # Test with clean prompt (extracted values)
    echo "📝 Testing with extracted data values..."
    curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL\",
            \"prompt\": \"$clean_prompt\",
            \"stream\": false
        }" | jq -r '.response // .error // "Unknown response"' 2>/dev/null || echo "Failed to get response"
    
    echo ""
    echo "📝 Now testing with full file content..."
    
    # Test with actual file content (full JSON data)
    full_file_content=$(cat sample_data.json | jq -c '.' 2>/dev/null || cat sample_data.json)
    full_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities. Here is the complete data: $full_file_content"
    
    curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL\",
            \"prompt\": \"$full_prompt\",
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
echo ""
echo "💡 Model Selection Tips:"
echo "  - Run without options for interactive selection"
echo "  - Use -m MODEL_NAME to specify a model directly"
echo "  - Use -h or --help to see all options"
