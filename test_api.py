#!/usr/bin/env python3
"""
Test script for the Trading Bot JSON Streaming API
Demonstrates how to use the REST endpoints and WebSocket streaming
"""

import requests
import json
import time
import websocket
import threading
from datetime import datetime

# API configuration
BASE_URL = "http://localhost:8080"
WS_BASE_URL = "ws://localhost:8080"

def test_health_check():
    """Test the health check endpoint"""
    print("🏥 Testing health check...")
    try:
        response = requests.get(f"{BASE_URL}/health")
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Health check passed: {data['status']}")
            print(f"   Service: {data['service']}")
            print(f"   Timestamp: {data['timestamp']}")
        else:
            print(f"❌ Health check failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Health check error: {e}")
    print()

def test_start_watching():
    """Test starting to watch a JSON file"""
    print("👀 Testing start watching...")
    try:
        payload = {"file_path": "./sample_data.json"}
        response = requests.post(f"{BASE_URL}/api/watch", json=payload)
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Started watching: {data['message']}")
        else:
            print(f"❌ Start watching failed: {response.status_code}")
            print(f"   Response: {response.text}")
    except Exception as e:
        print(f"❌ Start watching error: {e}")
    print()

def test_get_watched_files():
    """Test getting list of watched files"""
    print("📁 Testing get watched files...")
    try:
        response = requests.get(f"{BASE_URL}/api/files")
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Watched files: {data['watched_files']}")
        else:
            print(f"❌ Get watched files failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Get watched files error: {e}")
    print()

def test_get_file_content():
    """Test getting file content"""
    print("📄 Testing get file content...")
    try:
        response = requests.get(f"{BASE_URL}/api/content/sample_data.json")
        if response.status_code == 200:
            data = response.json()
            print(f"✅ File content retrieved for: {data['file_path']}")
            print(f"   Content preview: {str(data['content'])[:100]}...")
        else:
            print(f"❌ Get file content failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Get file content error: {e}")
    print()

def test_websocket_streaming():
    """Test WebSocket streaming for real-time updates"""
    print("🌊 Testing WebSocket streaming...")
    
    def on_message(ws, message):
        """Handle incoming WebSocket messages"""
        try:
            data = json.loads(message)
            msg_type = data.get('type', 'unknown')
            timestamp = data.get('timestamp', 'N/A')
            
            if msg_type == 'initial':
                print(f"📡 Received initial content at {timestamp}")
                print(f"   File: {data.get('file_path', 'N/A')}")
            elif msg_type == 'update':
                print(f"🔄 Received update at {timestamp}")
                print(f"   File: {data.get('file_path', 'N/A')}")
            elif msg_type == 'pong':
                print(f"🏓 Received pong at {timestamp}")
            
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse message: {e}")
    
    def on_error(ws, error):
        print(f"❌ WebSocket error: {error}")
    
    def on_close(ws, close_status_code, close_msg):
        print("🔌 WebSocket connection closed")
    
    def on_open(ws):
        print("🔗 WebSocket connection opened")
        # Send ping to test connection
        ws.send("ping")
    
    try:
        # Create WebSocket connection
        ws = websocket.WebSocketApp(
            f"{WS_BASE_URL}/api/stream/sample_data.json",
            on_open=on_open,
            on_message=on_message,
            on_error=on_error,
            on_close=on_close
        )
        
        # Start WebSocket in a separate thread
        ws_thread = threading.Thread(target=ws.run_forever)
        ws_thread.daemon = True
        ws_thread.start()
        
        # Wait a bit for connection and messages
        print("⏳ Waiting for WebSocket messages...")
        time.sleep(5)
        
        # Close connection
        ws.close()
        ws_thread.join(timeout=2)
        
    except Exception as e:
        print(f"❌ WebSocket test error: {e}")
    print()

def test_stop_watching():
    """Test stopping to watch a file"""
    print("🛑 Testing stop watching...")
    try:
        response = requests.get(f"{BASE_URL}/api/watch/sample_data.json")
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Stopped watching: {data['message']}")
        else:
            print(f"❌ Stop watching failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Stop watching error: {e}")
    print()

def update_sample_file():
    """Update the sample file to trigger streaming updates"""
    print("✏️ Updating sample file to trigger streaming...")
    try:
        # Read current content
        with open('sample_data.json', 'r') as f:
            data = json.load(f)
        
        # Update timestamp and price
        data['timestamp'] = datetime.utcnow().isoformat() + 'Z'
        data['price'] = round(data['price'] + (time.time() % 100), 2)
        
        # Write updated content
        with open('sample_data.json', 'w') as f:
            json.dump(data, f, indent=2)
        
        print(f"✅ Updated sample file - new price: {data['price']}")
        
    except Exception as e:
        print(f"❌ Failed to update sample file: {e}")

def main():
    """Run all API tests"""
    print("🧪 TRADING BOT API TEST SUITE")
    print("=" * 50)
    print()
    
    # Test basic endpoints
    test_health_check()
    test_start_watching()
    test_get_watched_files()
    test_get_file_content()
    
    # Test WebSocket streaming
    test_websocket_streaming()
    
    # Update file to trigger streaming
    update_sample_file()
    
    # Wait a bit for streaming to process
    time.sleep(2)
    
    # Test stopping
    test_stop_watching()
    
    print("🎉 API testing completed!")
    print()
    print("💡 To test real-time updates:")
    print("   1. Start the API server: ./trading_bot --api")
    print("   2. Run this test script: python3 test_api.py")
    print("   3. Modify sample_data.json to see live updates")

if __name__ == "__main__":
    main() 