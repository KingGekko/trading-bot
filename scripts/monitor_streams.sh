#!/bin/bash

# 📊 Real-Time Stream Monitor
# Monitors and logs the 4 JSON streams: News, Crypto, Options, Stocks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
LIVE_DATA_DIR="live_data"
LOG_DIR="stream_logs"
STREAMS=(
    "news:news_data_aapl.json,news_data_spy.json"
    "crypto:crypto_data_btc.json,crypto_data_eth.json"
    "options:options_data_spy.json"
    "stocks:stock_data_aapl.json"
)

echo -e "${BLUE}📊 Real-Time Stream Monitor${NC}"
echo "================================="
echo "Monitoring 4 JSON streams with live logging"
echo ""

# Create log directory
mkdir -p "$LOG_DIR"

# Function to get stream file paths
get_stream_files() {
    local stream_type=$1
    local files_string=$2
    
    # Split comma-separated files
    IFS=',' read -ra FILES <<< "$files_string"
    
    for file in "${FILES[@]}"; do
        local file_path="$LIVE_DATA_DIR/$file"
        if [ -f "$file_path" ]; then
            echo "$file_path"
        fi
    done
}

# Function to monitor a single stream
monitor_stream() {
    local stream_name=$1
    local files_string=$2
    local log_file="$LOG_DIR/${stream_name}_stream.log"
    
    echo -e "${CYAN}📡 Starting monitor for ${stream_name^^} stream...${NC}"
    
    # Get file paths for this stream
    local stream_files=($(get_stream_files "$stream_name" "$files_string"))
    
    if [ ${#stream_files[@]} -eq 0 ]; then
        echo -e "${RED}❌ No files found for ${stream_name} stream${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Monitoring ${#stream_files[@]} file(s) for ${stream_name} stream${NC}"
    
    # Start monitoring each file
    for file_path in "${stream_files[@]}"; do
        local filename=$(basename "$file_path")
        echo -e "${YELLOW}   📁 $filename${NC}"
        
        # Monitor file changes and log them
        (
            echo "=== $stream_name STREAM: $filename ===" >> "$log_file"
            echo "Started monitoring at: $(date)" >> "$log_file"
            echo "" >> "$log_file"
            
            # Initial content
            if [ -f "$file_path" ]; then
                echo "INITIAL CONTENT:" >> "$log_file"
                cat "$file_path" | jq . >> "$log_file" 2>/dev/null || cat "$file_path" >> "$log_file"
                echo "" >> "$log_file"
            fi
            
            # Monitor for changes
            while true; do
                if [ -f "$file_path" ]; then
                    # Get file modification time
                    local current_mtime=$(stat -c %Y "$file_path" 2>/dev/null || stat -f %m "$file_path" 2>/dev/null)
                    
                    # Check if file has changed
                    if [ "$current_mtime" != "$last_mtime" ]; then
                        echo "=== UPDATE: $(date) ===" >> "$log_file"
                        echo "File: $filename" >> "$log_file"
                        echo "Content:" >> "$log_file"
                        
                        # Try to format as JSON, fallback to raw content
                        if command -v jq >/dev/null 2>&1; then
                            cat "$file_path" | jq . >> "$log_file" 2>/dev/null || cat "$file_path" >> "$log_file"
                        else
                            cat "$file_path" >> "$log_file"
                        fi
                        
                        echo "" >> "$log_file"
                        echo "---" >> "$log_file"
                        echo "" >> "$log_file"
                        
                        # Update last modification time
                        last_mtime=$current_mtime
                        
                        # Show update in console
                        echo -e "${GREEN}📝 ${stream_name^^} stream updated: $filename${NC}"
                    fi
                fi
                
                # Wait before checking again
                sleep 1
            done
        ) &
        
        # Store PID for cleanup
        echo $! >> "$LOG_DIR/${stream_name}_pids.txt"
    done
    
    echo -e "${GREEN}✅ ${stream_name^^} stream monitor started${NC}"
    echo -e "${BLUE}   📋 Log file: $log_file${NC}"
    echo ""
}

# Function to show stream status
show_status() {
    echo -e "${BLUE}📊 Stream Status${NC}"
    echo "================"
    
    for stream_info in "${STREAMS[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        local log_file="$LOG_DIR/${stream_name}_stream.log"
        local pids_file="$LOG_DIR/${stream_name}_pids.txt"
        
        echo -e "${CYAN}${stream_name^^} Stream:${NC}"
        
        # Check if monitor is running
        if [ -f "$pids_file" ]; then
            local pids=$(cat "$pids_file" 2>/dev/null)
            if [ -n "$pids" ]; then
                echo -e "   ${GREEN}✅ Monitor: RUNNING${NC}"
                echo -e "   📋 Log: $log_file"
                
                # Show recent updates
                if [ -f "$log_file" ]; then
                    local update_count=$(grep -c "=== UPDATE:" "$log_file" 2>/dev/null || echo "0")
                    echo -e "   📝 Updates: $update_count"
                    
                    # Show last update time
                    local last_update=$(grep "=== UPDATE:" "$log_file" | tail -1 | cut -d' ' -f3- 2>/dev/null || echo "Never")
                    echo -e "   🕒 Last Update: $last_update"
                fi
            else
                echo -e "   ${RED}❌ Monitor: STOPPED${NC}"
            fi
        else
            echo -e "   ${YELLOW}⚠️ Monitor: NOT STARTED${NC}"
        fi
        
        # Show monitored files
        local stream_files=($(get_stream_files "$stream_name" "$files_string"))
        echo -e "   📁 Files: ${#stream_files[@]}"
        for file in "${stream_files[@]}"; do
            local filename=$(basename "$file")
            local size=$(ls -lh "$file" 2>/dev/null | awk '{print $5}' || echo "N/A")
            echo -e "      📄 $filename ($size)"
        done
        
        echo ""
    done
}

# Function to stop monitoring
stop_monitoring() {
    echo -e "${YELLOW}🛑 Stopping all stream monitors...${NC}"
    
    for stream_info in "${STREAMS[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        local pids_file="$LOG_DIR/${stream_name}_pids.txt"
        
        if [ -f "$pids_file" ]; then
            local pids=$(cat "$pids_file" 2>/dev/null)
            if [ -n "$pids" ]; then
                echo "$pids" | while read -r pid; do
                    if kill -0 "$pid" 2>/dev/null; then
                        kill "$pid" 2>/dev/null
                        echo -e "${GREEN}✅ Stopped monitor PID: $pid${NC}"
                    fi
                done
            fi
            rm -f "$pids_file"
        fi
    done
    
    echo -e "${GREEN}✅ All stream monitors stopped${NC}"
}

# Function to show live logs
show_live_logs() {
    echo -e "${BLUE}📋 Live Stream Logs${NC}"
    echo "====================="
    echo "Press Ctrl+C to stop viewing logs"
    echo ""
    
    # Show logs from all streams
    for stream_info in "${STREAMS[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        local log_file="$LOG_DIR/${stream_name}_stream.log"
        
        if [ -f "$log_file" ]; then
            echo -e "${CYAN}=== ${stream_name^^} STREAM LOG ===${NC}"
            tail -f "$log_file" &
            local tail_pid=$!
            echo $tail_pid >> "$LOG_DIR/tail_pids.txt"
        fi
    done
    
    # Wait for user to stop
    echo -e "${YELLOW}📋 Viewing live logs from all streams...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    
    # Cleanup on exit
    trap 'echo ""; echo "Stopping log viewing..."; cat "$LOG_DIR/tail_pids.txt" 2>/dev/null | xargs kill 2>/dev/null; rm -f "$LOG_DIR/tail_pids.txt"; exit 0' INT
    wait
}

# Function to show help
show_help() {
    echo -e "${BLUE}📊 Stream Monitor Help${NC}"
    echo "======================="
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo -e "  ${CYAN}interactive${NC} - Pick which streams to monitor (default)"
    echo -e "  ${CYAN}select${NC}      - Same as interactive"
    echo -e "  ${CYAN}start${NC}       - Start monitoring ALL streams"
    echo -e "  ${CYAN}stop${NC}        - Stop all stream monitors"
    echo -e "  ${CYAN}status${NC}      - Show current stream status"
    echo -e "  ${CYAN}logs${NC}        - Show live logs from all streams"
    echo -e "  ${CYAN}help${NC}        - Show this help message"
    echo ""
    echo "Interactive Mode Features:"
    echo -e "  🎯 ${CYAN}Pick streams${NC} - Choose which streams to monitor"
    echo -e "  📊 ${CYAN}Selective status${NC} - Check status of selected streams only"
    echo -e "  📋 ${CYAN}Selective logs${NC} - View logs of selected streams only"
    echo ""
    echo "Streams Available:"
    echo -e "  📰 ${CYAN}News${NC}: AAPL and SPY news data"
    echo -e "  🔐 ${CYAN}Crypto${NC}: BTC/USD and ETH/USD data"
    echo -e "  📊 ${CYAN}Options${NC}: SPY options data"
    echo -e "  📈 ${CYAN}Stocks${NC}: AAPL stock data"
    echo ""
    echo "Examples:"
    echo -e "  ${YELLOW}$0${NC}                    - Interactive mode (pick streams)"
    echo -e "  ${YELLOW}$0 select${NC}             - Interactive mode"
    echo -e "  ${YELLOW}$0 start${NC}              - Monitor all streams"
    echo -e "  ${YELLOW}$0 status${NC}             - Show all stream status"
    echo -e "  ${YELLOW}$0 logs${NC}               - View all stream logs"
    echo -e "  ${YELLOW}$0 stop${NC}               - Stop all monitors"
    echo ""
    echo "Log Files:"
    echo -e "  All logs are saved to: ${CYAN}$LOG_DIR/${NC}"
    echo -e "  Each stream has its own log file"
    echo -e "  Updates are logged with timestamps"
    echo ""
    echo "💡 Tip: Use interactive mode to pick only the streams you want to monitor!"
}

# Function to show stream selection menu
show_stream_menu() {
    echo -e "${BLUE}📊 Select Streams to Monitor${NC}"
    echo "================================"
    echo ""
    
    local selected_streams=()
    
    for i in "${!STREAMS[@]}"; do
        IFS=':' read -r stream_name files_string <<< "${STREAMS[$i]}"
        local stream_number=$((i + 1))
        
        # Check if stream is already being monitored
        local pids_file="$LOG_DIR/${stream_name}_pids.txt"
        local is_running=""
        if [ -f "$pids_file" ]; then
            local pids=$(cat "$pids_file" 2>/dev/null)
            if [ -n "$pids" ]; then
                is_running=" ${GREEN}[RUNNING]${NC}"
            fi
        fi
        
        echo -e "${CYAN}$stream_number.${NC} ${stream_name^^} Stream$is_running"
        
        # Show files for this stream
        local stream_files=($(get_stream_files "$stream_name" "$files_string"))
        for file in "${stream_files[@]}"; do
            local filename=$(basename "$file")
            local size=$(ls -lh "$file" 2>/dev/null | awk '{print $5}' || echo "N/A")
            echo -e "   📁 $filename ($size)"
        done
        echo ""
    done
    
    echo -e "${YELLOW}0.${NC} Monitor ALL streams"
    echo -e "${YELLOW}q.${NC} Quit"
    echo ""
}

# Function to get user stream selection
get_stream_selection() {
    local selected_streams=()
    
    while true; do
        show_stream_menu
        
        read -p "Select streams to monitor (comma-separated numbers, or 'all' for all): " selection
        
        if [ "$selection" = "q" ] || [ "$selection" = "quit" ]; then
            echo "Exiting..."
            exit 0
        fi
        
        if [ "$selection" = "all" ] || [ "$selection" = "0" ]; then
            # Select all streams
            for i in "${!STREAMS[@]}"; do
                IFS=':' read -r stream_name files_string <<< "${STREAMS[$i]}"
                selected_streams+=("$stream_name:$files_string")
            done
            break
        fi
        
        # Parse comma-separated selection
        IFS=',' read -ra SELECTED_NUMBERS <<< "$selection"
        
        for num in "${SELECTED_NUMBERS[@]}"; do
            num=$(echo "$num" | tr -d ' ')  # Remove spaces
            if [[ "$num" =~ ^[0-9]+$ ]] && [ "$num" -ge 1 ] && [ "$num" -le ${#STREAMS[@]} ]; then
                local index=$((num - 1))
                selected_streams+=("${STREAMS[$index]}")
            else
                echo -e "${RED}❌ Invalid selection: $num${NC}"
                continue
            fi
        done
        
        if [ ${#selected_streams[@]} -gt 0 ]; then
            break
        else
            echo -e "${RED}❌ No valid streams selected. Please try again.${NC}"
            echo ""
        fi
    done
    
    echo "${selected_streams[@]}"
}

# Function to monitor selected streams
monitor_selected_streams() {
    local selected_streams=($@)
    
    if [ ${#selected_streams[@]} -eq 0 ]; then
        echo -e "${RED}❌ No streams selected for monitoring${NC}"
        return 1
    fi
    
    echo -e "${GREEN}🚀 Starting monitoring for ${#selected_streams[@]} selected stream(s)...${NC}"
    echo ""
    
    # Start monitoring each selected stream
    for stream_info in "${selected_streams[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        monitor_stream "$stream_name" "$files_string"
    done
    
    echo -e "${GREEN}🎉 Selected stream monitors started!${NC}"
    echo ""
    echo -e "${BLUE}💡 Use these commands:${NC}"
    echo -e "   $0 status    - Check monitor status"
    echo -e "   $0 logs      - View live logs"
    echo -e "   $0 stop      - Stop monitoring"
    echo ""
}

# Function to show individual stream logs
show_stream_logs() {
    local selected_streams=($@)
    
    if [ ${#selected_streams[@]} -eq 0 ]; then
        echo -e "${RED}❌ No streams selected for log viewing${NC}"
        return 1
    fi
    
    echo -e "${BLUE}📋 Live Stream Logs${NC}"
    echo "====================="
    echo "Viewing logs for ${#selected_streams[@]} selected stream(s)"
    echo "Press Ctrl+C to stop viewing logs"
    echo ""
    
    # Show logs from selected streams
    for stream_info in "${selected_streams[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        local log_file="$LOG_DIR/${stream_name}_stream.log"
        
        if [ -f "$log_file" ]; then
            echo -e "${CYAN}=== ${stream_name^^} STREAM LOG ===${NC}"
            tail -f "$log_file" &
            local tail_pid=$!
            echo $tail_pid >> "$LOG_DIR/tail_pids.txt"
        fi
    done
    
    # Wait for user to stop
    echo -e "${YELLOW}📋 Viewing live logs from selected streams...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    
    # Cleanup on exit
    trap 'echo ""; echo "Stopping log viewing..."; cat "$LOG_DIR/tail_pids.txt" 2>/dev/null | xargs kill 2>/dev/null; rm -f "$LOG_DIR/tail_pids.txt"; exit 0' INT
    wait
}

# Function to show individual stream status
show_selected_status() {
    local selected_streams=($@)
    
    if [ ${#selected_streams[@]} -eq 0 ]; then
        echo -e "${RED}❌ No streams selected for status check${NC}"
        return 1
    fi
    
    echo -e "${BLUE}📊 Selected Stream Status${NC}"
    echo "========================="
    
    for stream_info in "${selected_streams[@]}"; do
        IFS=':' read -r stream_name files_string <<< "$stream_info"
        local log_file="$LOG_DIR/${stream_name}_stream.log"
        local pids_file="$LOG_DIR/${stream_name}_pids.txt"
        
        echo -e "${CYAN}${stream_name^^} Stream:${NC}"
        
        # Check if monitor is running
        if [ -f "$pids_file" ]; then
            local pids=$(cat "$pids_file" 2>/dev/null)
            if [ -n "$pids" ]; then
                echo -e "   ${GREEN}✅ Monitor: RUNNING${NC}"
                echo -e "   📋 Log: $log_file"
                
                # Show recent updates
                if [ -f "$log_file" ]; then
                    local update_count=$(grep -c "=== UPDATE:" "$log_file" 2>/dev/null || echo "0")
                    echo -e "   📝 Updates: $update_count"
                    
                    # Show last update time
                    local last_update=$(grep "=== UPDATE:" "$log_file" | tail -1 | cut -d' ' -f3- 2>/dev/null || echo "Never")
                    echo -e "   🕒 Last Update: $last_update"
                fi
            else
                echo -e "   ${RED}❌ Monitor: STOPPED${NC}"
            fi
        else
            echo -e "   ${YELLOW}⚠️ Monitor: NOT STARTED${NC}"
        fi
        
        # Show monitored files
        local stream_files=($(get_stream_files "$stream_name" "$files_string"))
        echo -e "   📁 Files: ${#stream_files[@]}"
        for file in "${stream_files[@]}"; do
            local filename=$(basename "$file")
            local size=$(ls -lh "$file" 2>/dev/null | awk '{print $5}' || echo "N/A")
            echo -e "      📄 $filename ($size)"
        done
        
        echo ""
    done
}

# Main execution
case "${1:-interactive}" in
    "start")
        echo -e "${GREEN}🚀 Starting Stream Monitoring...${NC}"
        echo ""
        
        # Start monitoring each stream
        for stream_info in "${STREAMS[@]}"; do
            IFS=':' read -r stream_name files_string <<< "$stream_info"
            monitor_stream "$stream_name" "$files_string"
        done
        
        echo -e "${GREEN}🎉 All stream monitors started!${NC}"
        echo ""
        echo -e "${BLUE}💡 Use these commands:${NC}"
        echo -e "   $0 status    - Check monitor status"
        echo -e "   $0 logs      - View live logs"
        echo -e "   $0 stop      - Stop monitoring"
        echo ""
        ;;
    "interactive"|"select")
        echo -e "${BLUE}🎯 Interactive Stream Selection${NC}"
        echo "================================"
        echo ""
        
        # Get user selection
        selected_streams=($(get_stream_selection))
        
        if [ ${#selected_streams[@]} -eq 0 ]; then
            echo -e "${YELLOW}No streams selected. Exiting...${NC}"
            exit 0
        fi
        
        # Start monitoring selected streams
        monitor_selected_streams "${selected_streams[@]}"
        ;;
    "stop")
        stop_monitoring
        ;;
    "status")
        if [ "$2" = "selected" ]; then
            # Show status for selected streams
            shift 2
            show_selected_status "$@"
        else
            show_status
        fi
        ;;
    "logs")
        if [ "$2" = "selected" ]; then
            # Show logs for selected streams
            shift 2
            show_stream_logs "$@"
        else
            show_live_logs
        fi
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo "Use '$0 help' for available commands"
        echo ""
        echo -e "${BLUE}💡 Try interactive mode:${NC}"
        echo -e "   $0 interactive  - Pick streams to monitor"
        echo -e "   $0 select       - Same as interactive"
        exit 1
        ;;
esac
