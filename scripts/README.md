# Scripts Directory

This directory contains all the scripts for managing and testing the trading bot system.

## 🎮 Main Control Scripts

### `trading_bot_control.sh` - Unified Trading Bot Control
**Purpose**: Main control script for starting, stopping, and monitoring the trading bot
**Usage**: `./trading_bot_control.sh [COMMAND]`

**Commands**:
- `start-live` - Start trading bot in live mode
- `start-test` - Start trading bot in test mode  
- `stop` - Stop the trading bot
- `status` - Show current status
- `restart` - Restart the trading bot
- `logs` - Show recent logs
- `help` - Show help message

**Example**: `./trading_bot_control.sh start-live`

## 🚀 Operational Scripts

### `start_live_mode.sh` - Live Mode Startup
**Purpose**: Starts the trading bot in live mode with real market data streaming
**Dependencies**: `config.env`, Rust toolchain, Ollama

### `start_test_mode.sh` - Test Mode Startup  
**Purpose**: Starts the trading bot in test mode for development and testing
**Dependencies**: `config.env`, Rust toolchain

### `stop_live_mode.sh` - Stop Trading Bot
**Purpose**: Gracefully stops the trading bot and cleans up processes
**Dependencies**: PID file from running bot

### `deploy_trading_bot.sh` - Deployment Script
**Purpose**: Deploys the trading bot to production environments
**Dependencies**: Docker, cloud credentials

## 🧪 Testing Scripts

### `run_all_tests.sh` - Comprehensive Test Suite
**Purpose**: Runs all tests for the trading bot system
**Usage**: `./run_all_tests.sh`
**Dependencies**: All individual test scripts

### `test_account_verification.sh` - Account Verification Test
**Purpose**: Tests Alpaca account connectivity and permissions
**Dependencies**: `config.env` with valid API keys

### `test_unified_websocket.sh` - WebSocket Streaming Test
**Purpose**: Tests the unified WebSocket-based streaming for all data types
**Dependencies**: Running trading bot, market data streams

### `test_live_mode.sh` - Live Mode Test
**Purpose**: Tests live mode functionality with real market data
**Dependencies**: Live mode running, market data files

### `test_enhanced_json_stream.sh` - JSON Stream Test
**Purpose**: Tests the enhanced JSON streaming capabilities
**Dependencies**: JSON streaming service

### `test_ollama_json_reading.sh` - Ollama Integration Test
**Purpose**: Tests Ollama integration and JSON processing
**Dependencies**: Ollama service running

### `test_real_streaming.sh` - Real-time Streaming Test
**Purpose**: Tests real-time market data streaming
**Dependencies**: Market data streams active

## 🔧 Utility Scripts

### `monitor_streams.sh` - Stream Monitoring
**Purpose**: Monitors and manages market data streams
**Usage**: `./monitor_streams.sh [OPTIONS]`

### `run_tests.sh` - Test Runner
**Purpose**: Legacy test runner (use `run_all_tests.sh` instead)
**Status**: Deprecated

### `setup_api_keys.sh` - API Key Setup
**Purpose**: Interactive setup for Alpaca API credentials
**Usage**: `./setup_api_keys.sh`

### `find_rust.sh` - Rust Installation Finder
**Purpose**: Locates Rust toolchain installations
**Usage**: `./find_rust.sh`

### `debug_file_watching.sh` - File Watch Debug
**Purpose**: Debug file watching and monitoring issues
**Usage**: `./debug_file_watching.sh`

## 🛠️ Maintenance Scripts

### `fix_protobuf.sh` - Protobuf Fix
**Purpose**: Fixes protobuf compilation issues
**Usage**: `./fix_protobuf.sh`

### `fix_npm_version.sh` - NPM Version Fix
**Purpose**: Fixes NPM version compatibility issues
**Usage**: `./fix_npm_version.sh`

### `update_dependencies.sh` - Dependency Updates
**Purpose**: Updates system and Rust dependencies
**Usage**: `./update_dependencies.sh`

## 📁 Directory Structure

```
scripts/
├── README.md                           # This file
├── trading_bot_control.sh             # Main control script
├── start_live_mode.sh                 # Live mode startup
├── start_test_mode.sh                 # Test mode startup
├── stop_live_mode.sh                  # Stop bot
├── deploy_trading_bot.sh              # Deployment
├── run_all_tests.sh                   # Comprehensive test suite
├── test_*.sh                          # Individual test scripts
├── monitor_streams.sh                 # Stream monitoring
├── setup_api_keys.sh                  # API key setup
├── find_rust.sh                       # Rust finder
├── debug_file_watching.sh             # Debug utilities
├── fix_*.sh                           # Fix scripts
└── update_dependencies.sh             # Dependency updates
```

## 🚀 Quick Start

1. **Setup API Keys**: `./setup_api_keys.sh`
2. **Start Live Mode**: `./trading_bot_control.sh start-live`
3. **Check Status**: `./trading_bot_control.sh status`
4. **Run Tests**: `./run_all_tests.sh`
5. **Stop Bot**: `./trading_bot_control.sh stop`

## 📋 Script Dependencies

### Required Files
- `../config.env` - Development configuration
- `../config.env` - Unified configuration (test/live modes)
- `../live_data/` - Market data directory

### Required Services
- Rust toolchain (cargo)
- Ollama service (for AI features)
- Alpaca API access

## 🔍 Troubleshooting

### Common Issues
1. **Script not found**: Ensure scripts are executable (`chmod +x *.sh`)
2. **Permission denied**: Check file permissions and ownership
3. **Config not found**: Verify config files exist in parent directory
4. **Rust not found**: Run `./find_rust.sh` to locate installation

### Debug Mode
Most scripts support verbose output. Add `set -x` at the top of any script for debugging.

## 📝 Script Development

When adding new scripts:
1. Follow the existing naming convention
2. Include proper error handling
3. Add usage documentation
4. Update this README
5. Make scripts executable (`chmod +x`)

## 🔒 Security Notes

- API keys are stored in environment files
- Scripts should not contain hardcoded credentials
- Use proper file permissions for sensitive scripts
- Validate all inputs and file paths
