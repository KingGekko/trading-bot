# 🔧 Protobuf Fix Guide

This guide helps you fix the common protobuf installation issues that cause Rust builds to fail with errors like:

```
Error: Custom { kind: NotFound, error: "Could not find `protoc`" }
```

## 🚀 Quick Fix

**Run this command from your project root:**

```bash
./fix_protobuf.sh
```

This script will automatically:
- Detect your operating system
- Install protobuf using the appropriate method
- Set up environment variables
- Test the installation
- Verify your Rust build works

## 🔍 What Causes This Error?

The error occurs when:
1. **protoc** (Protocol Buffers compiler) is not installed
2. **PROTOC** environment variable is not set
3. **protoc** is not in your system PATH
4. **protobuf-compiler** package is missing

## 🛠️ Manual Installation Methods

### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install protobuf-compiler
```

### CentOS/RHEL/Fedora
```bash
sudo yum install protobuf-compiler
# or
sudo dnf install protobuf-compiler
```

### macOS
```bash
brew install protobuf
```

### Windows
Download from: https://github.com/protocolbuffers/protobuf/releases

## 🔧 Environment Variable Setup

After installation, set the PROTOC environment variable:

```bash
# Find protoc location
which protoc

# Set environment variable (replace /usr/bin/protoc with your actual path)
export PROTOC="/usr/bin/protoc"

# Make it permanent
echo 'export PROTOC="/usr/bin/protoc"' >> ~/.bashrc
echo 'export PROTOC="/usr/bin/protoc"' >> ~/.profile

# Reload shell
source ~/.bashrc
```

## 🧪 Testing the Fix

### Test protoc installation:
```bash
protoc --version
```

### Test protobuf compilation:
```bash
protoc --cpp_out=/tmp proto/receipt.proto
```

### Test Rust build:
```bash
cargo check
```

## 📋 Troubleshooting

### Issue: "Permission denied" during installation
```bash
# Make sure you have sudo privileges
sudo whoami
```

### Issue: "Package not found"
```bash
# Update package lists
sudo apt-get update  # Ubuntu/Debian
sudo yum update      # CentOS/RHEL
```

### Issue: "Build from source failed"
```bash
# Install build dependencies first
sudo apt-get install build-essential cmake pkg-config
```

### Issue: "Environment variable not working"
```bash
# Check if variable is set
echo $PROTOC

# Reload shell configuration
source ~/.bashrc
source ~/.profile
```

## 🔄 Alternative Solutions

### 1. Use Docker
```bash
docker run --rm -v $(pwd):/workspace -w /workspace \
  --env PROTOC=/usr/bin/protoc \
  rust:latest cargo build
```

### 2. Download Pre-built Binary
```bash
# Download from GitHub releases
curl -L -o protoc.zip \
  "https://github.com/protocolbuffers/protobuf/releases/download/v25.3/protoc-25.3-linux-x86_64.zip"

# Extract and install
unzip protoc.zip
sudo mv bin/protoc /usr/local/bin/
sudo chmod +x /usr/local/bin/protoc
```

### 3. Use System Package Manager
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# CentOS/RHEL
sudo yum install protobuf-compiler

# Arch Linux
sudo pacman -S protobuf

# Alpine Linux
sudo apk add protobuf
```

## 📚 Additional Resources

- [Protocol Buffers Documentation](https://developers.google.com/protocol-buffers)
- [prost-build Documentation](https://docs.rs/prost-build/)
- [Rust Protobuf Guide](https://github.com/tokio-rs/prost)
- [GitHub Releases](https://github.com/protocolbuffers/protobuf/releases)

## 🆘 Still Having Issues?

If none of the above solutions work:

1. **Check system requirements**: Ensure you have a supported OS and architecture
2. **Verify network access**: Make sure you can download packages and source code
3. **Check disk space**: Ensure you have enough free space for installation
4. **Review error logs**: Look for specific error messages in the output
5. **Try different versions**: Some systems work better with specific protobuf versions

## 📞 Support

For additional help:
- Check the error messages carefully
- Run `./fix_protobuf.sh` and share the output
- Provide your OS version and architecture
- Share the complete build error message

---

**Remember**: The `./fix_protobuf.sh` script handles most common cases automatically. Start there for the quickest solution! 