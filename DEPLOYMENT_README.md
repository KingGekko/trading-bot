# 🚀 Trading Bot Deployment Guide

This guide explains how to deploy the trading bot using the provided scripts and cloud-init templates.

## 📋 What's Included

### **1. Main Deployment Script (`deploy_trading_bot.sh`)**
- **Complete automation** of the entire deployment process
- **Downloads source code** from GitHub
- **Runs installation** from the setup folder
- **Creates cloud-init scripts** for future use
- **Sets up systemd service** for production use
- **Comprehensive logging** and error handling

### **2. Cloud-Init Template (`cloud-init-template.yml`)**
- **Ready-to-use** cloud-init configuration
- **Automated deployment** on cloud platforms
- **No manual intervention** required
- **Production-ready** configuration

### **3. Generated Scripts (after running deployment)**
- **Cloud-init YAML** for automated deployments
- **Quick deployment script** for manual installations
- **Systemd service** for production management

## 🚀 Quick Start

### **Option 1: Full Automated Deployment**
```bash
# Download and run the deployment script
curl -fsSL https://raw.githubusercontent.com/KingGekko/trading-bot/main/deploy_trading_bot.sh | sudo bash
```

### **Option 2: Manual Download and Run**
```bash
# Download the script
wget https://raw.githubusercontent.com/KingGekko/trading-bot/main/deploy_trading_bot.sh

# Make executable
chmod +x deploy_trading_bot.sh

# Run with sudo
sudo ./deploy_trading_bot.sh
```

## 🔧 What the Deployment Script Does

### **Phase 1: Preparation**
1. **📁 Creates directories** (`/opt/trading-bot`, `/opt/cloud-init-scripts`, etc.)
2. **💾 Backs up** existing installations (if any)
3. **📥 Downloads** latest source code from GitHub
4. **📦 Installs** system dependencies (protobuf, build tools, etc.)

### **Phase 2: Installation**
1. **🔧 Runs** the trading bot installation script
2. **⚙️ Configures** production settings
3. **🔨 Builds** the trading bot binary
4. **🧪 Tests** the installation

### **Phase 3: Production Setup**
1. **🔧 Creates** systemd service for automatic startup
2. **📝 Generates** cloud-init scripts for future deployments
3. **✅ Verifies** everything is working correctly
4. **📋 Shows** management commands and next steps

## ☁️ Cloud Deployment

### **Using Cloud-Init Template**

#### **AWS EC2:**
1. **Launch instance** with Ubuntu 22.04 LTS
2. **Advanced details** → User data
3. **Paste** the content of `cloud-init-template.yml`
4. **Launch instance** - it will auto-deploy!

#### **Google Cloud Platform:**
1. **Create VM instance**
2. **Metadata** → Startup script
3. **Paste** the cloud-init content
4. **Create** - automatic deployment!

#### **Azure:**
1. **Create VM**
2. **Advanced** → Custom data
3. **Paste** cloud-init content
4. **Review + create**

### **Manual Cloud Deployment**
```bash
# On your cloud instance, run:
curl -fsSL https://raw.githubusercontent.com/KingGekko/trading-bot/main/deploy_trading_bot.sh | sudo bash
```

## 🏗️ Local/VM Deployment

### **Step-by-Step Process:**
```bash
# 1. Download the deployment script
wget https://raw.githubusercontent.com/KingGekko/trading-bot/main/deploy_trading_bot.sh

# 2. Make it executable
chmod +x deploy_trading_bot.sh

# 3. Run with sudo (requires admin privileges)
sudo ./deploy_trading_bot.sh
```

### **What Happens:**
1. **📥 Downloads** latest trading bot source
2. **🔧 Installs** all dependencies automatically
3. **⚙️ Runs** the complete installation process
4. **🔨 Builds** the production binary
5. **📝 Creates** cloud-init scripts for future use
6. **🧪 Tests** everything works correctly

## 📁 Generated Files

After running the deployment script, you'll have:

