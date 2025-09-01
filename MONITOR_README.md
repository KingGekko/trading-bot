# Trading Account Monitor Scripts

These scripts help you monitor the `trading_account.json` file and manage the trading bot process.

## Files Created

1. **`monitor_trading_account.ps1`** - PowerShell script (recommended)
2. **`monitor_trading_account.bat`** - Simple batch file
3. **`trading_monitor.log`** - Log file created by the PowerShell script

## Usage

### PowerShell Script (Recommended)

```powershell
# Check current status
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -Status

# Start monitoring (default 10-second interval)
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1

# Start monitoring with custom interval (5 seconds)
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -Interval 5

# Start the trading bot only
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -StartBot

# Stop the trading bot
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -StopBot
```

### Batch File (Simple)

```cmd
# Just run the batch file
monitor_trading_account.bat
```

## Features

### PowerShell Script Features:
- ✅ **Smart Process Management**: Automatically detects if trading bot is already running
- ✅ **Auto-Start**: Starts the trading bot if it's not running
- ✅ **Real-time Monitoring**: Shows updates when the JSON file changes
- ✅ **Logging**: Creates `trading_monitor.log` with timestamps
- ✅ **Status Checking**: Shows current bot status and file info
- ✅ **Error Handling**: Graceful error handling and logging
- ✅ **Customizable Interval**: Set monitoring interval (default 10 seconds)

### What You'll See:
- **File Updates**: When `trading_account.json` changes
- **Timestamp**: Current update time
- **Market Status**: Whether market is open/closed
- **Portfolio Value**: Current account value
- **Cash Balance**: Available cash
- **Equity**: Total equity
- **Next Market Open**: When market opens next

### Batch File Features:
- ✅ **Simple Monitoring**: Basic file monitoring
- ✅ **No Dependencies**: Works on any Windows system
- ✅ **Continuous Loop**: Monitors every 10 seconds

## Example Output

```
[2025-09-01 13:23:38] === Trading Account Monitor Started ===
[2025-09-01 13:23:38] Trading bot is already running (PID: 34672)
[2025-09-01 13:23:38] Starting trading account monitoring (Interval: 10 seconds)
[2025-09-01 13:23:38] Press Ctrl+C to stop monitoring
[2025-09-01 13:23:45] === File Updated ===
[2025-09-01 13:23:45] Last Modified: 09/01/2025 13:23:45
[2025-09-01 13:23:45] Timestamp: 2025-09-01T12:23:45.896637100+00:00
[2025-09-01 13:23:45] Market Open: false
[2025-09-01 13:23:45] Portfolio Value: 100000
[2025-09-01 13:23:45] Cash: 100000
[2025-09-01 13:23:45] Equity: 100000
[2025-09-01 13:23:45] MARKET IS CLOSED
[2025-09-01 13:23:45] Next Open: 2025-09-01T13:30:00+00:00
```

## Troubleshooting

### If the trading bot is not running:
The PowerShell script will automatically try to start it for you.

### If you get permission errors:
Run PowerShell as Administrator or use the batch file instead.

### If the file is not updating:
1. Check if the trading bot is running: `powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -Status`
2. Make sure the `trading_portfolio` folder exists
3. Check the log file: `trading_monitor.log`

## Stopping the Monitor

- **PowerShell**: Press `Ctrl+C`
- **Batch**: Press `Ctrl+C`

## Stopping the Trading Bot

```powershell
powershell -ExecutionPolicy Bypass -File monitor_trading_account.ps1 -StopBot
```

Or manually:
```cmd
taskkill /f /im trading_bot.exe
```
