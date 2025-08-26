#!/bin/bash
# Build Linux binary using Docker

echo "🐧 Building Linux binary using Docker..."

# Build the Docker image and extract the binary
docker build -t trading-bot-builder .

# Create a temporary container to extract the binary
docker create --name temp-container trading-bot-builder

# Extract the binary to current directory
docker cp temp-container:/usr/local/bin/trading_bot ./trading_bot_linux

# Clean up
docker rm temp-container

echo "✅ Linux binary created: ./trading_bot_linux"
echo "💡 Transfer this file to your Linux system and run: ./trading_bot_linux --help"