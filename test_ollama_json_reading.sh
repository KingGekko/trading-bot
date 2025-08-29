#!/bin/bash

# 🤖 Test Ollama JSON Reading Capabilities
# This script tests if your AI models can read and process JSON data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:8080"
SAMPLE_FILE="sample_data.json"
TEST_FILES=("test_simple.json" "test_complex.json" "live_data.json")

echo -e "${BLUE}🤖 Testing Ollama JSON Reading Capabilities${NC}"
echo "=================================================="
echo "This script will test if your AI models can read and process JSON data"
echo ""

# Function to check if API is running
check_api_health() {
    echo -e "${BLUE}🔍 Step 1: Check API Health${NC}"
    
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

# Function to check if Ollama is running
check_ollama_health() {
    echo -e "${BLUE}🔍 Step 2: Check Ollama Health${NC}"
    
    if curl -s "http://localhost:11434/api/tags" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Ollama is running${NC}"
        
        # Get available models
        models_response=$(curl -s "http://localhost:11434/api/tags" 2>/dev/null)
        if [ -n "$models_response" ]; then
            echo "Available models:"
            echo "$models_response" | jq -r '.models[]?.name // empty' 2>/dev/null || echo "Could not parse models"
        fi
        return 0
    else
        echo -e "${RED}❌ Ollama is not running${NC}"
        echo "Please start Ollama:"
        echo "  ollama serve"
        return 1
    fi
}

# Function to create test JSON files
create_test_files() {
    echo -e "${BLUE}🔍 Step 3: Create Test JSON Files${NC}"
    
    # Create simple test file
    cat > test_simple.json << EOF
{
  "price": 45000,
  "volume": 1250,
  "trend": "up",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
    
    # Create complex test file
    cat > test_complex.json << EOF
{
  "market_data": {
    "btc_price": 45000,
    "eth_price": 3200,
    "volume_24h": 2500000000,
    "market_cap": 850000000000
  },
  "technical_indicators": {
    "rsi": 65,
    "macd": "bullish",
    "bollinger_bands": {
      "upper": 46000,
      "middle": 45000,
      "lower": 44000
    }
  },
  "sentiment": "positive",
  "news_events": ["ETF approval", "Institutional adoption"],
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
    
    # Create live data file
    cat > live_data.json << EOF
{"price": 45000, "status": "stable", "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"}
EOF
    
    echo -e "${GREEN}✅ Created test files:${NC}"
    for file in "${TEST_FILES[@]}"; do
        if [ -f "$file" ]; then
            echo "  ✅ $file ($(wc -c < "$file") bytes)"
        else
            echo "  ❌ $file (failed to create)"
        fi
    done
    echo ""
}

# Function to test basic Ollama processing
test_basic_processing() {
    echo -e "${BLUE}🔍 Step 4: Test Basic Ollama Processing${NC}"
    
    echo "Testing basic JSON processing with sample_data.json..."
    
    response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d '{
            "file_path": "sample_data.json",
            "prompt": "Analyze this trading data and provide insights about market conditions, price trends, and trading opportunities."
        }' 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Basic processing successful${NC}"
        echo "Response:"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Basic processing failed${NC}"
        echo "Response: $response"
    fi
    
    echo ""
}

# Function to test different processing modes
test_processing_modes() {
    echo -e "${BLUE}🔍 Step 5: Test Different Processing Modes${NC}"
    
    local modes=("threaded" "ultra-fast" "ultra-threaded")
    local test_file="test_complex.json"
    
    for mode in "${modes[@]}"; do
        echo "Testing $mode processing mode..."
        
        response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process/$mode" \
            -H "Content-Type: application/json" \
            -d "{
                \"file_path\": \"$test_file\",
                \"prompt\": \"Analyze this data using $mode processing and provide insights.\"
            }" 2>/dev/null)
        
        if [ $? -eq 0 ] && [ -n "$response" ]; then
            echo -e "${GREEN}✅ $mode processing successful${NC}"
            echo "Response length: $(echo "$response" | wc -c) bytes"
        else
            echo -e "${RED}❌ $mode processing failed${NC}"
            echo "Response: $response"
        fi
        
        echo ""
    done
}

# Function to test multi-model conversations
test_multi_model_conversations() {
    echo -e "${BLUE}🔍 Step 6: Test Multi-Model Conversations${NC}"
    
    echo "Testing multi-model AI conversation..."
    
    response=$(curl -s -X POST "$API_BASE_URL/api/ollama/conversation" \
        -H "Content-Type: application/json" \
        -d '{
            "file_path": "test_complex.json",
            "initial_prompt": "Analyze this trading data together and debate the best trading strategy.",
            "models": ["tinyllama", "phi3"],
            "conversation_type": "debate"
        }' 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✅ Multi-model conversation successful${NC}"
        echo "Response:"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Multi-model conversation failed${NC}"
        echo "Response: $response"
    fi
    
    echo ""
}

# Function to test real-time updates
test_realtime_updates() {
    echo -e "${BLUE}🔍 Step 7: Test Real-Time Updates${NC}"
    
    echo "Testing real-time data processing..."
    
    # Start watching the live data file
    echo "Starting file watch..."
    watch_response=$(curl -s -X POST "$API_BASE_URL/api/watch" \
        -H "Content-Type: application/json" \
        -d '{"file_path": "live_data.json"}' 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$watch_response" ]; then
        echo -e "${GREEN}✅ File watch started${NC}"
        
        # Process initial data
        echo "Processing initial data..."
        initial_response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
            -H "Content-Type: application/json" \
            -d '{
                "file_path": "live_data.json",
                "prompt": "What is the current market status?"
            }' 2>/dev/null)
        
        if [ $? -eq 0 ] && [ -n "$initial_response" ]; then
            echo -e "${GREEN}✅ Initial processing successful${NC}"
        else
            echo -e "${RED}❌ Initial processing failed${NC}"
        fi
        
        # Update the file (simulate real-time change)
        echo "Updating file with new data..."
        cat > live_data.json << EOF
{"price": 45100, "status": "rising", "change": "+100", "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"}
EOF
        
        # Wait a moment for the system to detect changes
        sleep 2
        
        # Process the updated data
        echo "Processing updated data..."
        updated_response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
            -H "Content-Type: application/json" \
            -d '{
                "file_path": "live_data.json",
                "prompt": "What changed and what does it mean for trading?"
            }' 2>/dev/null)
        
        if [ $? -eq 0 ] && [ -n "$updated_response" ]; then
            echo -e "${GREEN}✅ Updated data processing successful${NC}"
        else
            echo -e "${RED}❌ Updated data processing failed${NC}"
        fi
        
    else
        echo -e "${RED}❌ Failed to start file watch${NC}"
    fi
    
    echo ""
}

# Function to test different JSON formats
test_json_formats() {
    echo -e "${BLUE}🔍 Step 8: Test Different JSON Formats${NC}"
    
    # Test with simple data
    echo "Testing simple JSON format..."
    simple_response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d '{
            "file_path": "test_simple.json",
            "prompt": "What does this simple data tell you?"
        }' 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$simple_response" ]; then
        echo -e "${GREEN}✅ Simple JSON processing successful${NC}"
    else
        echo -e "${RED}❌ Simple JSON processing failed${NC}"
    fi
    
    # Test with complex data
    echo "Testing complex JSON format..."
    complex_response=$(curl -s -X POST "$API_BASE_URL/api/ollama/process" \
        -H "Content-Type: application/json" \
        -d '{
            "file_path": "test_complex.json",
            "prompt": "Analyze this complex market data and provide trading insights."
        }' 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$complex_response" ]; then
        echo -e "${GREEN}✅ Complex JSON processing successful${NC}"
    else
        echo -e "${RED}❌ Complex JSON processing failed${NC}"
    fi
    
    echo ""
}

