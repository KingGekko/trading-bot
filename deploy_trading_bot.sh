#!/bin/bash

# Trading Bot Deployment Script
# This script downloads, installs, and sets up the trading bot for production use

set -e

echo "🚀 Trading Bot Deployment Script"
echo "================================"
echo "This script will:"
echo "  • Download the latest trading bot source code"
echo "  • Run the complete installation process"
echo "  • Create a cloud-init script for future deployments"
echo "  • Clean up temporary files"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip"
GIT_REPO="https://github.com/KingGekko/trading-bot.git"
INSTALL_DIR="/opt/trading-bot"
BACKUP_DIR="/opt/trading-bot-backup"
CLOUD_INIT_DIR="/opt/cloud-init-scripts"
LOG_FILE="/var/log/trading-bot-deployment.log"

# Function to log messages
log_message() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $message" | tee -a "$LOG_FILE"
}

# Function to check if running with sudo
check_sudo() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}❌ This script requires admin privileges!${NC}"
        echo ""
        echo "Please run with sudo:"
        echo "  sudo ./deploy_trading_bot.sh"
        exit 1
    fi
    
    # Check if we're running as root (which is correct for sudo)
    if [ "$EUID" -eq 0 ]; then
        echo -e "${GREEN}✅ Running with admin privileges${NC}"
    fi
}

