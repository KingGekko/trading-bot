# 🧪 Trading Bot Test Suite

Comprehensive testing framework for the trading bot system, covering all aspects from unit tests to deployment validation.

## 📁 **Test Structure**

```
tests/
├── unit/           # Rust unit tests and component testing
├── integration/    # API integration and system tests
├── performance/    # Performance benchmarking and stress tests
├── deployment/     # Deployment validation and environment tests
└── manual/         # Manual testing guides and scripts
```

## 🚀 **Quick Start**

### **Run All Tests**
```bash
# Run complete test suite
./run_tests.sh

# Run specific test categories
./run_tests.sh --unit-only
./run_tests.sh --integration-only
./run_tests.sh --performance-only
./run_tests.sh --deployment-only
```

### **Run Individual Test Categories**
```bash
# Unit tests (Rust)
cargo test

# Integration tests
./tests/integration/run_integration_tests.sh

# Performance tests
./tests/performance/run_performance_tests.sh

# Deployment tests
./tests/deployment/run_deployment_tests.sh
```

## 🔧 **Test Categories**

### **1. Unit Tests (`tests/unit/`)**
- **Rust component testing** - Individual module functionality
- **Mock data testing** - Isolated component behavior
- **Error handling** - Edge case validation
- **Data structure tests** - JSON parsing, validation

**Run with:**
```bash
cargo test
# or
./tests/unit/run_unit_tests.sh
```

### **2. Integration Tests (`tests/integration/`)**
- **API endpoint testing** - Complete API functionality
- **JSON stream testing** - File watching and streaming
- **WebSocket testing** - Real-time communication
- **Ollama integration** - AI processing workflows
- **Cross-component testing** - System integration

**Run with:**
```bash
./tests/integration/run_integration_tests.sh
# or individual tests
./tests/integration/test_api_endpoints.sh
./tests/integration/test_json_stream.sh
./tests/integration/test_websocket.sh
```

### **3. Performance Tests (`tests/performance/`)**
- **Load testing** - High-volume file updates
- **Memory profiling** - Resource usage analysis
- **Latency testing** - Response time measurement
- **Throughput testing** - Data processing capacity
- **Stress testing** - System limits validation

**Run with:**
```bash
./tests/performance/run_performance_tests.sh
# or specific tests
./tests/performance/test_load.sh
./tests/performance/test_memory.sh
./tests/performance/test_latency.sh
```

### **4. Deployment Tests (`tests/deployment/`)**
- **Environment validation** - System requirements check
- **Dependency testing** - Package availability
- **Service health** - Running service validation
- **Configuration testing** - Environment variable validation
- **Docker testing** - Container functionality

**Run with:**
```bash
./tests/deployment/run_deployment_tests.sh
# or specific tests
./tests/deployment/test_environment.sh
./tests/deployment/test_dependencies.sh
./tests/deployment/test_services.sh
```

### **5. Manual Tests (`tests/manual/`)**
- **Step-by-step guides** - Interactive testing
- **Troubleshooting** - Common issue resolution
- **User workflows** - End-to-end scenarios
- **Configuration examples** - Setup validation

**Use for:**
- Learning the system
- Debugging issues
- Validating configurations
- User acceptance testing

## 🎯 **Test Scenarios**

### **Core Functionality**
- ✅ **File watching** - JSON file monitoring
- ✅ **Real-time streaming** - WebSocket updates
- ✅ **API endpoints** - REST API functionality
- ✅ **Ollama integration** - AI processing
- ✅ **Error handling** - Robust error management

### **Performance Validation**
- ✅ **Response times** - < 200ms API responses
- ✅ **Update latency** - < 100ms file change detection
- ✅ **WebSocket latency** - < 50ms real-time updates
- ✅ **Memory usage** - Stable resource consumption
- ✅ **Concurrent users** - Multi-user support

### **Deployment Validation**
- ✅ **Environment setup** - System requirements
- ✅ **Service health** - Running services
- ✅ **Dependency availability** - Required packages
- ✅ **Configuration validation** - Environment variables
- ✅ **Network connectivity** - Service communication

## 🚨 **Prerequisites**

### **Required Services**
```bash
# Trading Bot API
cargo run -- --api

# Ollama (for AI testing)
ollama serve

# Node.js tools (for WebSocket testing)
npm install -g wscat@5.1.1
```

### **Required Tools**
```bash
# Basic tools
curl, wget, jq

# Testing tools
wscat, ab (Apache Bench), htop

# Development tools
cargo, rustc, git
```

## 📊 **Test Results**

### **Success Indicators**
- ✅ **All tests pass** - No failures
- ✅ **Performance targets met** - Response times within limits
- ✅ **Services healthy** - All endpoints responding
- ✅ **Resources stable** - Memory and CPU usage normal

### **Failure Investigation**
```bash
# Check test logs
tail -f test_results.log

# Run specific failing test
./tests/integration/test_api_endpoints.sh --verbose

# Debug mode
RUST_LOG=debug ./run_tests.sh
```

## 🔄 **Continuous Testing**

### **Automated Testing**
```bash
# Pre-commit hooks
./tests/run_pre_commit_tests.sh

# CI/CD integration
./tests/run_ci_tests.sh

# Deployment validation
./tests/run_deployment_tests.sh
```

### **Scheduled Testing**
```bash
# Daily health checks
crontab -e
0 9 * * * /path/to/trading-bot/tests/run_health_checks.sh

# Weekly performance tests
0 10 * * 0 /path/to/trading-bot/tests/run_performance_tests.sh
```

## 🎯 **Testing Best Practices**

### **Before Running Tests**
1. **Check prerequisites** - Services running, dependencies installed
2. **Clean environment** - Remove old test data
3. **Set configuration** - Environment variables, test data
4. **Verify isolation** - Tests don't interfere with each other

### **During Testing**
1. **Monitor resources** - CPU, memory, disk usage
2. **Check logs** - Error messages, warnings
3. **Validate results** - Expected vs actual outcomes
4. **Document issues** - Bug reports, performance problems

### **After Testing**
1. **Cleanup resources** - Remove test files, stop services
2. **Analyze results** - Performance metrics, error rates
3. **Generate reports** - Test summary, recommendations
4. **Update documentation** - Known issues, workarounds

## 🚀 **Next Steps**

1. **Run basic tests** - Start with unit and integration tests
2. **Validate deployment** - Ensure environment is ready
3. **Performance testing** - Measure system capabilities
4. **Manual validation** - Interactive testing and exploration
5. **Continuous integration** - Set up automated testing

---

**Happy Testing! 🧪** This comprehensive test suite ensures your trading bot is robust, performant, and ready for production.