```
/opt/
├── trading-bot/                    # Main installation
│   ├── target/release/trading_bot  # Production binary
│   ├── setup/                      # Installation scripts
│   └── config.env                  # Production configuration
├── trading-bot-backup/             # Backup of previous installation
└── cloud-init-scripts/             # Generated deployment scripts
    ├── trading-bot-cloud-init.yml  # Cloud-init YAML
    └── deploy-trading-bot.sh       # Quick deployment script
```

## 🔧 Management Commands

### **Service Management:**
```bash
# Start the trading bot
sudo systemctl start trading-bot

# Stop the trading bot
sudo systemctl stop trading-bot

# Check status
sudo systemctl status trading-bot

# View logs
sudo journalctl -u trading-bot -f

# Restart service
sudo systemctl restart trading-bot

# Enable auto-start
sudo systemctl enable trading-bot
```

### **Ollama Management:**
```bash
# Start Ollama
sudo systemctl start ollama

# Check Ollama status
sudo systemctl status ollama

# View Ollama logs
sudo journalctl -u ollama -f
```

## 📊 Monitoring and Logs

### **Trading Bot Logs:**
```bash
# Follow logs in real-time
sudo journalctl -u trading-bot -f

# View recent logs
sudo journalctl -u trading-bot -n 100

# View logs since boot
sudo journalctl -u trading-bot -b
```

### **System Logs:**
```bash
# View deployment log
tail -f /var/log/trading-bot-deployment.log

# View trading bot application logs
tail -f /var/log/trading-bot/*
```

## 🔄 Updates and Maintenance

### **Update Trading Bot:**
```bash
# Navigate to installation directory
cd /opt/trading-bot

# Run the update script
sudo ../setup/update.sh
```

### **Update Dependencies:**
```bash
# Update system packages
sudo apt update && sudo apt upgrade

# Update Rust toolchain
rustup update

# Update Cargo dependencies
cargo update
```

## 🚨 Troubleshooting

### **Common Issues:**

#### **1. Permission Denied:**
```bash
# Make sure you're running with sudo
sudo ./deploy_trading_bot.sh
```

#### **2. Build Failures:**
```bash
# Check if protobuf is installed
protoc --version

# If not, install manually
sudo apt-get install protobuf-compiler
```

#### **3. Service Won't Start:**
```bash
# Check service status
sudo systemctl status trading-bot

# View detailed logs
sudo journalctl -u trading-bot -n 50
```

#### **4. Ollama Issues:**
```bash
# Check Ollama status
sudo systemctl status ollama

# Restart Ollama
sudo systemctl restart ollama
```

### **Getting Help:**
1. **Check logs**: `sudo journalctl -u trading-bot -f`
2. **Verify installation**: `ls -la /opt/trading-bot/`
3. **Test binary**: `/opt/trading-bot/target/release/trading_bot --help`
4. **Check dependencies**: `which protoc`, `which cargo`

## 🌟 Advanced Features

### **Custom Configuration:**
Edit `/opt/trading-bot/config.env` to customize:
- Ollama model selection
- Log levels
- Network settings
- Performance parameters

### **Multiple Instances:**
Deploy multiple trading bots by:
1. **Copying** the installation to different directories
2. **Modifying** ports in config.env
3. **Creating** separate systemd services

### **Load Balancing:**
Use the cloud-init template to deploy multiple instances behind a load balancer.

## 📚 Additional Resources

- **GitHub Repository**: https://github.com/KingGekko/trading-bot
- **Protobuf Fix Guide**: `PROTOBUF_FIX.md`
- **Installation Guide**: `setup/README.md`
- **Update Guide**: `setup/update.sh`

## 🎯 Next Steps

After successful deployment:

1. **Start the service**: `sudo systemctl start trading-bot`
2. **Verify it's running**: `sudo systemctl status trading-bot`
3. **Check logs**: `sudo journalctl -u trading-bot -f`
4. **Test functionality**: Use the trading bot commands
5. **Set up monitoring**: Configure log rotation and monitoring
6. **Plan scaling**: Use cloud-init for additional instances

---

**Happy Trading! 🚀📈** 