#!/usr/bin/env python3
"""
Trading Bot API Test Script
Tests all endpoints including the new Ollama JSON processing
"""

import requests
import json
import time
import sys

# Configuration
BASE_URL = "http://localhost:8080"
TIMEOUT = 30

def test_endpoint(method, endpoint, data=None, description=""):
    """Test an API endpoint and return the response"""
    url = f"{BASE_URL}{endpoint}"
    
    print(f"\n🔍 Testing: {description}")
    print(f"   {method} {endpoint}")
    
    try:
        if method == "GET":
            response = requests.get(url, timeout=TIMEOUT)
        elif method == "POST":
            headers = {"Content-Type": "application/json"}
            response = requests.post(url, json=data, headers=headers, timeout=TIMEOUT)
        else:
            print(f"❌ Unknown method: {method}")
            return None
            
        print(f"   Status: {response.status_code}")
        
        if response.status_code == 200:
            try:
                result = response.json()
                print(f"   ✅ Success: {json.dumps(result, indent=2)}")
                return result
            except json.JSONDecodeError:
                print(f"   ⚠️ Response is not JSON: {response.text}")
                return response.text
        else:
            print(f"   ❌ Error: {response.status_code}")
            try:
                error = response.json()
                print(f"   Error details: {json.dumps(error, indent=2)}")
            except:
                print(f"   Error text: {response.text}")
            return None
            
    except requests.exceptions.RequestException as e:
        print(f"   ❌ Request failed: {e}")
        return None

def main():
    """Run all API tests"""
    print("🧪 Trading Bot API Test Suite")
    print("=" * 50)
    
    # Test 1: Health Check
    test_endpoint("GET", "/health", description="Health Check")
    
    # Test 2: Start watching sample_data.json
    watch_data = {"file_path": "./sample_data.json"}
    test_endpoint("POST", "/api/watch", watch_data, "Start Watching File")
    
    # Test 3: List watched files
    test_endpoint("GET", "/api/files", description="List Watched Files")
    
    # Test 4: Get file content
    test_endpoint("GET", "/api/content/sample_data.json", description="Get File Content")
    
    # Test 5: NEW! Process JSON with Ollama AI
    ollama_data = {
        "file_path": "./sample_data.json",
        "prompt": "Analyze this trading data and provide insights about market sentiment, price trends, and trading opportunities. Focus on the technical indicators and recent price action.",
        "model": "phi"  # Optional: specify a model, or let it use default
    }
    test_endpoint("POST", "/api/ollama/process", ollama_data, "Ollama AI JSON Analysis")
    
    # Test 6: Test with different prompt
    analysis_data = {
        "file_path": "./sample_data.json",
        "prompt": "What are the key risk factors in this trading data? Provide a risk assessment score from 1-10.",
    }
    test_endpoint("POST", "/api/ollama/process", analysis_data, "Ollama AI Risk Analysis")
    
    print("\n🎉 API Test Suite Completed!")
    print("\n💡 Try these additional tests:")
    print("   • Change the prompt in the Ollama requests")
    print("   • Use different models (e.g., 'qwen2.5:0.5b', 'gemma2:2b')")
    print("   • Watch different JSON files")
    print("   • Test WebSocket streaming with: wscat -c ws://localhost:8080/api/stream/sample_data.json")

if __name__ == "__main__":
    try:
    main() 
    except KeyboardInterrupt:
        print("\n\n⏹️ Test interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        sys.exit(1) 