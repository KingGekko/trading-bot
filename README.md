# 🚀 Trading Bot - AI-Powered JSON Streaming System

A high-performance Rust-based trading bot with Ollama AI integration, real-time JSON streaming, and comprehensive testing infrastructure.

## ✨ Features

### 🤖 **AI Integration**
- **Ollama AI Processing** - Multiple AI models (Gemma, Llama, Phi, TinyLlama)
- **Ultra-Fast Threading** - Optimized performance with parallel processing
- **Multi-Model Conversations** - AI models can interact with each other
- **Real-Time Analysis** - Live trading data analysis and insights

### 📡 **JSON Streaming System**
- **File Watching** - Real-time monitoring of JSON data files
- **WebSocket Streaming** - Live data streaming to clients
- **Change Detection** - Automatic detection of file modifications
- **Event-Driven Updates** - Instant notifications of data changes

### ⚡ **Performance Optimizations**
- **Ultra-Fast Threading** - Maximum speed with parallel operations
- **Multi-Threaded Processing** - Non-blocking, concurrent operations
- **Streaming Optimization** - Real-time data processing
- **Memory Management** - Efficient resource utilization

### 🧪 **Comprehensive Testing**
- **Unit Tests** - Rust component testing
- **Integration Tests** - API and system testing
- **Performance Tests** - Load and benchmarking
- **Deployment Tests** - Environment validation
- **Manual Testing** - Step-by-step guides

## 🚀 Quick Start

### **Prerequisites**
- Rust 1.70+
- Ollama (for AI processing)
- Node.js + npm (for WebSocket testing)

### **Installation**
```bash
# Clone the repository
git clone https://github.com/KingGekko/trading-bot.git
cd trading-bot

# Install dependencies
./setup/install.sh

# Start the server
cargo run -- --api
```

### **Quick Test**
```bash
# Test the JSON streaming system
./test_real_streaming.sh

# Run all tests
./run_tests.sh
```

## 🏗️ Architecture

### **Core Components**
- **API Server** - Axum-based REST API with WebSocket support
- **JSON Stream Manager** - Real-time file monitoring and streaming
- **Ollama Client** - AI model integration and processing
- **Performance Engine** - Multi-threaded optimization system

### **API Endpoints**
```
GET  /health                    - Health check
POST /api/watch                 - Start watching a JSON file
GET  /api/watch/:file_path     - Stop watching a file
GET  /api/files                - List watched files
GET  /api/content/:file_path   - Get file content
GET  /api/stream/:file_path    - WebSocket stream for real-time updates
POST /api/ollama/process       - Process JSON with AI (ULTRA-FAST THREADING)
POST /api/ollama/conversation  - Multi-model AI conversations
GET  /api/available-files      - List available JSON files
```

## 🧪 Testing

### **Test Categories**
```
tests/
├── unit/           # Rust unit tests
├── integration/    # API integration tests
├── performance/    # Load and benchmarking
├── deployment/     # Environment validation
└── manual/         # Manual testing guides
```

### **Run Tests**
```bash
# All tests
./run_tests.sh

# Specific categories
./tests/unit/run_unit_tests.sh
./tests/integration/run_integration_tests.sh
./tests/performance/run_performance_tests.sh
./tests/deployment/run_deployment_tests.sh
```

## 🚀 Deployment

### **Local Development**
```bash
cargo run -- --api
```

### **Production Deployment**
```bash
# Automated deployment
./deploy_trading_bot.sh

# Docker deployment
docker build -t trading-bot .
docker run -p 8080:8080 trading-bot
```

### **Cloud Deployment**
```bash
# Cloud-init template
cloud-init-template.yml

# Automated setup
./setup/install.sh
```

## 🔧 Configuration

### **Environment Variables**
```bash
# Copy and customize
cp config.env.example config.env

# Key settings
OLLAMA_BASE_URL=http://localhost:11434
API_PORT=8080
LOG_LEVEL=info
```

### **AI Model Configuration**
```bash
# Available models
ollama list

# Pull specific models
ollama pull gemma3:27b
ollama pull llama2:latest
ollama pull tinyllama:latest
```

## 📊 Performance

### **Optimization Modes**
- **Ultra-Fast** - Maximum speed, direct async processing
- **Threaded** - Non-blocking, parallel operations
- **Ultra-Threaded** - Maximum threading, parallel operations
- **Default** - Balanced performance and quality

### **Benchmarks**
- **File Processing** - <100ms for typical JSON files
- **AI Response** - 8-12s for comprehensive analysis
- **Streaming Latency** - <50ms for real-time updates
- **Concurrent Users** - 100+ simultaneous connections

## 🐛 Troubleshooting

### **Common Issues**
```bash
# Ollama not running
ollama serve

# Port conflicts
cargo run -- --api --port 8081

# Model not found
ollama pull <model_name>

# Permission issues
sudo chown -R $USER:$USER /opt/trading-bot
```

### **Debug Mode**
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- --api

# Verbose testing
./test_real_streaming.sh --verbose
```

## 📚 Documentation

- **DEV_DIARY.md** - Complete development history and commit log
- **API_README.md** - Detailed API documentation
- **DEPLOYMENT_README.md** - Deployment guides and troubleshooting
- **ULTRA_FAST_DEFAULT.md** - Performance optimization details
- **THREADING_OPTIMIZATION.md** - Multi-threading implementation
- **MULTI_MODEL_CONVERSATIONS.md** - AI conversation system

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

## 🆘 Support

- **Issues** - GitHub Issues
- **Discussions** - GitHub Discussions
- **Documentation** - Check DEV_DIARY.md for recent changes

---

**Built with Rust, Axum, Tokio, and Ollama** 🦀⚡🤖