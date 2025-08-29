# 🚀 VM Setup Guide - Trading Bot

## Quick Start in VM

### 1. **Setup API Keys (Super Easy!)**
```bash
# Make the script executable
chmod +x setup_api_keys.sh

# Run the interactive setup
./setup_api_keys.sh
```

### 2. **What the Script Does**
- ✅ **Interactive Menu**: Choose what you want to do
- ✅ **Add Multiple Keys**: Easily add Alpaca + custom API keys
- ✅ **Validation**: Checks if everything is configured correctly
- ✅ **Security**: Masks API keys when displaying

### 3. **Menu Options**
```
1. Add/Update API Keys     - Quick setup for Alpaca
2. Add Multiple API Keys   - Add Alpaca + custom keys
3. Show Configuration      - View current settings (keys masked)
4. Validate Configuration  - Check if ready to run
5. Exit                   - Quit the setup
```

### 4. **Example Usage**
```bash
# In your VM, just run:
./setup_api_keys.sh

# Choose option 2 for multiple keys
# Enter your Alpaca API key when prompted
# Enter your Alpaca secret key when prompted
# Add any other API keys you need
# Validate when done
```

### 5. **Configuration File**
The script automatically updates `config.env` with your keys:
```env
ALPACA_API_KEY=your_actual_key_here
ALPACA_SECRET_KEY=your_actual_secret_here
# ... other settings
```

### 6. **Ready to Run!**
Once configured, you can:
```bash
# Start the trading bot
cargo run -- --api

# Or run in live mode
./start_live_mode.sh
```

## 🔑 **API Key Sources**

- **Alpaca**: Get free paper trading keys at [alpaca.markets](https://alpaca.markets)
- **Paper Trading**: Use paper trading for testing (free, no real money)
- **Live Trading**: Switch to live API when ready

## 🚨 **Security Notes**

- ✅ API keys are stored in `config.env` (don't commit this file!)
- ✅ Keys are masked when displaying configuration
- ✅ Use paper trading keys for testing
- ✅ Never share your live trading API keys
