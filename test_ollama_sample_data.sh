#!/bin/bash

# 🧪 Ollama Sample Data Test
# Tests if the Ollama model can read and process the sample trading data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:3000"
SAMPLE_DATA_FILE="sample_data.json"
LOG_FILE="ollama_test_results.log"

echo -e "${BLUE}🧪 Ollama Sample Data Test${NC}"
echo "================================"
echo "Testing if Ollama can read and process sample trading data"
echo ""

# Function to log results
log_result() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    case $status in
        "PASS")
            echo -e "${GREEN}✅ PASS${NC}: $test_name - $message"
            ;;
        "FAIL")
            echo -e "${RED}❌ FAIL${NC}: $test_name - $message"
            ;;
        "SKIP")
            echo -e "${YELLOW}⚠️ SKIP${NC}: $test_name - $message"
            ;;
    esac
    
    # Log to file
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $status: $test_name - $message" >> "$LOG_FILE"
}

# Function to check if API is running
check_api_health() {
    echo -e "${BLUE}🔍 Checking API health...${NC}"
    
    if curl -s "$API_BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ API is running at $API_BASE_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ API is not running at $API_BASE_URL${NC}"
        echo "Please start the API server first:"
        echo "  cargo run -- --api"
        return 1
    fi
}

# Function to check Ollama
check_ollama() {
    echo -e "${BLUE}🔍 Checking Ollama...${NC}"
    
    if curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Ollama is running${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️ Ollama is not running${NC}"
        echo "Please start Ollama with:"
        echo "  ollama serve"
        return 1
    fi
}

# Function to check available models
check_models() {
    echo -e "${BLUE}🔍 Checking available Ollama models...${NC}"
    
    local models_response=$(curl -s "http://localhost:11434/api/tags" 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$models_response" ]; then
        echo -e "${GREEN}✅ Available models:${NC}"
        echo "$models_response" | jq -r '.models[]?.name // empty' 2>/dev/null || echo "$models_response"
        return 0
    else
        echo -e "${RED}❌ Failed to get models from Ollama${NC}"
        return 1
    fi
}

# Function to check sample data file
check_sample_data() {
    echo -e "${BLUE}🔍 Checking sample data file...${NC}"
    
    if [ -f "$SAMPLE_DATA_FILE" ]; then
        echo -e "${GREEN}✅ Sample data file found: $SAMPLE_DATA_FILE${NC}"
        echo -e "${BLUE}📊 Sample data content:${NC}"
        cat "$SAMPLE_DATA_FILE" | jq '.' 2>/dev/null || cat "$SAMPLE_DATA_FILE"
        echo ""
        return 0
    else
        echo -e "${RED}❌ Sample data file not found: $SAMPLE_DATA_FILE${NC}"
        return 1
    fi
}

# Function to test basic Ollama processing
test_basic_ollama() {
    echo -e "${BLUE}🧪 Testing basic Ollama processing...${NC}"
    
    local test_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities."
    
    echo "Sending test prompt to Ollama..."
    echo "Prompt: $test_prompt"
    echo ""
    
    # Get first available model
    local model_name=$(curl -s "http://localhost:11434/api/tags" 2>/dev/null | jq -r '.models[0]?.name // "llama3.2"')
    echo "Using model: $model_name"
    echo ""
    
    # Test direct Ollama API call
    local response=$(curl -s -X POST "http://localhost:11434/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$model_name\",
            \"prompt\": \"$test_prompt\",
            \"stream\": false
        }" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Ollama API call successful${NC}"
        echo -e "${BLUE}📝 Response:${NC}"
        echo "$response" | jq -r '.response // .error // "Unknown response"' 2>/dev/null || echo "$response"
        echo ""
        log_result "Basic Ollama Processing" "PASS" "Successfully called Ollama API"
        return 0
    else
        echo -e "${RED}❌ Ollama API call failed${NC}"
        log_result "Basic Ollama Processing" "FAIL" "Failed to call Ollama API"
        return 1
    fi
}

# Function to test trading bot API with sample data
test_trading_bot_api() {
    echo -e "${BLUE}🧪 Testing Trading Bot API with sample data...${NC}"
    
    local test_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities."
    
    echo "Sending sample data to Trading Bot API..."
    echo "File: $SAMPLE_DATA_FILE"
    echo "Prompt: $test_prompt"
    echo ""
    
    # Test the ultra-fast processing endpoint
    local response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d "{
            \"file_path\": \"$SAMPLE_DATA_FILE\",
            \"prompt\": \"$test_prompt\"
        }" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Trading Bot API call successful${NC}"
        echo -e "${BLUE}📝 Response:${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        echo ""
        log_result "Trading Bot API Processing" "PASS" "Successfully processed sample data through API"
        return 0
    else
        echo -e "${RED}❌ Trading Bot API call failed${NC}"
        log_result "Trading Bot API Processing" "FAIL" "Failed to process sample data through API"
        return 1
    fi
}

# Function to test different processing modes
test_processing_modes() {
    echo -e "${BLUE}🧪 Testing different processing modes...${NC}"
    
    local test_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities."
    local modes=("threaded" "ultra-fast" "ultra-threaded")
    
    for mode in "${modes[@]}"; do
        echo -e "${BLUE}Testing $mode mode...${NC}"
        
        local endpoint="$API_BASE_URL/api/ollama/process/$mode"
        local response=$(curl -s -X POST "$endpoint" \
            -H "Content-Type: application/json" \
            -d "{
                \"file_path\": \"$SAMPLE_DATA_FILE\",
                \"prompt\": \"$test_prompt\"
            }" 2>/dev/null)
        
        if [ $? -eq 0 ] && [ -n "$response" ]; then
            echo -e "${GREEN}✅ $mode mode successful${NC}"
            echo "Response status: $(echo "$response" | jq -r '.status // "unknown"' 2>/dev/null || echo "unknown")"
        else
            echo -e "${RED}❌ $mode mode failed${NC}"
        fi
        echo ""
    done
}

# Function to test multi-model conversation
test_multi_model() {
    echo -e "${BLUE}🧪 Testing multi-model conversation...${NC}"
    
    local test_prompt="Analyze this trading data and provide insights about the market conditions, price trends, and trading opportunities."
    
    echo "Testing multi-model conversation with sample data..."
    echo ""
    
    local response=$(curl -s -X POST "$API_BASE_URL/api/ollama/conversation" \
        -H "Content-Type: application/json" \
        -d "{
            \"file_path\": \"$SAMPLE_DATA_FILE\",
            \"initial_prompt\": \"$test_prompt\",
            \"models\": [\"llama3.2\", \"llama3.2\"],
            \"conversation_type\": \"analysis\",
            \"conversation_rounds\": 2
        }" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Multi-model conversation successful${NC}"
        echo -e "${BLUE}📝 Response:${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        echo ""
        log_result "Multi-Model Conversation" "PASS" "Successfully tested multi-model conversation"
        return 0
    else
        echo -e "${RED}❌ Multi-model conversation failed${NC}"
        log_result "Multi-Model Conversation" "FAIL" "Failed to test multi-model conversation"
        return 1
    fi
}

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Ollama Sample Data Test Summary${NC}"
    echo "========================================="
    echo ""
    
    if [ -f "$LOG_FILE" ]; then
        echo "Test results logged to: $LOG_FILE"
        echo ""
        echo "Recent test results:"
        tail -10 "$LOG_FILE" 2>/dev/null || echo "No log file found"
    fi
    
    echo ""
    echo -e "${GREEN}🎉 Ollama sample data test completed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Check the responses above to see if Ollama understood the trading data"
    echo "2. Try different prompts with the sample data"
    echo "3. Test with your own trading data files"
    echo "4. Monitor performance metrics in the API responses"
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting Ollama Sample Data Test${NC}"
    echo ""
    
    # Clear log file
    > "$LOG_FILE"
    
    # Check prerequisites
    check_api_health || exit 1
    check_ollama || exit 1
    check_models
    check_sample_data || exit 1
    echo ""
    
    # Run tests
    test_basic_ollama
    echo ""
    
    test_trading_bot_api
    echo ""
    
    test_processing_modes
    echo ""
    
    test_multi_model
    echo ""
    
    # Show summary
    show_summary
}

# Run main function
main "$@"
