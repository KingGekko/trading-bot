# Trading Account Monitor Script
# This script monitors the trading_account.json file and can start the trading bot if needed

param(
    [switch]$StartBot,
    [switch]$StopBot,
    [switch]$Status,
    [int]$Interval = 10
)

$TradingBotPath = ".\target\release\trading_bot.exe"
$TradingAccountFile = "trading_portfolio\trading_account.json"
$LogFile = "trading_monitor.log"

function Write-Log {
    param([string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "[$timestamp] $Message"
    Write-Host $logMessage
    Add-Content -Path $LogFile -Value $logMessage
}

function Get-TradingBotStatus {
    $processes = Get-Process -Name "trading_bot" -ErrorAction SilentlyContinue
    if ($processes) {
        return @{
            IsRunning = $true
            ProcessId = $processes[0].Id
            ProcessName = $processes[0].ProcessName
        }
    }
    return @{
        IsRunning = $false
        ProcessId = $null
        ProcessName = $null
    }
}

function Start-TradingBot {
    $status = Get-TradingBotStatus
    if ($status.IsRunning) {
        Write-Log "Trading bot is already running (PID: $($status.ProcessId))"
        return $true
    }
    
    if (-not (Test-Path $TradingBotPath)) {
        Write-Log "ERROR: Trading bot executable not found at $TradingBotPath"
        return $false
    }
    
    try {
        Write-Log "Starting trading bot..."
        Start-Process -FilePath $TradingBotPath -ArgumentList "--trading" -WindowStyle Hidden
        Start-Sleep -Seconds 3
        
        $newStatus = Get-TradingBotStatus
        if ($newStatus.IsRunning) {
            Write-Log "Trading bot started successfully (PID: $($newStatus.ProcessId))"
            return $true
        } else {
            Write-Log "ERROR: Failed to start trading bot"
            return $false
        }
    }
    catch {
        Write-Log "ERROR: Exception starting trading bot: $($_.Exception.Message)"
        return $false
    }
}

function Stop-TradingBot {
    $status = Get-TradingBotStatus
    if (-not $status.IsRunning) {
        Write-Log "Trading bot is not running"
        return $true
    }
    
    try {
        Write-Log "Stopping trading bot (PID: $($status.ProcessId))..."
        Stop-Process -Id $status.ProcessId -Force
        Start-Sleep -Seconds 2
        
        $newStatus = Get-TradingBotStatus
        if (-not $newStatus.IsRunning) {
            Write-Log "Trading bot stopped successfully"
            return $true
        } else {
            Write-Log "ERROR: Failed to stop trading bot"
            return $false
        }
    }
    catch {
        Write-Log "ERROR: Exception stopping trading bot: $($_.Exception.Message)"
        return $false
    }
}

function Monitor-TradingAccount {
    Write-Log "Starting trading account monitoring (Interval: $Interval seconds)"
    Write-Log "Press Ctrl+C to stop monitoring"
    
    $lastModified = $null
    $lastContent = $null
    
    while ($true) {
        try {
            if (Test-Path $TradingAccountFile) {
                $currentModified = (Get-Item $TradingAccountFile).LastWriteTime
                $currentContent = Get-Content $TradingAccountFile -Raw
                
                if ($lastModified -ne $currentModified -or $lastContent -ne $currentContent) {
                    Write-Log "=== File Updated ==="
                    Write-Log "Last Modified: $currentModified"
                    
                    # Extract key information
                    $jsonContent = $currentContent | ConvertFrom-Json
                    
                    Write-Log "Timestamp: $($jsonContent.timestamp)"
                    Write-Log "Market Open: $($jsonContent.market_status.is_open)"
                    Write-Log "Portfolio Value: $($jsonContent.account_info.portfolio_value)"
                    Write-Log "Cash: $($jsonContent.account_info.cash)"
                    Write-Log "Equity: $($jsonContent.account_info.equity)"
                    
                    if ($jsonContent.market_status.is_open) {
                        Write-Log "MARKET IS OPEN"
                    } else {
                        Write-Log "MARKET IS CLOSED"
                        Write-Log "Next Open: $($jsonContent.market_status.next_open)"
                    }
                    
                    $lastModified = $currentModified
                    $lastContent = $currentContent
                }
            } else {
                Write-Log "WARNING: Trading account file not found: $TradingAccountFile"
            }
        }
        catch {
            Write-Log "ERROR: Exception monitoring file: $($_.Exception.Message)"
        }
        
        Start-Sleep -Seconds $Interval
    }
}

function Show-Status {
    $status = Get-TradingBotStatus
    Write-Log "=== Trading Bot Status ==="
    if ($status.IsRunning) {
        Write-Log "Status: RUNNING (PID: $($status.ProcessId))"
    } else {
        Write-Log "Status: NOT RUNNING"
    }
    
    if (Test-Path $TradingAccountFile) {
        $fileInfo = Get-Item $TradingAccountFile
        Write-Log "Account File: EXISTS (Last Modified: $($fileInfo.LastWriteTime))"
        
        try {
            $jsonContent = Get-Content $TradingAccountFile -Raw | ConvertFrom-Json
            Write-Log "Current Portfolio Value: $($jsonContent.account_info.portfolio_value)"
            Write-Log "Market Status: $(if ($jsonContent.market_status.is_open) { 'OPEN' } else { 'CLOSED' })"
        }
        catch {
            Write-Log "ERROR: Could not parse account file"
        }
    } else {
        Write-Log "Account File: NOT FOUND"
    }
}

# Main execution
Write-Log "=== Trading Account Monitor Started ==="

if ($StartBot) {
    Start-TradingBot
    exit
}

if ($StopBot) {
    Stop-TradingBot
    exit
}

if ($Status) {
    Show-Status
    exit
}

# Default: Start monitoring
$botStatus = Get-TradingBotStatus
if (-not $botStatus.IsRunning) {
    Write-Log "Trading bot is not running. Starting it..."
    if (Start-TradingBot) {
        Write-Log "Waiting for trading bot to initialize..."
        Start-Sleep -Seconds 5
    } else {
        Write-Log "ERROR: Could not start trading bot. Monitoring will continue but may not see updates."
    }
} else {
    Write-Log "Trading bot is already running (PID: $($botStatus.ProcessId))"
}

Monitor-TradingAccount