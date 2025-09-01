@echo off
REM Trading Account Monitor - Simple Batch Version
REM This script monitors the trading_account.json file

set TRADING_ACCOUNT_FILE=trading_portfolio\trading_account.json
set LOG_FILE=trading_monitor.log

echo === Trading Account Monitor Started ===
echo Press Ctrl+C to stop monitoring
echo.

:monitor_loop
echo === %date% %time% ===
if exist "%TRADING_ACCOUNT_FILE%" (
    echo File exists - checking for updates...
    for /f "tokens=*" %%i in ('type "%TRADING_ACCOUNT_FILE%" ^| findstr "timestamp"') do echo %%i
    for /f "tokens=*" %%i in ('type "%TRADING_ACCOUNT_FILE%" ^| findstr "is_open"') do echo %%i
    for /f "tokens=*" %%i in ('type "%TRADING_ACCOUNT_FILE%" ^| findstr "portfolio_value"') do echo %%i
    for /f "tokens=*" %%i in ('type "%TRADING_ACCOUNT_FILE%" ^| findstr "cash"') do echo %%i
) else (
    echo WARNING: Trading account file not found: %TRADING_ACCOUNT_FILE%
)

echo.
timeout /t 10 /nobreak >nul
goto monitor_loop
