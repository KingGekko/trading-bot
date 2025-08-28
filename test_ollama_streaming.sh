#!/bin/bash

# 🚀 Ollama Streaming File Test
# Tests streaming file content to Ollama in real-time

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SAMPLE_DATA_FILE="sample_data.json"

echo -e "${BLUE}🚀 Ollama Streaming File Test${NC}"
echo "================================="
echo "Testing streaming file content to Ollama"
echo ""

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
            echo "  -m, --model MODEL_NAME  Specify model to use"
            echo "  -h, --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

# Check if sample data exists
if [ ! -f "$SAMPLE_DATA_FILE" ]; then
    echo -e "${RED}❌ Sample data file not found: $SAMPLE_DATA_FILE${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Sample data file found${NC}"
echo ""

# Check if Ollama is running
echo -e "${BLUE}🔍 Checking if Ollama is running...${NC}"
if curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Ollama is running${NC}"
    
    # Get available models and let user select
    echo -e "${BLUE}🔍 Available models:${NC}"
    models_response=$(curl -s "http://localhost:11434/api/tags" 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$models_response" ]; then
        # Extract model names
        model_names=($(echo "$models_response" | jq -r '.models[]?.name // empty' 2>/dev/null))
        
        if [ ${#model_names[@]} -eq 0 ]; then
            echo -e "${RED}❌ No models found${NC}"
            exit 1
        fi
        
        # Check if user specified a model via command line
        if [ -n "$SELECTED_MODEL" ]; then
            # Check if specified model exists
            if [[ " ${model_names[@]} " =~ " ${SELECTED_MODEL} " ]]; then
                MODEL="$SELECTED_MODEL"
                echo -e "${GREEN}✅ Using specified model: $MODEL${NC}"
            else
                echo -e "${RED}❌ Specified model '$SELECTED_MODEL' not found${NC}"
                echo "Available models:"
                for i in "${!model_names[@]}"; do
                    echo "  $((i+1)). ${model_names[$i]}"
                done
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
                    echo -e "${GREEN}✅ Selected model: $MODEL${NC}"
                else
                    echo -e "${RED}❌ Invalid selection, using first model: ${model_names[0]}${NC}"
                    MODEL="${model_names[0]}"
                fi
            else
                # Non-interactive mode, use first model
                MODEL="${model_names[0]}"
                echo -e "${GREEN}✅ Using first available model: $MODEL${NC}"
            fi
        fi
    else
        echo -e "${RED}❌ Failed to get models, using default: llama3.2${NC}"
        MODEL="llama3.2"
    fi
    echo ""
    
    # Test 1: Stream file content line by line
    echo -e "${BLUE}🧪 Test 1: Streaming file content line by line...${NC}"
    echo "Reading $SAMPLE_DATA_FILE and sending to Ollama..."
    echo ""
    
    # Read file and send each line to Ollama
    line_number=1
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            echo -e "${YELLOW}📤 Sending line $line_number to Ollama...${NC}"
            echo "Content: $line"
            
            # Send line to Ollama
            response=$(curl -s -X POST "http://localhost:11434/api/generate" \
                -H "Content-Type: application/json" \
                -d "{
                    \"model\": \"$MODEL\",
                    \"prompt\": \"Analyze this line of trading data: $line\",
                    \"stream\": false
                }" 2>/dev/null)
            
            if [ $? -eq 0 ] && [ -n "$response" ]; then
                echo -e "${GREEN}✅ Response:${NC}"
                echo "$response" | jq -r '.response // .error // "Unknown response"' 2>/dev/null || echo "$response"
            else
                echo -e "${RED}❌ Failed to get response${NC}"
            fi
            echo ""
            ((line_number++))
        fi
    done < "$SAMPLE_DATA_FILE"
    
    # Test 2: Stream with context accumulation
    echo -e "${BLUE}🧪 Test 2: Streaming with context accumulation...${NC}"
    echo "Building context progressively..."
    echo ""
    
    # Read file and build context
    context=""
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            context="$context $line"
            echo -e "${YELLOW}📚 Context built (${#context} chars):${NC}"
            echo "Current context: ${context:0:100}..."
            
            # Send accumulated context to Ollama
            response=$(curl -s -X POST "http://localhost:11434/api/generate" \
                -H "Content-Type: application/json" \
                -d "{
                    \"model\": \"$MODEL\",
                    \"prompt\": \"Based on this accumulated trading data context, provide insights: $context\",
                    \"stream\": false
                }" 2>/dev/null)
            
            if [ $? -eq 0 ] && [ -n "$response" ]; then
                echo -e "${GREEN}✅ Progressive analysis:${NC}"
                echo "$response" | jq -r '.response // .error // "Unknown response"' 2>/dev/null | head -3
                echo "..."
            fi
            echo ""
        fi
    done < "$SAMPLE_DATA_FILE"
    
    # Test 3: Real-time file watching and streaming
    echo -e "${BLUE}🧪 Test 3: Real-time file watching simulation...${NC}"
    echo "Simulating real-time data updates..."
    echo ""
    
    # Create a temporary test file for real-time updates
    temp_file=$(mktemp)
    echo '{"timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'", "test": "real-time-update"}' > "$temp_file"
    
    echo "📁 Created test file: $temp_file"
    echo "📤 Sending real-time update to Ollama..."
    
    response=$(curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL\",
            \"prompt\": \"New real-time trading data update received: $(cat $temp_file). Analyze this update.\",
            \"stream\": false
        }" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Real-time analysis:${NC}"
        echo "$response" | jq -r '.response // .error // "Unknown response"' 2>/dev/null || echo "$response"
    fi
    
    # Clean up
    rm -f "$temp_file"
    
else
    echo -e "${RED}❌ Ollama is not running${NC}"
    echo "Please start Ollama with: ollama serve"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Streaming file tests completed!${NC}"
echo ""
echo "💡 What we tested:"
echo "  1. Line-by-line streaming to Ollama"
echo "  2. Context accumulation streaming"
echo "  3. Real-time file update simulation"
echo ""
echo "🚀 Next steps:"
echo "  - Use Trading Bot API for production streaming"
echo "  - Implement file watchers for real-time updates"
echo "  - Set up WebSocket streaming for live data"
