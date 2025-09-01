# 🚀 Elite Trading Bot - AI-Powered Automated Trading System

A sophisticated Rust-based trading bot with AI-enhanced decision making, real-time market data processing, and comprehensive portfolio management.

## ✨ Key Features

### 🤖 **AI-Enhanced Trading**
- **Elite Quantitative Trading Analyst** - Custom AI prompt for profit multiplication
- **Multi-Model AI Analysis** - Single or multi-model AI decision engines
- **Mathematical + AI Fusion** - Combines Modern Portfolio Theory with AI insights
- **Real-Time Decision Making** - Continuous AI-powered trading recommendations

### 📊 **Advanced Trading Strategy**
- **Modern Portfolio Theory (MPT)** - Optimal portfolio allocation
- **Kelly Criterion** - Optimal position sizing
- **Capital Asset Pricing Model (CAPM)** - Expected return calculation
- **Market Regime Analysis** - Bull/Bear/Sideways market detection
- **Risk Management** - Sharpe Ratio, VaR, Maximum Drawdown protection

### 🎯 **Interactive Setup Wizard**
- **Trading Mode Selection** - Paper Trading vs Live Trading
- **AI Model Selection** - Automatic Ollama model detection and selection
- **Automatic Server Startup** - All services start automatically
- **Continuous Trading Loop** - 30-second analysis cycles with real-time execution

### 📡 **Real-Time Data Processing**
- **Alpaca API Integration** - Live market data from Alpaca
- **WebSocket Streaming** - Real-time data for live trading mode
- **REST API Fallback** - Paper trading mode with REST API calls
- **Portfolio Monitoring** - Real-time account and position tracking

### 💾 **Data Management**
- **Protocol Buffer Storage** - Efficient binary data storage
- **JSON Export/Import** - Human-readable data formats
- **Portfolio Analysis** - Comprehensive portfolio insights
- **Historical Data** - Complete trading history and analysis

## 🚀 Quick Start

### **Prerequisites**
- Rust 1.70+
- Ollama (for AI processing)
- Alpaca API credentials (for live trading)

### **Installation**
```bash
# Clone the repository
git clone https://github.com/KingGekko/trading-bot.git
cd trading-bot

# Install dependencies (Windows)
./install.ps1

# Or manual installation
cargo build --release
```

### **Interactive Setup (Recommended)**
```bash
# Start the interactive setup wizard
./target/release/trading_bot.exe --interactive

# Follow the prompts:
# 1. Choose Paper Trading or Live Trading
# 2. Select Single Model or Multi-Model AI
# 3. Pick your preferred AI model
# 4. Automatic server startup and trading begins
```

### **Manual Commands**
```bash
# Paper Trading Mode
./target/release/trading_bot.exe --simulated

# Live Trading Mode (requires API keys)
./target/release/trading_bot.exe --websocket

# Enhanced Strategy Analysis
./target/release/trading_bot.exe --enhanced-strategy

# AI-Enhanced Decisions
./target/release/trading_bot.exe --ai-decisions

# Portfolio Analysis
./target/release/trading_bot.exe --portfolio-analysis

# Market Regime Analysis
./target/release/trading_bot.exe --market-regime
```

## 🏗️ Architecture

### **Core Components**
- **Interactive Setup** - Guided configuration wizard
- **Market Data Engine** - Real-time data collection and processing
- **Trading Strategy Engine** - Advanced mathematical trading algorithms
- **AI Decision Engine** - AI-enhanced trading recommendations
- **Order Execution System** - Automated trade execution
- **Portfolio Management** - Real-time portfolio monitoring and analysis

### **Data Flow**
```
Market Data → Strategy Analysis → AI Enhancement → Order Execution → Portfolio Update
     ↓              ↓                    ↓              ↓              ↓
Alpaca API → Mathematical Models → AI Insights → Alpaca Orders → Real-time Tracking
```

### **File Structure**
```
src/
├── interactive_setup.rs    # Interactive setup wizard
├── main.rs                 # Main entry point
├── market_data/           # Market data handling
│   ├── unified_websocket.rs
│   ├── trading_account.rs
│   ├── asset_universe.rs
│   └── market_regime.rs
├── trading_strategy/      # Trading algorithms
│   ├── enhanced_decision_engine.rs
│   └── ai_decision_engine.rs
├── order_execution/       # Order execution system
├── ollama/               # AI model integration
├── api/                  # JSON streaming API
└── protobuf/             # Data storage
```

## 🎯 Trading Modes

### **Paper Trading Mode**
- **Safe Testing** - Virtual money, no real financial risk
- **REST API Data** - Uses Alpaca REST API for market data
- **Full Strategy Testing** - Complete trading strategy validation
- **Performance Analysis** - Track virtual portfolio performance

