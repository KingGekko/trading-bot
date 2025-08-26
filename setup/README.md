# Trading Bot Setup

This folder contains the complete installation script for the Rust trading bot.

## 🚨 Pre-Setup: No Git Required!

**If you're on a fresh machine without Git, use these methods:**

### **Option 1: Direct Script Download (Recommended)**
```bash
# Download and run in one command (no Git needed)
curl -fsSL https://raw.githubusercontent.com/KingGekko/trading-bot/main/setup/install.sh -o install.sh && chmod +x install.sh && ./install.sh
```

### **Option 2: Download Source Archive**
```bash
# Download source code directly
curl -L -o trading-bot.zip https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip

# Install unzip and extract
sudo apt update && sudo apt install -y unzip
unzip trading-bot.zip
mv trading-bot-main trading-bot
cd trading-bot/setup
chmod +x install.sh
./install.sh
```

### **Option 3: Install Git First**
```bash
# Install Git, then clone
sudo apt update && sudo apt install -y git
git clone https://github.com/KingGekko/trading-bot.git
cd trading-bot/setup
chmod +x install.sh
./install.sh
```

## 🚀 Quick Start

**Prerequisites: Git must be installed on your system**

For a complete automated installation, run:

```bash
chmod +x install.sh
./install.sh
```

This single script will install everything needed and test the installation.

## 📋 What Gets Installed

- **System Dependencies**: Build tools, OpenSSL, pkg-config
- **Rust**: Latest stable Rust compiler and Cargo
- **Trading Bot**: Optimized binary with all features
- **Ollama AI**: AI engine with tinyllama model
- **Optional Models**: llama2, phi, gemma2:2b for better analysis

## 📊 Performance Expectations

- **Response Time**: 8-12 seconds (tinyllama default)
- **Analysis Quality**: ⭐⭐⭐ Good structured analysis  
- **Binary Size**: ~5-10 MB
- **Memory Usage**: ~50-100 MB during operation
- **Default Model**: tinyllama (~1.1GB download)

## 🔧 Supported Linux Distributions

- ✅ Ubuntu/Debian (apt)
- ✅ CentOS/RHEL/Fedora (yum/dnf)
- ✅ Alpine Linux (apk)
- ⚠️ Other distributions (manual dependency installation required)

## 🎯 After Installation

Your trading bot will be available at:
- **Binary**: `trading-bot/target/release/trading_bot`
- **Config**: `trading-bot/config.env`
- **Logs**: `trading-bot/ollama_logs/`

### Usage Examples:
```bash
cd trading-bot

# Test mode
./target/release/trading_bot -t "Analyze Bitcoin trends"

# Interactive mode
./target/release/trading_bot -i

# View logs
./target/release/trading_bot -l
```

## 🤖 Ollama Model Management

### Available Models:
- **tinyllama** (1.1GB) - Default, fast responses, basic analysis
- **phi** (2.7GB) - Good analysis quality, balanced performance
- **llama2** (6GB) - Best analysis quality, slower responses
- **gemma2:2b** (1.5GB) - Excellent analysis, good performance

### Model Commands:
```bash
# List installed models
ollama list

# Pull a new model
ollama pull llama2
ollama pull phi
ollama pull gemma2:2b

# Remove a model
ollama rm llama2

# Switch model in config
# Edit config.env and change OLLAMA_MODEL=llama2
```

## 🚨 Troubleshooting

### Common Issues:

**Git not found:**
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y git

# CentOS/RHEL
sudo yum install -y git

# Fedora
sudo dnf install -y git

# Alpine
sudo apk add git
```

**Rust not found after installation:**
```bash
source ~/.cargo/env
```

**Ollama connection failed:**
```bash
ollama serve &
sleep 3
ollama list
```

**Build fails with OpenSSL errors:**
```bash
sudo apt install libssl-dev pkg-config  # Ubuntu/Debian
sudo yum install openssl-devel pkg-config  # CentOS/RHEL
```

**Permission denied:**
```bash
chmod +x target/release/trading_bot
```

**Ollama model download fails:**
```bash
# Check internet connection
curl -I https://ollama.ai

# Restart Ollama service
pkill ollama
ollama serve &

# Try downloading again
ollama pull tinyllama
```

## 📞 Support

- **Repository**: https://github.com/KingGekko/trading-bot
- **Issues**: https://github.com/KingGekko/trading-bot/issues
- **Documentation**: See main README.md

## 🎉 Features

- ✅ Real-time streaming responses
- ✅ Fast performance (8-12s responses with tinyllama)
- ✅ Comprehensive logging and receipts
- ✅ Security validation and input sanitization
- ✅ Multiple interaction modes (interactive, single prompt, test)
- ✅ Cross-platform support
- ✅ Performance optimizations (connection pooling, TCP keep-alive)
- ✅ Multiple AI model support (tinyllama, phi, llama2, gemma2:2b)

---

**Happy Trading! 🚀**