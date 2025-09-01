#!/bin/bash

# 🚀 Scripts Launcher
# This script provides easy access to scripts in the scripts/ folder

set -e

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Trading Bot Scripts Launcher${NC}"
echo "================================="
echo ""

# Function to show available scripts
show_scripts() {
    echo -e "${BLUE}📁 Available Scripts:${NC}"
    echo ""
    
    echo -e "${GREEN}🎮 Main Control:${NC}"
    echo "  ./scripts/trading_bot_control.sh [COMMAND]  - Unified bot control"
    echo ""
    
    echo -e "${GREEN}🧪 Testing:${NC}"
    echo "  ./scripts/run_all_tests.sh                 - Run all tests"
    echo ""
    
    echo -e "${GREEN}🚀 Operations:${NC}"
    echo "  ./scripts/start_live_mode.sh               - Start live mode"
    echo "  ./scripts/start_paper_mode.sh               - Start paper trading mode"
    echo "  ./scripts/stop_live_mode.sh                - Stop bot"
    echo ""
    
    echo -e "${GREEN}🔧 Utilities:${NC}"
echo "  ./scripts/setup_api_keys.sh                - Setup API keys"
echo "  ./scripts/monitor_streams.sh               - Monitor streams"
echo "  ./scripts/deploy_trading_bot.sh            - Deploy bot"
    echo "  ./scripts/switch_mode.sh                   - Switch paper/live modes"
    echo ""
    
    echo -e "${GREEN}🛠️  Maintenance:${NC}"
    echo "  ./scripts/update_dependencies.sh           - Update dependencies"
    echo "  ./scripts/fix_protobuf.sh                  - Fix protobuf issues"
    echo "  ./scripts/fix_npm_version.sh               - Fix NPM issues"
    echo ""
    
    echo -e "${YELLOW}💡 Quick Commands:${NC}"
    echo "  ./scripts.sh start                         - Start live mode"
    echo "  ./scripts.sh stop                          - Stop bot"
    echo "  ./scripts.sh status                        - Show status"
    echo "  ./scripts.sh test                          - Run all tests"
    echo "  ./scripts.sh mode [paper|live]             - Switch modes"
    echo "  ./scripts.sh stream [start|stop|status]   - Control simulated stream"
    echo "  ./scripts.sh watch                         - Interactive stream viewer"
    echo "  ./scripts.sh help                          - Show this help"
    echo ""
}

# Function to run main control script
run_control() {
    local command=$1
    echo -e "${BLUE}🎮 Running: trading_bot_control.sh $command${NC}"
    echo ""
    ./scripts/trading_bot_control.sh "$command"
}

# Function to run test suite
run_tests() {
    echo -e "${BLUE}🧪 Running: Comprehensive Test Suite${NC}"
    echo ""
    ./scripts/run_all_tests.sh
}

# Main script logic
case "${1:-help}" in
    start)
        run_control "start-live"
        ;;
    stop)
        run_control "stop"
        ;;
    status)
        run_control "status"
        ;;
    test)
        run_tests
        ;;
    mode)
        echo -e "${BLUE}🔄 Running: Mode Switcher${NC}"
        echo ""
        ./scripts/switch_mode.sh "${2:-status}"
        ;;
    stream)
        echo -e "${BLUE}📊 Running: Simulated Stream Controller${NC}"
        case "${2:-start}" in
            "start")
                ./scripts/start_simulated_stream.sh start
                ;;
            "stop")
                ./scripts/stop_simulated_stream.sh stop
                ;;
            "status")
                ./scripts/start_simulated_stream.sh status
                ;;
            *)
                echo -e "${YELLOW}Usage: ./scripts.sh stream [start|stop|status]${NC}"
                ;;
        esac
        ;;
    "watch")
        echo -e "${BLUE}📊 Running: Stream Viewer${NC}"
        ./scripts/watch_stream.sh
        ;;
    help|--help|-h)
        show_scripts
        ;;
    *)
        echo -e "${YELLOW}💡 Quick Commands:${NC}"
        echo "  ./scripts.sh start                         - Start live mode"
        echo "  ./scripts.sh stop                          - Stop bot"
        echo "  ./scripts.sh status                        - Show status"
        echo "  ./scripts.sh test                          - Run all tests"
        echo "  ./scripts.sh mode [test|live]             - Switch modes"
        echo "  ./scripts.sh stream [start|stop|status]   - Control simulated stream"
        echo "  ./scripts.sh watch                         - Interactive stream viewer"
        echo "  ./scripts.sh help                          - Show this help"
        echo ""
        echo -e "${BLUE}📁 For more options, see: ./scripts/README.md${NC}"
        echo ""
        show_scripts
        ;;
esac
