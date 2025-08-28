# 🚀 JSON Stream Testing Guide

This guide shows you how to test the trading bot's **real-time JSON streaming functionality** manually.

## 📋 **Prerequisites**

1. **Trading Bot API Running**
   ```bash
   cargo run -- --api
   ```
   The API will start on `http://localhost:3000`

2. **Ollama Running** (for AI processing tests)
   ```bash
   ollama serve
   ```

3. **wscat Installed** (for WebSocket testing)
   ```bash
   npm install -g wscat@5.1.1
   ```

## 🔌 **API Endpoints Overview**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | API health check |
| `/api/watch` | POST | Start watching a JSON file |
| `/api/watch/{file_path}` | GET | Stop watching a file |
| `/api/files` | GET | List all watched files |
| `/api/content/{file_path}` | GET | Get current file content |
| `/api/stream/{file_path}` | GET | **WebSocket stream** for real-time updates |
| `/api/ollama/process` | POST | Process JSON with Ollama AI |
| `/api/available-files` | GET | List available files |

## 🧪 **Step-by-Step Testing**

### **Step 1: Start the API Server**
```bash
# In one terminal
cargo run -- --api
```

### **Step 2: Test Basic API Endpoints**
```bash
# Health check
curl http://localhost:3000/health

# List available files
curl http://localhost:3000/api/available-files

# List watched files (should be empty initially)
curl http://localhost:3000/api/files
```

### **Step 3: Create Test JSON File**
```bash
# Create test directory
mkdir -p test_files

# Create test JSON file
cat > test_files/trading_data.json << 'EOF'
{
    "id": 1,
    "name": "BTC Trading Data",
    "timestamp": "2024-01-15T10:00:00Z",
    "data": {
        "price": 100.50,
        "volume": 1000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.0"
    }
}
EOF
```

### **Step 4: Start File Watching**
```bash
# Start watching the test file
curl -X POST http://localhost:3000/api/watch \
  -H "Content-Type: application/json" \
  -d '{"file_path": "test_files/trading_data.json"}'

# Verify it's being watched
curl http://localhost:3000/api/files

# Get current content
curl http://localhost:3000/api/content/test_files/trading_data.json
```

### **Step 5: Test WebSocket Streaming**
```bash
# In a new terminal, connect to WebSocket stream
wscat -c ws://localhost:3000/api/stream/test_files/trading_data.json
```

**Keep this WebSocket connection open!** You'll see real-time updates.

### **Step 6: Trigger File Updates**
```bash
# In another terminal, modify the file to trigger updates
cat > test_files/trading_data.json << 'EOF'
{
    "id": 1,
    "name": "BTC Trading Data - UPDATED",
    "timestamp": "2024-01-15T10:05:00Z",
    "data": {
        "price": 101.25,
        "volume": 1500,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.1",
        "updated": true
    }
}
EOF
```

**Watch the WebSocket terminal** - you should see the real-time update!

### **Step 7: Test Ollama Processing**
```bash
# Process the JSON file with Ollama AI
curl -X POST http://localhost:3000/api/ollama/process \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "test_files/trading_data.json",
    "prompt": "Analyze this trading data and provide insights about the price movement and volume changes.",
    "model": "llama2"
  }'
```

### **Step 8: Test Multiple Updates**
```bash
# Update 1: Price change
cat > test_files/trading_data.json << 'EOF'
{
    "id": 1,
    "name": "BTC Trading Data - PRICE UPDATE",
    "timestamp": "2024-01-15T10:10:00Z",
    "data": {
        "price": 102.75,
        "volume": 2000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.2",
        "update_type": "price_change"
    }
}
EOF

# Wait 2 seconds, then update 2: Volume change
sleep 2
cat > test_files/trading_data.json << 'EOF'
{
    "id": 1,
    "name": "BTC Trading Data - VOLUME UPDATE",
    "timestamp": "2024-01-15T10:15:00Z",
    "data": {
        "price": 102.75,
        "volume": 3000,
        "symbol": "BTCUSD"
    },
    "metadata": {
        "source": "test",
        "version": "1.3",
        "update_type": "volume_change"
    }
}
EOF
```

### **Step 9: Cleanup**
```bash
# Stop watching the file
curl http://localhost:3000/api/watch/test_files/trading_data.json

# Remove test files
rm -rf test_files
```

