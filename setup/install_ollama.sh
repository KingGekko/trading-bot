#!/bin/bash
# Trading Bot - Ollama Installation Script
# This script installs and configures Ollama AI

set -e  # Exit on any error

echo "🤖 Trading Bot - Ollama Installation"
echo "===================================="

# Check if Ollama is already installed
if command -v ollama &> /dev/null; then
    echo "✅ Ollama is already installed!"
    echo "📋 Checking status..."
    ollama --version
    echo ""
    echo "🔍 Available models:"
    ollama list
    echo ""
    echo "🎯 Skip to: Run ./test_installation.sh"
    exit 0
fi

echo "📥 Installing Ollama..."

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

echo "✅ Ollama installed successfully!"

# Start Ollama service in background
echo "🚀 Starting Ollama service..."
ollama serve &

# Wait a moment for service to start
sleep 3

# Pull the default model (tinyllama - fast and lightweight)
echo "📦 Downloading tinyllama model (default for trading bot)..."
ollama pull tinyllama

# Optional: Pull additional fast models for better analysis
echo ""
echo "🎯 Would you like to install additional fast models for better analysis? (y/n)"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo "📦 Installing additional models..."
    echo "⏳ This may take several minutes..."
    
    # Install phi for balanced speed/quality
    echo "📥 Installing phi (Microsoft's efficient model)..."
    ollama pull phi
    
    # Install gemma2:2b for good analysis
    echo "📥 Installing gemma2:2b (Google's optimized model)..."
    ollama pull gemma2:2b
    
    echo "✅ Additional models installed!"
fi

echo ""
echo "📋 Installed models:"
ollama list

echo ""
echo "✅ Ollama setup complete!"
echo "📋 Available models:"
echo "   • tinyllama    - Default, very fast (3-5s responses)"
echo "   • phi          - Balanced speed/quality (5-8s responses)"
echo "   • gemma2:2b    - Better analysis (8-15s responses)"
echo ""
echo "🔧 To change models, edit: trading-bot/config.env"
echo "💡 Set OLLAMA_MODEL=phi for better analysis quality"
echo ""
echo "🎯 Next step: Run ./test_installation.sh"