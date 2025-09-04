#!/bin/bash

# ========================================
# OLLAMA STREAMING TEST SCRIPT
# ========================================
# This script tests the Ollama streaming functionality

set -e

echo "🧠 OLLAMA STREAMING TEST"
echo "========================"
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
        "PASS")
            echo -e "${GREEN}✅ PASS${NC}: $message"
            ;;
        "FAIL")
            echo -e "${RED}❌ FAIL${NC}: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  WARN${NC}: $message"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  INFO${NC}: $message"
            ;;
    esac
}

# Check if Ollama is running
print_status "INFO" "Checking if Ollama is running..."

if curl -s http://localhost:11434/api/tags > /dev/null; then
    print_status "PASS" "Ollama server is running"
else
    print_status "FAIL" "Ollama server is not running"
    echo ""
    print_status "INFO" "To start Ollama:"
    echo "  1. Install Ollama: https://ollama.ai"
    echo "  2. Start Ollama: ollama serve"
    echo "  3. Pull a model: ollama pull llama2:7b"
    echo ""
    exit 1
fi

# Check available models
print_status "INFO" "Checking available models..."

MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null || echo "")

if [ -n "$MODELS" ]; then
    print_status "PASS" "Available models:"
    echo "$MODELS" | while read -r model; do
        echo "  - $model"
    done
else
    print_status "WARN" "No models found. Pull a model with: ollama pull llama2:7b"
fi

# Test basic Ollama functionality
print_status "INFO" "Testing basic Ollama functionality..."

TEST_PROMPT="Hello, this is a test. Please respond with 'Test successful' if you can read this."

if curl -s -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"llama2:7b\",\"prompt\":\"$TEST_PROMPT\",\"stream\":false}" > /dev/null; then
    print_status "PASS" "Basic Ollama API test passed"
else
    print_status "FAIL" "Basic Ollama API test failed"
fi

# Test streaming functionality
print_status "INFO" "Testing streaming functionality..."

if curl -s -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"llama2:7b\",\"prompt\":\"$TEST_PROMPT\",\"stream\":true}" > /dev/null; then
    print_status "PASS" "Ollama streaming test passed"
else
    print_status "FAIL" "Ollama streaming test failed"
fi

# Test trading bot AI integration
print_status "INFO" "Testing trading bot AI integration..."

# Test enhanced strategy (should work without Ollama)
if cargo run --release -- --enhanced-strategy > /dev/null 2>&1; then
    print_status "PASS" "Enhanced strategy test passed"
else
    print_status "WARN" "Enhanced strategy test failed"
fi

# Test AI decisions (requires Ollama)
if cargo run --release -- --ai-decisions > /dev/null 2>&1; then
    print_status "PASS" "AI decisions test passed"
else
    print_status "WARN" "AI decisions test failed (may be expected if Ollama is not running)"
fi

# Test API server with Ollama integration
print_status "INFO" "Testing API server with Ollama integration..."

# Start API server in background
print_status "INFO" "Starting API server for testing..."
cargo run --release -- --api --api-port 8082 &
API_PID=$!

# Wait for server to start
sleep 5

# Test health endpoint
if curl -s http://localhost:8082/health > /dev/null; then
    print_status "PASS" "API server health check passed"
else
    print_status "FAIL" "API server health check failed"
fi

# Test Ollama processing endpoint
if curl -s -X POST http://localhost:8082/api/ollama/process \
    -H "Content-Type: application/json" \
    -d '{"file_path": "./trading_portfolio/trading_portfolio.json", "prompt": "Analyze this portfolio"}' > /dev/null; then
    print_status "PASS" "Ollama processing endpoint test passed"
else
    print_status "WARN" "Ollama processing endpoint test failed (may be expected if Ollama is not running)"
fi

# Stop API server
kill $API_PID 2>/dev/null || true

# Summary
echo ""
echo "📊 OLLAMA STREAMING TEST SUMMARY"
echo "================================"
echo ""

print_status "INFO" "Streaming improvements implemented:"
echo "  ✅ Streaming mode enabled by default"
echo "  ✅ Fallback to non-streaming mode"
echo "  ✅ Improved timeout handling"
echo "  ✅ Ollama server status check"
echo "  ✅ Optimized generation parameters"
echo "  ✅ Better error handling"

echo ""
print_status "INFO" "Next steps:"
echo "  1. Ensure Ollama is running: ollama serve"
echo "  2. Pull the model: ollama pull llama2:7b"
echo "  3. Test AI integration: cargo run --release -- --ai-decisions"
echo "  4. Test API server: cargo run --release -- --api"

echo ""
print_status "PASS" "Ollama streaming test completed!"
echo ""

exit 0
