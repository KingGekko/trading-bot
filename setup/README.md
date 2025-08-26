# Trading Bot Setup Scripts

This folder contains automated setup scripts for deploying the Rust trading bot on Linux systems.

## 🚀 Quick Start (Recommended)

For a complete automated installation, run:

```bash
chmod +x *.sh
./full_setup.sh
```

This will install everything needed and test the installation.

## 📋 Individual Scripts

If you prefer step-by-step installation:

### 1. Install Dependencies
```bash
chmod +x install_dependencies.sh
./install_dependencies.sh
```
Installs: git, build tools, OpenSSL, pkg-config

### 2. Install Rust
```bash
chmod +x install_rust.sh
./install_rust.sh
source ~/.cargo/env
```
Installs: Rust programming language and Cargo

### 3. Clone and Build
```bash
chmod +x clone_and_build.sh
./clone_and_build.sh
```
Clones the repository and builds the optimized binary

### 4. Install Ollama
```bash
chmod +x install_ollama.sh
./install_ollama.sh
```
Installs Ollama AI and downloads models

### 5. Test Installation
```bash
chmod +x test_installation.sh
./test_installation.sh
```
Tests the complete installation

## 🎯 What Gets Installed

- **System Dependencies**: Build tools, Git, OpenSSL
- **Rust**: Latest stable Rust compiler and Cargo
- **Trading Bot**: Optimized binary with all features
- **Ollama AI**: AI engine with tinyllama model
- **Optional Models**: phi, gemma2:2b for better analysis

## 📊 Performance Expectations

- **Response Time**: 8-12 seconds (balanced mode)
- **Analysis Quality**: ⭐⭐⭐ Good structured analysis  
- **Binary Size**: ~5-10 MB
- **Memory Usage**: ~50-100 MB during operation

## 🔧 Supported Linux Distributions

- ✅ Ubuntu/Debian (apt)
- ✅ CentOS/RHEL/Fedora (yum/dnf)
- ✅ Alpine Linux (apk)
- ⚠️ Other distributions (manual dependency installation required)

## 🛠️ Manual Installation

If the automated scripts fail, you can install manually:

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/KingGekko/trading-bot.git
cd trading-bot
cargo build --release

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh
ollama serve &
ollama pull tinyllama
```

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

## 🚨 Troubleshooting

### Common Issues:

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

## 📞 Support

- **Repository**: https://github.com/KingGekko/trading-bot
- **Issues**: https://github.com/KingGekko/trading-bot/issues
- **Documentation**: See main README.md

## 🎉 Features

- ✅ Real-time streaming responses
- ✅ Balanced performance (8-12s responses)
- ✅ Comprehensive logging and receipts
- ✅ Security validation and input sanitization
- ✅ Multiple interaction modes (interactive, single prompt, test)
- ✅ Cross-platform support
- ✅ Performance optimizations (connection pooling, TCP keep-alive)

---

**Happy Trading! 🚀**