### **Live Trading Mode**
- **Real Money Trading** - Actual financial transactions
- **WebSocket Streaming** - Real-time market data streaming
- **High Performance** - Ultra-low latency execution
- **Risk Management** - Built-in stop-loss and profit targets

## 🤖 AI Integration

### **Elite Trading Analyst Prompt**
```
"You are an Elite quantitative trading analyst. Analyze the following trading data to transcend in profit multiplication:"
```

### **AI Features**
- **Market Regime Assessment** - AI-powered market condition analysis
- **Decision Validation** - AI validation of mathematical trading decisions
- **Risk Assessment** - AI-enhanced risk analysis
- **Opportunity Identification** - AI discovery of missed trading opportunities
- **Portfolio Optimization** - AI recommendations for portfolio rebalancing

### **Supported AI Models**
- **TinyLlama** - Fast, lightweight analysis
- **Llama2** - Comprehensive market analysis
- **Gemma** - Advanced quantitative analysis
- **Phi** - Specialized financial analysis

## 📊 Portfolio Management

### **Real-Time Monitoring**
- **Portfolio Value** - Live portfolio valuation
- **Cash Balance** - Available trading capital
- **Position Tracking** - Current holdings and performance
- **Risk Metrics** - Real-time risk assessment

### **Data Storage**
- **Protocol Buffers** - Efficient binary storage
- **JSON Export** - Human-readable data export
- **Historical Analysis** - Complete trading history
- **Performance Tracking** - Detailed performance metrics

## 🔧 Configuration

### **Environment Setup**
```bash
# Copy configuration template
cp config.env.example config.env

# Configure Alpaca API keys
ALPACA_API_KEY=your_api_key
ALPACA_SECRET_KEY=your_secret_key
ALPACA_BASE_URL=https://paper-api.alpaca.markets  # Paper trading
# ALPACA_BASE_URL=https://api.alpaca.markets      # Live trading

# Configure Ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=tinyllama
```

### **Trading Configuration**
- **Risk-Free Rate** - Treasury yield for CAPM calculations
- **Profit Target** - 5% profit target per position
- **Stop Loss** - Portfolio protection at starting value
- **Position Sizing** - Kelly Criterion-based sizing

## 🧪 Testing & Validation

### **Strategy Testing**
```bash
# Test enhanced strategy
./target/release/trading_bot.exe --enhanced-strategy

# Test AI decisions
./target/release/trading_bot.exe --ai-decisions

# Test market regime analysis
./target/release/trading_bot.exe --market-regime
```

### **Portfolio Analysis**
```bash
# Generate portfolio analysis
./target/release/trading_bot.exe --portfolio-analysis

# View stored data
./target/release/trading_bot.exe --view-protobuf

# Export data
./target/release/trading_bot.exe --export-protobuf
```

## 📈 Performance Metrics

### **Trading Performance**
- **Sharpe Ratio** - Risk-adjusted returns
- **Maximum Drawdown** - Maximum portfolio decline
- **Value at Risk (VaR)** - Potential loss estimation
- **Expected Shortfall** - Tail risk assessment

### **System Performance**
- **Analysis Speed** - 30-second trading cycles
- **AI Response Time** - 8-12 seconds for comprehensive analysis
- **Data Processing** - Real-time market data processing
- **Order Execution** - Sub-second order placement

## 🚀 Deployment

### **Local Development**
```bash
# Build and run
cargo build --release
./target/release/trading_bot.exe --interactive
```

### **Production Deployment**
```bash
# Windows PowerShell
./install.ps1

# Linux/macOS
./setup/install.sh
```

### **Docker Deployment**
```bash
# Build Docker image
docker build -t elite-trading-bot .

# Run container
docker run -p 8080:8080 elite-trading-bot
```

## 🐛 Troubleshooting

### **Common Issues**
```bash
# Ollama not running
ollama serve

# API keys not configured
# Check config.env file

# Model not found
ollama pull tinyllama

# Permission issues (Linux/macOS)
sudo chown -R $USER:$USER .
```

### **Debug Mode**
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/trading_bot.exe --interactive
```

## 📚 Documentation

- **DEV_DIARY.md** - Complete development history and features
- **API_README.md** - API documentation and endpoints
- **DEPLOYMENT_README.md** - Deployment guides
- **MONITOR_README.md** - Portfolio monitoring setup

## ⚠️ Risk Disclaimer

**This software is for educational and research purposes only. Trading involves substantial risk of loss and is not suitable for all investors. Past performance is not indicative of future results. Always consult with a qualified financial advisor before making investment decisions.**

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

**Built with Rust, AI, and Advanced Mathematics** 🦀🤖📊

*Elite Trading Bot - Transcending in Profit Multiplication*