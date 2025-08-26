#!/bin/bash
# Trading Bot - Installation Test Script
# This script tests the complete installation

set -e  # Exit on any error

echo "🧪 Trading Bot - Installation Test"
echo "=================================="

# Check if we're in the setup directory
if [ ! -d "../trading-bot" ] && [ ! -f "../target/release/trading_bot" ]; then
    if [ -d "trading-bot" ]; then
        cd trading-bot
    else
        echo "❌ Trading bot not found!"
        echo "🔧 Please run ./clone_and_build.sh first"
        exit 1
    fi
fi

# Find the trading bot binary
TRADING_BOT_PATH=""
if [ -f "target/release/trading_bot" ]; then
    TRADING_BOT_PATH="./target/release/trading_bot"
elif [ -f "../trading-bot/target/release/trading_bot" ]; then
    TRADING_BOT_PATH="../trading-bot/target/release/trading_bot"
else
    echo "❌ Trading bot binary not found!"
    echo "🔧 Please run ./clone_and_build.sh first"
    exit 1
fi

echo "📍 Trading bot found at: $TRADING_BOT_PATH"

# Test 1: Check binary
echo ""
echo "🧪 Test 1: Binary functionality"
echo "-------------------------------"
$TRADING_BOT_PATH --help
echo "✅ Binary is working!"

# Test 2: Check Ollama
echo ""
echo "🧪 Test 2: Ollama connectivity"
echo "------------------------------"
if ! command -v ollama &> /dev/null; then
    echo "❌ Ollama not found!"
    echo "🔧 Please run ./install_ollama.sh first"
    exit 1
fi

echo "📋 Ollama version:"
ollama --version

echo "📋 Available models:"
ollama list

# Test 3: Quick response test
echo ""
echo "🧪 Test 3: Quick response test"
echo "------------------------------"
echo "⏳ Testing with prompt: 'What is blockchain?'"
echo "📊 Expected: 8-12 second response with good analysis"
echo ""

# Run the test
$TRADING_BOT_PATH -t "What is blockchain?"

echo ""
echo "🧪 Test 4: Performance check"
echo "----------------------------"
echo "📊 Performance expectations:"
echo "   • Response time: 8-12 seconds (balanced mode)"
echo "   • Analysis quality: ⭐⭐⭐ Good structured analysis"
echo "   • Response length: ~150-200 words"
echo "   • Streaming: Real-time output during generation"

echo ""
echo "🎉 ALL TESTS COMPLETED!"
echo "========================"
echo ""
echo "✅ Your trading bot is fully operational!"
echo ""
echo "📋 Quick Reference:"
echo "   • Test mode:        $TRADING_BOT_PATH -t 'Your prompt'"
echo "   • Interactive mode: $TRADING_BOT_PATH -i"
echo "   • Single prompt:    $TRADING_BOT_PATH -p 'Your prompt'"
echo "   • View logs:        $TRADING_BOT_PATH -l"
echo ""
echo "🔧 Configuration file: $(dirname $TRADING_BOT_PATH)/../config.env"
echo "📁 Log directory: $(dirname $TRADING_BOT_PATH)/../ollama_logs/"
echo ""
echo "💡 Tips:"
echo "   • For faster responses: Set OLLAMA_MODEL=tinyllama in config.env"
echo "   • For better analysis: Set OLLAMA_MODEL=phi in config.env"
echo "   • For system-wide access: sudo cp $TRADING_BOT_PATH /usr/local/bin/"
echo ""
echo "🚀 Happy trading!"