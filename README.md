# Trading Bot

A Rust-based trading bot foundation with Ollama integration for AI-powered decision making.

## Features

- **Ollama Integration**: Communicate with Ollama API for AI responses
- **Real-time Streaming by Default**: All responses stream in real-time for enhanced responsiveness
- **Interactive Mode**: Chat with the AI using streaming responses (with optional regular mode)
- **Single Prompt Mode**: Send one-off prompts with streaming output
- **Ollama Receipt System**: Detailed transaction logging with separate success/failure logs
- **Performance Testing**: Comprehensive timing analysis with streaming and detailed metrics
- **Configuration Management**: Environment-based configuration with security validation
- **Error Handling**: Robust error handling and structured logging
- **File-based Logging**: JSON-formatted receipts stored in separate log files

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Ollama**: Install and run Ollama locally
   - Download from [ollama.ai](https://ollama.ai/)
   - Pull a model: `ollama pull llama2` (or your preferred model)
   - Start the service: `ollama serve`

## Setup

1. Clone and navigate to the project directory
2. Copy the configuration file:
   ```bash
   cp config.env .env
   ```
3. Edit `.env` to match your setup:
   ```
   OLLAMA_BASE_URL=http://localhost:11434
   OLLAMA_MODEL=llama2
   BOT_NAME=TradingBot
   LOG_LEVEL=info
   ```

## Usage

### Build the project
```bash
cargo build --release
```

### Run in interactive mode (streaming by default)
```bash
cargo run -- -i
```

### Send a single prompt (streaming by default)
```bash
cargo run -- -p "What are the current market trends?"
```

### Run performance test with streaming and detailed timing
```bash
cargo run -- -t "Analyze the cryptocurrency market trends"
```

### Send a prompt with streaming (same as -p, provided for compatibility)
```bash
cargo run -- -s "Explain blockchain technology in detail"
```

### View prettified receipt logs
```bash
cargo run -- -l
```

### Show help
```bash
cargo run -- --help
```

## Configuration

The bot uses environment variables for configuration. You can set these in:
- `config.env` file (recommended)
- `.env` file (fallback)
- System environment variables

### Available Configuration Options

#### Required Variables (No Defaults)
- `OLLAMA_BASE_URL`: URL where Ollama is running (e.g., http://localhost:11434)
- `OLLAMA_MODEL`: Model to use for generation (e.g., llama2, codellama, etc.)

#### Optional Variables (With Secure Defaults)
- `BOT_NAME`: Name of the bot (default: TradingBot)
- `LOG_LEVEL`: Logging level - error, warn, info, debug, trace (default: info)
- `MAX_TIMEOUT_SECONDS`: Request timeout in seconds (default: 300, max: 3600)
- `LOG_DIRECTORY`: Directory for log files (default: ollama_logs)
- `MAX_PROMPT_LENGTH`: Maximum prompt length in characters (default: 8192, max: 1,000,000)
- `MAX_RESPONSE_LENGTH`: Maximum response length in characters (default: 32768, max: 10,000,000)

## Ollama Receipt System & Performance Analysis

The bot includes a comprehensive receipt system that logs all Ollama transactions:

### Test Mode
Use the `-t` flag to run detailed performance tests:
```bash
cargo run -- -t "Explain blockchain technology"
```

This will provide:
- **Response Time**: Precise millisecond timing
- **Generation Speed**: Characters per second analysis
- **Performance Classification**: Fast/Moderate/Slow categorization
- **Detailed Metrics**: Start/end times, prompt/response lengths
- **Error Analysis**: Comprehensive error reporting if requests fail
- **Receipt Logging**: Automatic saving to JSON log files

### Automatic Receipt Generation
All modes (interactive, single prompt, test) automatically generate receipts:
- **Success receipts**: Saved to `ollama_logs/success_receipts.jsonl`
- **Failure receipts**: Saved to `ollama_logs/failure_receipts.jsonl`
- **Pretty-printed JSON Standard**: All receipts saved in human-readable format by default
- **Smart parsing**: Robust parser handles various JSON formatting
- **No response content**: Receipts contain metadata only, not actual AI responses

### Streaming by Default
All modes now use streaming by default for the best user experience:

**All commands stream responses in real-time:**
- `-p "prompt"` - Single prompt with streaming
- `-i` - Interactive mode with streaming
- `-t "prompt"` - Performance testing with streaming
- `-s "prompt"` - Explicit streaming (same as -p)

**Benefits of default streaming:**
- 🌊 **Real-time output** as the AI generates responses
- ⚡ **Enhanced responsiveness** - see responses immediately
- 📊 **Full receipt tracking** with streaming-specific timing analysis
- 🔄 **Chunked processing** for efficient memory usage
- 🚀 **Performance optimizations** - reduced tokens, faster sampling, connection pooling

### Interactive Mode Options
In interactive mode, you now have two options:
```
>>> Tell me about cryptocurrency trends
🌊 Streaming response...
Bot: [Real-time response appears here as it's generated]

>>> /regular Tell me about cryptocurrency trends  
Sending to Ollama (regular mode)...
Bot: [Complete response appears at once]
```

### Performance Optimization

**Significant Speed Improvements:**
- **Optimized Model Parameters**: Reduced max tokens (500), lower temperature (0.1), focused sampling
- **Connection Pooling**: HTTP connections reused for better throughput  
- **TCP Keep-Alive**: Persistent connections reduce latency
- **Streaming Buffer Optimization**: Pre-allocated buffers for faster processing

**Fastest Model Recommendations** (3-5 second responses):
- `phi` - Microsoft's efficient small model
- `qwen2.5:0.5b` - Alibaba's ultra-fast 0.5B parameter model  
- `gemma2:2b` - Google's optimized 2B parameter model
- `tinyllama` - Current default, good balance of speed/quality

**Example with fast model:**
```bash
# Download a fast model
ollama pull phi

# Update config.env
OLLAMA_MODEL=phi

# Test performance  
cargo run -- -t "Explain blockchain quickly"
```

### Viewing Logs
Use the `--logs` or `-l` flag to view a prettified summary:
```bash
cargo run -- -l
```

This displays:
- 📊 **Summary view** of all receipts with status icons
- ⏱️ **Quick timing info** (duration, characters processed)
- 🔍 **Error details** for failed transactions
- 📈 **Transaction counts** for success/failure analysis

### Receipt Structure
Each receipt contains:
```json
{
  "start_time": "2024-01-15T14:30:25.123Z",
  "end_time": "2024-01-15T14:30:28.456Z", 
  "duration_ms": 3333,
  "request_type": "Generate",
  "model": "llama2",
  "prompt_length": 25,
  "response_length": 156,
  "success": true,
  "error_message": null
}
```

## Project Structure

```
src/
├── main.rs             # Entry point and CLI handling
└── ollama/             # Ollama-related modules
    ├── mod.rs              # Module exports and re-exports
    ├── ollama_client.rs    # Ollama API client implementation
    ├── ollama_config.rs    # Ollama configuration management
    └── ollama_receipt.rs   # Receipt generation and file logging
ollama_logs/
├── success_receipts.jsonl  # Successful Ollama transactions
└── failure_receipts.jsonl  # Failed Ollama transactions
```

## Security & Configuration

### Security Features

The trading bot includes several security measures:

- **Required Environment Variables**: Critical configuration (OLLAMA_BASE_URL, OLLAMA_MODEL) must be explicitly set
- **Input Validation**: All user inputs are sanitized to prevent injection attacks
- **URL Validation**: Ollama URLs are validated for proper format and protocol (http/https only)
- **Resource Limits**: Configurable timeouts and content length limits prevent resource exhaustion
- **Model Name Validation**: Only alphanumeric characters and safe symbols allowed in model names
- **Log Level Validation**: Only valid log levels accepted

### Security Best Practices

1. **Always use trusted Ollama endpoints** - Only connect to Ollama instances you control
2. **Set appropriate limits** - Configure MAX_TIMEOUT_SECONDS and length limits based on your needs
3. **Use HTTPS in production** - When possible, use secure HTTPS connections to Ollama
4. **Monitor logs** - Regularly review receipt logs for unusual activity
5. **Keep dependencies updated** - Run `cargo audit` to check for security vulnerabilities

### Environment File Security

The `config.env` file contains configuration but should not contain secrets. For production:
- Use system environment variables for sensitive configuration
- Ensure `config.env` has appropriate file permissions
- Consider using a secrets management system

## Development

### Testing Ollama Connection

You can test if Ollama is running and accessible:

```bash
curl http://localhost:11434/api/tags
```

### Testing WebSocket Functionality

After deployment, you can test the WebSocket streaming functionality:

```bash
# Make the test script executable
chmod +x test_websocket.sh

# Run comprehensive WebSocket test
./test_websocket.sh

# Check prerequisites only
./test_websocket.sh --check-only

# Test only WebSocket connection
./test_websocket.sh --websocket-only

# Test only file watching
./test_websocket.sh --watch-only
```

The WebSocket test script will:
- ✅ Check if the trading bot API is running
- ✅ Verify wscat is available (installs if needed)
- ✅ Test file watching functionality
- ✅ Establish WebSocket connection
- ✅ Monitor real-time file updates

### Adding New Features

This is a foundation project. You can extend it by:

1. Adding trading-specific modules
2. Implementing market data fetchers
3. Adding strategy execution logic
4. Integrating with trading APIs
5. Adding persistence and logging

## Dependencies

- **tokio**: Async runtime
- **reqwest**: HTTP client for Ollama API
- **serde**: JSON serialization/deserialization
- **clap**: Command-line argument parsing
- **dotenv**: Environment variable loading
- **anyhow**: Error handling
- **log/env_logger**: Structured logging for timing analysis
- **chrono**: Date/time handling for precise timing measurements

## License

MIT License