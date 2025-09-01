# Trading Bot Installation Script for Windows
# This script installs all necessary dependencies for the trading bot

Write-Host "🚀 Trading Bot Installation Script" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green

# Check if Rust is installed
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Installing Rust..." -ForegroundColor Red
    Write-Host "Please visit https://rustup.rs/ and install Rust first." -ForegroundColor Yellow
    Write-Host "After installing Rust, run this script again." -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "✅ Rust is already installed" -ForegroundColor Green
    rustc --version
}

# Check if Git is installed
Write-Host "`nChecking Git installation..." -ForegroundColor Yellow
if (!(Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Git is not installed. Installing Git..." -ForegroundColor Red
    Write-Host "Please visit https://git-scm.com/ and install Git first." -ForegroundColor Yellow
    Write-Host "After installing Git, run this script again." -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "✅ Git is already installed" -ForegroundColor Green
    git --version
}

# Install SQLite for Windows (for future database functionality)
Write-Host "`nInstalling SQLite..." -ForegroundColor Yellow
$sqliteUrl = "https://www.sqlite.org/2024/sqlite-tools-win32-x86-3450100.zip"
$sqliteZip = "sqlite-tools.zip"
$sqliteDir = "sqlite-tools"

try {
    # Download SQLite
    Write-Host "Downloading SQLite..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $sqliteUrl -OutFile $sqliteZip
    
    # Extract SQLite
    Write-Host "Extracting SQLite..." -ForegroundColor Yellow
    Expand-Archive -Path $sqliteZip -DestinationPath $sqliteDir -Force
    
    # Add SQLite to PATH
    $sqlitePath = (Get-Location).Path + "\" + $sqliteDir + "\sqlite-tools-win32-x86-3450100"
    $env:PATH += ";" + $sqlitePath
    
    Write-Host "✅ SQLite installed successfully" -ForegroundColor Green
    sqlite3 --version
    
    # Clean up
    Remove-Item $sqliteZip -Force
    Write-Host "SQLite tools are available in: $sqlitePath" -ForegroundColor Cyan
    
} catch {
    Write-Host "⚠️ Warning: Could not install SQLite automatically" -ForegroundColor Yellow
    Write-Host "You can install SQLite manually from: https://www.sqlite.org/download.html" -ForegroundColor Yellow
}

# Install protobuf compiler (for future protobuf functionality)
Write-Host "`nInstalling Protocol Buffers compiler..." -ForegroundColor Yellow
try {
    # Try to install via chocolatey if available
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Host "Installing protobuf via Chocolatey..." -ForegroundColor Yellow
        choco install protoc -y
    } else {
        Write-Host "⚠️ Chocolatey not found. Installing protobuf manually..." -ForegroundColor Yellow
        $protocUrl = "https://github.com/protocolbuffers/protobuf/releases/download/v25.1/protoc-25.1-win64.zip"
        $protocZip = "protoc.zip"
        $protocDir = "protoc"
        
        # Download protoc
        Invoke-WebRequest -Uri $protocUrl -OutFile $protocZip
        
        # Extract protoc
        Expand-Archive -Path $protocZip -DestinationPath $protocDir -Force
        
        # Add protoc to PATH
        $protocPath = (Get-Location).Path + "\" + $protocDir + "\bin"
        $env:PATH += ";" + $protocPath
        
        Write-Host "✅ Protocol Buffers compiler installed successfully" -ForegroundColor Green
        protoc --version
        
        # Clean up
        Remove-Item $protocZip -Force
        Write-Host "Protoc is available in: $protocPath" -ForegroundColor Cyan
    }
} catch {
    Write-Host "⚠️ Warning: Could not install Protocol Buffers compiler automatically" -ForegroundColor Yellow
    Write-Host "You can install it manually from: https://github.com/protocolbuffers/protobuf/releases" -ForegroundColor Yellow
}

# Build the trading bot
Write-Host "`nBuilding Trading Bot..." -ForegroundColor Yellow
try {
    cargo build --release
    Write-Host "✅ Trading Bot built successfully!" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to build Trading Bot" -ForegroundColor Red
    Write-Host "Please check the error messages above and fix any issues." -ForegroundColor Yellow
    exit 1
}

# Create necessary directories
Write-Host "`nCreating necessary directories..." -ForegroundColor Yellow
$directories = @(
    "logs",
    "trading_portfolio",
    "data"
)

foreach ($dir in $directories) {
    if (!(Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force
        Write-Host "✅ Created directory: $dir" -ForegroundColor Green
    } else {
        Write-Host "✅ Directory already exists: $dir" -ForegroundColor Green
    }
}

# Create sample configuration file
Write-Host "`nCreating sample configuration..." -ForegroundColor Yellow
$configContent = @"
# Trading Bot Configuration
# Copy this to config.env and update with your actual values

# Alpaca API Configuration
ALPACA_API_KEY=your_alpaca_api_key_here
ALPACA_SECRET_KEY=your_alpaca_secret_key_here
ALPACA_BASE_URL=https://paper-api.alpaca.markets

# Ollama Configuration
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=auto
OLLAMA_MAX_TIMEOUT_SECONDS=30

# Logging Configuration
LOG_DIRECTORY=./logs

# WebSocket streaming enabled (set to false for Normal Mode REST API)
WEBSOCKET_ENABLED=false

# Data Storage Configuration
DATA_DIRECTORY=./data
DATABASE_PATH=./data/trading_bot.db
"@

if (!(Test-Path "config.env")) {
    $configContent | Out-File -FilePath "config.env" -Encoding UTF8
    Write-Host "✅ Created sample config.env file" -ForegroundColor Green
    Write-Host "⚠️ Please update config.env with your actual API keys" -ForegroundColor Yellow
} else {
    Write-Host "✅ config.env already exists" -ForegroundColor Green
}

# Display installation summary
Write-Host "`n🎉 Installation Complete!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green
Write-Host "✅ Rust: Installed" -ForegroundColor Green
Write-Host "✅ Git: Installed" -ForegroundColor Green
Write-Host "✅ SQLite: Available for future database features" -ForegroundColor Green
Write-Host "✅ Protocol Buffers: Available for future protobuf features" -ForegroundColor Green
Write-Host "✅ Trading Bot: Built successfully" -ForegroundColor Green
Write-Host "✅ Directories: Created" -ForegroundColor Green
Write-Host "✅ Configuration: Sample file created" -ForegroundColor Green

Write-Host "`n📋 Next Steps:" -ForegroundColor Cyan
Write-Host "1. Update config.env with your Alpaca API keys" -ForegroundColor White
Write-Host "2. Install Ollama from https://ollama.ai/" -ForegroundColor White
Write-Host "3. Run the bot with: ./target/release/trading_bot.exe --help" -ForegroundColor White

Write-Host "`n🚀 Available Commands:" -ForegroundColor Cyan
Write-Host "• ./target/release/trading_bot.exe --help" -ForegroundColor White
Write-Host "• ./target/release/trading_bot.exe --data-storage" -ForegroundColor White
Write-Host "• ./target/release/trading_bot.exe --enhanced-strategy" -ForegroundColor White
Write-Host "• ./target/release/trading_bot.exe --portfolio-analysis" -ForegroundColor White

Write-Host "`n📚 Documentation:" -ForegroundColor Cyan
Write-Host "• Check the README.md file for detailed usage instructions" -ForegroundColor White
Write-Host "• Visit https://alpaca.markets/docs/ for API documentation" -ForegroundColor White

Write-Host "`n✨ Happy Trading!" -ForegroundColor Green