# Function to check and install essential tools
check_and_install_tools() {
    log_message "Checking and installing essential tools..."
    
    local tools_missing=()
    
    # Check for essential tools
    for tool in curl wget git unzip tar; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            tools_missing+=("$tool")
        fi
    done
    
    if [ ${#tools_missing[@]} -gt 0 ]; then
        echo -e "${YELLOW}⚠️ Missing tools: ${tools_missing[*]}${NC}"
        echo "Installing missing tools..."
        
        # Update package lists
        apt-get update
        
        # Install missing tools
        apt-get install -y "${tools_missing[@]}"
        
        echo -e "${GREEN}✅ Essential tools installed${NC}"
    else
        echo -e "${GREEN}✅ All essential tools available${NC}"
    fi
}

# Function to setup Python environment for Ubuntu 22.04+
setup_python_environment() {
    log_message "Setting up Python environment..."
    
    # Install python3-venv and pipx for better Python management
    apt-get install -y python3-venv python3-pip python3-full
    
    # Create a virtual environment for the trading bot
    echo "🐍 Creating Python virtual environment..."
    python3 -m venv /opt/trading-bot-venv
    
    # Activate virtual environment and upgrade pip
    echo "🔄 Upgrading pip in virtual environment..."
    /opt/trading-bot-venv/bin/pip install --upgrade pip
    
    # Configure pip to not use --user flag
    echo "⚙️ Configuring pip settings..."
    /opt/trading-bot-venv/bin/pip config set install.user false
    
    # Set proper permissions for the virtual environment
    chmod -R 755 /opt/trading-bot-venv
    chown -R trading-bot-user:trading-bot-user /opt/trading-bot-venv 2>/dev/null || true
    
    echo -e "${GREEN}✅ Python environment setup complete${NC}"
}

# Function to create necessary directories
create_directories() {
    log_message "Creating necessary directories..."
    
    # Create directories with proper permissions
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$CLOUD_INIT_DIR"
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Set proper ownership and permissions
    chown -R root:root "$INSTALL_DIR" 2>/dev/null || true
    chown -R root:root "$BACKUP_DIR" 2>/dev/null || true
    chown -R root:root "$CLOUD_INIT_DIR" 2>/dev/null || true
    
    chmod -R 755 "$INSTALL_DIR"
    chmod -R 755 "$BACKUP_DIR"
    chmod -R 755 "$CLOUD_INIT_DIR"
    
    echo -e "${GREEN}✅ Directories created with proper permissions${NC}"
}

# Function to backup existing installation
backup_existing() {
    if [ -d "$INSTALL_DIR" ] && [ "$(ls -A "$INSTALL_DIR")" ]; then
        log_message "Backing up existing installation..."
        
        local backup_name="trading-bot-backup-$(date +%Y%m%d-%H%M%S)"
        cp -r "$INSTALL_DIR" "$BACKUP_DIR/$backup_name"
        
        echo -e "${GREEN}✅ Existing installation backed up to $BACKUP_DIR/$backup_name${NC}"
    else
        log_message "No existing installation to backup"
    fi
}

# Function to download trading bot source with fallbacks
download_source() {
    log_message "Downloading trading bot source code..."
    
    # Ensure /tmp is clean and accessible
    cd /tmp
    rm -rf trading-bot-* 2>/dev/null || true
    
    # Ensure we have write permissions
    if [ ! -w /tmp ]; then
        echo -e "${RED}❌ No write permission to /tmp directory${NC}"
        exit 1
    fi
    
    # Try multiple download methods with fallbacks
    local download_success=false
    
    # Method 1: Try git clone first (most reliable)
    if command -v git >/dev/null 2>&1; then
        echo "🌐 Attempting git clone..."
        
        # Clean up any existing directories
        rm -rf trading-bot-git trading-bot-main
        
        # Try git clone with proper error handling
        if git clone "$GIT_REPO" trading-bot-git 2>/dev/null; then
            echo "✅ Git clone successful"
            rm -rf "$INSTALL_DIR"
            mv trading-bot-git "$INSTALL_DIR"
            download_success=true
        else
            echo "❌ Git clone failed, trying alternative methods..."
            # Clean up failed clone attempt
            rm -rf trading-bot-git 2>/dev/null || true
        fi
    fi
    
    # Method 2: Try ZIP download and extraction
    if [ "$download_success" = false ] && command -v unzip >/dev/null 2>&1; then
        echo "📥 Attempting ZIP download..."
        if curl -L -o trading-bot.zip "$REPO_URL" 2>/dev/null; then
            echo "📦 Extracting source code..."
            if unzip -q trading-bot.zip 2>/dev/null; then
                if [ -d "trading-bot-main" ]; then
                    rm -rf "$INSTALL_DIR"
                    mv trading-bot-main "$INSTALL_DIR"
                    download_success=true
                    echo "✅ ZIP extraction successful"
                fi
            fi
            rm -f trading-bot.zip
        fi
    fi
    
    # Method 3: Try tar.gz download and extraction
    if [ "$download_success" = false ] && command -v tar >/dev/null 2>&1; then
        echo "📥 Attempting tar.gz download..."
        local tar_url="https://github.com/KingGekko/trading-bot/archive/refs/heads/main.tar.gz"
        if curl -L -o trading-bot.tar.gz "$tar_url" 2>/dev/null; then
            echo "📦 Extracting source code..."
            if tar -xzf trading-bot.tar.gz 2>/dev/null; then
                if [ -d "trading-bot-main" ]; then
                    rm -rf "$INSTALL_DIR"
                    mv trading-bot-main "$INSTALL_DIR"
                    download_success=true
                    echo "✅ TAR extraction successful"
                fi
            fi
            rm -f trading-bot.tar.gz
        fi
    fi
    
    # Method 4: Manual file download as last resort
    if [ "$download_success" = false ]; then
        echo "📥 Attempting manual file download..."
        mkdir -p "$INSTALL_DIR"
        
        # Download essential files
        local files=(
            "Cargo.toml"
            "build.rs"
            "README.md"
            "src/main.rs"
            "src/ollama/mod.rs"
            "src/ollama/ollama_client.rs"
            "src/ollama/ollama_config.rs"
            "src/ollama/ollama_receipt.rs"
            "setup/install.sh"
            "setup/update.sh"
            "setup/README.md"
            "proto/receipt.proto"
        )
        
        local download_count=0
        for file in "${files[@]}"; do
            local dir=$(dirname "$file")
            mkdir -p "$INSTALL_DIR/$dir"
            
            local url="https://raw.githubusercontent.com/KingGekko/trading-bot/main/$file"
            if curl -s -o "$INSTALL_DIR/$file" "$url" 2>/dev/null; then
                ((download_count++))
            fi
        done
        
        if [ $download_count -gt 0 ]; then
            download_success=true
            echo "✅ Manual download successful ($download_count files)"
        fi
    fi
    
    if [ "$download_success" = false ]; then
        echo -e "${RED}❌ All download methods failed${NC}"
        echo "Please check your internet connection and try again"
        exit 1
    fi
    
    # Set proper permissions
    chmod -R 755 "$INSTALL_DIR"
    find "$INSTALL_DIR" -name "*.sh" -exec chmod +x {} \;
    
    echo -e "${GREEN}✅ Source code downloaded to $INSTALL_DIR${NC}"
    
    cd - > /dev/null
}

# Function to install dependencies
install_dependencies() {
    log_message "Installing system dependencies..."
    
    # Update package lists
    apt-get update
    
    # Install essential packages
    apt-get install -y \
        curl \
        wget \
        git \
        unzip \
        build-essential \
        cmake \
        pkg-config \
        python3 \
        python3-pip \
        protobuf-compiler \
        libprotobuf-dev \
        libssl-dev \
        clang \
        llvm-dev
    
    echo -e "${GREEN}✅ System dependencies installed${NC}"
}

# Function to run trading bot installation
run_installation() {
    log_message "Running trading bot installation..."
    
    cd "$INSTALL_DIR/setup"
    
    if [ -f "install.sh" ]; then
        echo "🔧 Running installation script..."
        chmod +x install.sh
        
        # Create a temporary user for running the install script
        # since the install script requires non-root execution
        if ! id "trading-bot-user" &>/dev/null; then
            echo "👤 Creating temporary user for installation..."
            useradd -m -s /bin/bash trading-bot-user
            usermod -aG sudo trading-bot-user
            echo "trading-bot-user ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/trading-bot-user
        fi
        
        # Copy the setup directory to the user's home
        cp -r "$INSTALL_DIR/setup" /home/trading-bot-user/
        chown -R trading-bot-user:trading-bot-user /home/trading-bot-user/setup
        
        # Run the install script as the non-root user with virtual environment
        echo "🔧 Running installation as trading-bot-user..."
        
        # Create a modified install script that removes --user flags
        # Use absolute paths to avoid directory issues
        cp "$INSTALL_DIR/setup/install.sh" "$INSTALL_DIR/setup/install.sh.modified"
        
        # Remove --user flags from pip commands
        sed -i 's/--user//g' "$INSTALL_DIR/setup/install.sh.modified"
        
        # Set environment variables to force virtual environment usage
        export PIP_USER=no
        export PIP_REQUIRE_VIRTUALENV=true
        export VIRTUAL_ENV=/opt/trading-bot-venv
        
        # Run the modified install script with proper environment
        su - trading-bot-user -c "cd $INSTALL_DIR/setup && source /opt/trading-bot-venv/bin/activate && export PIP_USER=no && export PIP_REQUIRE_VIRTUALENV=true && ./install.sh.modified"
        
        # Clean up modified script
        rm "$INSTALL_DIR/setup/install.sh.modified"
        
        # Copy back any generated files
        if [ -d "/home/trading-bot-user/.cargo" ]; then
            cp -r /home/trading-bot-user/.cargo /root/
            echo "✅ Rust environment copied to root user"
        fi
        
        # Handle Python environment issues
        echo "🐍 Setting up Python environment..."
        if [ -d "/home/trading-bot-user/.local" ]; then
            cp -r /home/trading-bot-user/.local /root/
            echo "✅ Python packages copied to root user"
        fi
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Installation completed successfully${NC}"
        else
            echo -e "${RED}❌ Installation failed${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Installation script not found${NC}"
        exit 1
    fi
    
    cd - > /dev/null
}

# Function to configure trading bot
configure_trading_bot() {
    log_message "Configuring trading bot..."
    
    cd "$INSTALL_DIR"
    
    # Create production config
    if [ ! -f "config.env" ]; then
        echo "📝 Creating production configuration..."
        cat > config.env << 'EOF'
# Trading Bot Production Configuration
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=auto

# Performance Optimization
MAX_RESPONSE_LENGTH=500

# Logging Configuration
LOG_LEVEL=info
LOG_DIRECTORY=/var/log/trading-bot

# Security Settings
ENABLE_SSL=true
ENABLE_AUTH=false

# Network Configuration
HOST=0.0.0.0
PORT=8080
EOF
        chmod 644 config.env
        echo -e "${GREEN}✅ Production configuration created${NC}"
    fi
    
    # Set proper permissions
    chown -R root:root "$INSTALL_DIR"
    chmod -R 755 "$INSTALL_DIR"
    chmod 644 config.env
    
    # Create log directory
    mkdir -p /var/log/trading-bot
    chown -R root:root /var/log/trading-bot
    chmod -R 755 /var/log/trading-bot
    
    cd - > /dev/null
}

# Function to create systemd service
create_systemd_service() {
    log_message "Creating systemd service..."
    
    cat > /etc/systemd/system/trading-bot.service << 'EOF'
[Unit]
Description=Trading Bot Service
After=network.target ollama.service
Wants=ollama.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/trading-bot
ExecStart=/opt/trading-bot/target/release/trading_bot
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=trading-bot

# Environment variables
Environment=LOG_LEVEL=info
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/trading-bot

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable trading-bot.service
    
    echo -e "${GREEN}✅ Systemd service created and enabled${NC}"
}

# Function to create cloud-init script
create_cloud_init() {
    log_message "Creating cloud-init script..."
    
    cat > "$CLOUD_INIT_DIR/trading-bot-cloud-init.yml" << 'EOF'
#cloud-config
# Trading Bot Cloud Init Configuration
# Use this for automated deployment on cloud platforms

package_update: true
package_upgrade: true

packages:
  - curl
  - wget
  - git
  - unzip
  - build-essential
  - cmake
  - pkg-config
  - python3
  - python3-pip
  - protobuf-compiler
  - libprotobuf-dev
  - libssl-dev
  - clang
  - llvm-dev

runcmd:
  # Create directories
  - mkdir -p /opt/trading-bot
  - mkdir -p /var/log/trading-bot
  
  # Download and install trading bot
  - cd /tmp
  - curl -L -o trading-bot.zip "https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip"
  - unzip -q trading-bot.zip
  - mv trading-bot-main/* /opt/trading-bot/
  - rm -rf trading-bot-main trading-bot.zip
  
  # Install Rust
  - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  - source /root/.cargo/env
  
  # Install Ollama
  - curl -fsSL https://ollama.ai/install.sh | sh
  
  # Build trading bot
  - cd /opt/trading-bot
  - source /root/.cargo/env
  - cargo build --release
  
  # Set permissions
  - chown -R root:root /opt/trading-bot
  - chmod -R 755 /opt/trading-bot
  - chmod 644 /opt/trading-bot/config.env
  - chown -R root:root /var/log/trading-bot
  - chmod -R 755 /var/log/trading-bot
  
  # Create systemd service
  - |
    cat > /etc/systemd/system/trading-bot.service << 'SERVICEEOF'
    [Unit]
    Description=Trading Bot Service
    After=network.target ollama.service
    Wants=ollama.service
    
    [Service]
    Type=simple
    User=root
    WorkingDirectory=/opt/trading-bot
    ExecStart=/opt/trading-bot/target/release/trading_bot
    Restart=always
    RestartSec=10
    StandardOutput=journal
    StandardError=journal
    SyslogIdentifier=trading-bot
    Environment=LOG_LEVEL=info
    Environment=RUST_LOG=info
    NoNewPrivileges=true
    PrivateTmp=true
    ProtectSystem=strict
    ProtectHome=true
    ReadWritePaths=/var/log/trading-bot
    
    [Install]
    WantedBy=multi-user.target
    SERVICEEOF
  
  # Enable and start services
  - systemctl daemon-reload
  - systemctl enable trading-bot.service
  - systemctl enable ollama.service
  - systemctl start ollama.service
  - systemctl start trading-bot.service

# Final message
final_message: "Trading Bot deployment completed successfully!"
EOF
    
    # Create deployment script
    cat > "$CLOUD_INIT_DIR/deploy-trading-bot.sh" << 'EOF'
#!/bin/bash
# Quick deployment script for trading bot
# Run this on any Ubuntu system to deploy the trading bot

set -e

echo "🚀 Quick Trading Bot Deployment"
echo "==============================="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ This script requires admin privileges!"
    echo "Please run with sudo: sudo ./deploy-trading-bot.sh"
    exit 1
fi

# Install dependencies
echo "📦 Installing dependencies..."
apt-get update
apt-get install -y curl wget git unzip build-essential cmake pkg-config \
    python3 python3-pip protobuf-compiler libprotobuf-dev libssl-dev clang llvm-dev

# Download and install
echo "📥 Downloading trading bot..."
cd /tmp
curl -L -o trading-bot.zip "https://github.com/KingGekko/trading-bot/archive/refs/heads/main.zip"
unzip -q trading-bot.zip
rm -rf /opt/trading-bot
mv trading-bot-main /opt/trading-bot

# Install Rust
echo "🦀 Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source /root/.cargo/env

# Install Ollama
echo "🤖 Installing Ollama..."
curl -fsSL https://ollama.ai/install.sh | sh

# Build
echo "🔨 Building trading bot..."
cd /opt/trading-bot
source /root/.cargo/env
cargo build --release

# Configure
echo "⚙️ Configuring..."
mkdir -p /var/log/trading-bot
chown -R root:root /opt/trading-bot /var/log/trading-bot
chmod -R 755 /opt/trading-bot /var/log/trading-bot

# Create service
echo "🔧 Creating systemd service..."
cat > /etc/systemd/system/trading-bot.service << 'SERVICEEOF'
[Unit]
Description=Trading Bot Service
After=network.target ollama.service
Wants=ollama.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/trading-bot
ExecStart=/opt/trading-bot/target/release/trading_bot
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=trading-bot
Environment=LOG_LEVEL=info
Environment=RUST_LOG=info
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/trading-bot

[Install]
WantedBy=multi-user.target
SERVICEEOF

# Enable and start
systemctl daemon-reload
systemctl enable trading-bot.service ollama.service
systemctl start ollama.service
systemctl start trading-bot.service

echo "✅ Trading Bot deployed successfully!"
echo "📊 Status:"
systemctl status trading-bot.service --no-pager -l
echo ""
echo "🔧 Management commands:"
echo "  Start:   sudo systemctl start trading-bot"
echo "  Stop:    sudo systemctl stop trading-bot"
echo "  Status:  sudo systemctl status trading-bot"
echo "  Logs:    sudo journalctl -u trading-bot -f"
EOF
    
    chmod +x "$CLOUD_INIT_DIR/deploy-trading-bot.sh"
    
    echo -e "${GREEN}✅ Cloud-init script created at $CLOUD_INIT_DIR/trading-bot-cloud-init.yml${NC}"
    echo -e "${GREEN}✅ Deployment script created at $CLOUD_INIT_DIR/deploy-trading-bot.sh${NC}"
}

# Function to test installation
test_installation() {
    log_message "Testing installation..."
    
    cd "$INSTALL_DIR"
    
    # Test protobuf
    if command -v protoc >/dev/null 2>&1; then
        echo "✅ Protobuf compiler available"
    else
        echo -e "${RED}❌ Protobuf compiler not found${NC}"
        return 1
    fi
    
    # Test Rust
    if command -v cargo >/dev/null 2>&1; then
        echo "✅ Rust toolchain available"
    else
        echo -e "${RED}❌ Rust toolchain not found${NC}"
        return 1
    fi
    
    # Test trading bot binary
    if [ -f "target/release/trading_bot" ]; then
        echo "✅ Trading bot binary built successfully"
        
        # Test binary execution
        if timeout 5s ./target/release/trading_bot --help >/dev/null 2>&1; then
            echo "✅ Trading bot binary executes correctly"
        else
            echo -e "${YELLOW}⚠️ Trading bot binary execution test failed (may be normal)${NC}"
        fi
    else
        echo -e "${RED}❌ Trading bot binary not found${NC}"
        return 1
    fi
    
    # Test Ollama
    if command -v ollama >/dev/null 2>&1; then
        echo "✅ Ollama available"
    else
        echo -e "${YELLOW}⚠️ Ollama not found${NC}"
    fi
    
    cd - > /dev/null
}

# Function to show deployment summary
show_summary() {
    echo ""
    echo -e "${GREEN}🎉 Trading Bot Deployment Completed Successfully!${NC}"
    echo ""
    echo "📋 Deployment Summary:"
    echo "  📁 Installation Directory: $INSTALL_DIR"
    echo "  📁 Backup Directory: $BACKUP_DIR"
    echo "  📁 Cloud-init Scripts: $CLOUD_INIT_DIR"
    echo "  📄 Log File: $LOG_FILE"
    echo ""
    echo "🔧 Management Commands:"
    echo "  Start Service:   sudo systemctl start trading-bot"
    echo "  Stop Service:    sudo systemctl stop trading-bot"
    echo "  Check Status:    sudo systemctl status trading-bot"
    echo "  View Logs:       sudo journalctl -u trading-bot -f"
    echo "  Restart Service: sudo systemctl restart trading-bot"
    echo ""
    echo "📚 Cloud Deployment:"
    echo "  Cloud-init YAML: $CLOUD_INIT_DIR/trading-bot-cloud-init.yml"
    echo "  Quick Deploy:    $CLOUD_INIT_DIR/deploy-trading-bot.sh"
    echo ""
    echo "💡 Next Steps:"
    echo "  1. Start the service: sudo systemctl start trading-bot"
    echo "  2. Check status: sudo systemctl status trading-bot"
    echo "  3. View logs: sudo journalctl -u trading-bot -f"
    echo "  4. Use cloud-init script for future deployments"
}

# Main execution
main() {
    log_message "Starting trading bot deployment..."
    
    # Check sudo privileges
    check_sudo
    
    # Check and install essential tools
    check_and_install_tools
    
    # Setup Python environment for Ubuntu 22.04+
    setup_python_environment
    
    # Create directories
    create_directories
    
    # Backup existing installation
    backup_existing
    
    # Download source code
    download_source
    
    # Install system dependencies
    install_dependencies
    
    # Run trading bot installation
    run_installation
    
    # Configure trading bot
    configure_trading_bot
    
    # Create systemd service
    create_systemd_service
    
    # Create cloud-init scripts
    create_cloud_init
    
    # Test installation
    test_installation
    
    # Show summary
    show_summary
    
    log_message "Deployment completed successfully"
}

# Run main function
main "$@" 