## 🌊 **WebSocket Message Format**

The WebSocket sends JSON messages in this format:

### **Initial Content Message**
```json
{
    "type": "initial",
    "file_path": "test_files/trading_data.json",
    "content": { /* file content */ }
}
```

### **Update Message**
```json
{
    "type": "update",
    "file_path": "test_files/trading_data.json",
    "content": { /* updated content */ },
    "timestamp": "2024-01-15T10:05:00Z"
}
```

### **Pong Message** (response to "ping")
```json
{
    "type": "pong",
    "timestamp": "2024-01-15T10:05:00Z"
}
```

## 🚀 **Advanced Testing Scenarios**

### **1. Multiple File Watching**
```bash
# Watch multiple files simultaneously
curl -X POST http://localhost:3000/api/watch \
  -H "Content-Type: application/json" \
  -d '{"file_path": "test_files/file1.json"}'

curl -X POST http://localhost:3000/api/watch \
  -H "Content-Type: application/json" \
  -d '{"file_path": "test_files/file2.json"}'

# List all watched files
curl http://localhost:3000/api/files
```

### **2. Large JSON Files**
```bash
# Create a large JSON file for performance testing
python3 -c "
import json
data = {
    'id': 1,
    'items': [{'id': i, 'value': f'item_{i}'} for i in range(10000)],
    'metadata': {'size': 'large', 'timestamp': '2024-01-15T10:00:00Z'}
}
with open('test_files/large_data.json', 'w') as f:
    json.dump(data, f, indent=2)
"

# Watch the large file
curl -X POST http://localhost:3000/api/watch \
  -H "Content-Type: application/json" \
  -d '{"file_path": "test_files/large_data.json"}'
```

### **3. Rapid File Updates**
```bash
# Script to rapidly update a file
for i in {1..10}; do
    cat > test_files/trading_data.json << EOF
{
    "id": 1,
    "update_number": $i,
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "data": {
        "price": $((100 + i)),
        "volume": $((1000 + i * 100))
    }
}
EOF
    sleep 0.5
done
```

## 🔍 **Troubleshooting**

### **Common Issues**

1. **API not responding**
   ```bash
   # Check if API is running
   curl http://localhost:3000/health
   
   # Check logs
   cargo run -- --api
   ```

2. **WebSocket connection fails**
   ```bash
   # Check if wscat is installed
   wscat --version
   
   # Install wscat if needed
   npm install -g wscat@5.1.1
   ```

3. **File not being watched**
   ```bash
   # Check if file exists
   ls -la test_files/
   
   # Check watched files
   curl http://localhost:3000/api/files
   
   # Restart watching
   curl -X POST http://localhost:3000/api/watch \
     -H "Content-Type: application/json" \
     -d '{"file_path": "test_files/trading_data.json"}'
   ```

4. **Ollama processing fails**
   ```bash
   # Check if Ollama is running
   curl http://localhost:11434/api/tags
   
   # Start Ollama if needed
   ollama serve
   ```

### **Debug Mode**
```bash
# Run API with debug logging
RUST_LOG=debug cargo run -- --api
```

## 📊 **Expected Results**

### **Successful Test Run Should Show:**

1. ✅ **API Health**: `{"status":"healthy","service":"trading-bot-api"}`
2. ✅ **File Watching**: `{"status":"success","message":"Started watching file: ..."}`
3. ✅ **WebSocket Connection**: Real-time updates when files change
4. ✅ **File Updates**: Immediate notifications in WebSocket
5. ✅ **Ollama Processing**: AI analysis of JSON data
6. ✅ **Cleanup**: No errors when stopping file watching

### **Performance Indicators:**

- **File Update Detection**: < 100ms
- **WebSocket Latency**: < 50ms
- **API Response Time**: < 200ms
- **Memory Usage**: Stable during file watching

## 🎯 **Next Steps**

After successful testing:

1. **Integrate with real trading data** sources
2. **Set up automated file monitoring** for production
3. **Configure WebSocket clients** for real-time dashboards
4. **Implement error handling** and retry logic
5. **Add authentication** for production use

---

**Happy Testing! 🚀** The JSON stream system provides real-time monitoring capabilities perfect for trading applications.