# Function to cleanup
cleanup() {
    echo -e "${BLUE}🧹 Cleanup${NC}"
    
    # Stop watching test files
    for file in "${TEST_FILES[@]}"; do
        curl -s -X GET "$API_BASE_URL/api/watch/$file" >/dev/null 2>&1 || true
    done
    
    # Remove test files
    for file in "${TEST_FILES[@]}"; do
        if [ -f "$file" ]; then
            rm -f "$file"
            echo -e "${GREEN}✅ Removed $file${NC}"
        fi
    done
    
    echo ""
}

# Function to show test summary
show_summary() {
    echo -e "${BLUE}📊 Ollama JSON Reading Test Summary${NC}"
    echo "=========================================="
    echo ""
    
    echo "💡 What we tested:"
    echo "  1. ✅ API health and Ollama connectivity"
    echo "  2. ✅ Basic JSON processing capabilities"
    echo "  3. ✅ Different processing modes (threaded, ultra-fast, ultra-threaded)"
    echo "  4. ✅ Multi-model AI conversations"
    echo "  5. ✅ Real-time data updates and processing"
    echo "  6. ✅ Different JSON formats (simple vs complex)"
    echo ""
    
    echo "🚀 If all tests passed, your AI models can successfully:"
    echo "  - Read and parse JSON data"
    echo "  - Process different data formats"
    echo "  - Provide insights and analysis"
    echo "  - Handle real-time updates"
    echo "  - Work with multiple AI models"
    echo ""
    
    echo "🔧 Next steps:"
    echo "  - Test with your actual trading data"
    echo "  - Integrate with real-time data feeds"
    echo "  - Build trading dashboards"
    echo "  - Implement automated trading strategies"
}

# Main execution
main() {
    echo -e "${BLUE}🤖 Starting Ollama JSON Reading Tests${NC}"
    echo ""
    
    # Run all test steps
    check_api_health || exit 1
    check_ollama_health || exit 1
    create_test_files
    
    test_basic_processing
    test_processing_modes
    test_multi_model_conversations
    test_realtime_updates
    test_json_formats
    
    # Cleanup and show summary
    cleanup
    show_summary
    
    echo -e "${BLUE}🤖 Ollama JSON Reading Tests Complete${NC}"
}

# Run main function
main "$@"
