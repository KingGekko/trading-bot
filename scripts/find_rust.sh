#!/bin/bash

# 🔍 Find Rust Installation Script
# Run this in your VM to locate Rust/Cargo

echo "🔍 Finding Rust Installation in VM"
echo "=================================="

# Check common locations
echo "📁 Checking common Rust locations..."

# Standard PATH
if command -v cargo >/dev/null 2>&1; then
    echo "✅ Rust found in PATH: $(which cargo)"
    echo "   Cargo version: $(cargo --version)"
    echo "   Rustc version: $(rustc --version)"
    exit 0
fi

# Common installation directories
RUST_LOCATIONS=(
    "$HOME/.cargo/bin"
    "/root/.cargo/bin"
    "/usr/local/cargo/bin"
    "/opt/rust/bin"
    "/usr/bin"
    "/usr/local/bin"
    "/opt/cargo/bin"
)

for location in "${RUST_LOCATIONS[@]}"; do
    if [ -f "$location/cargo" ]; then
        echo "✅ Rust found at: $location/cargo"
        echo "   Adding to PATH..."
        export PATH="$location:$PATH"
        
        # Test if it works
        if cargo --version >/dev/null 2>&1; then
            echo "   ✅ Cargo is now accessible!"
            echo "   Cargo version: $(cargo --version)"
            echo "   Rustc version: $(rustc --version)"
            echo ""
            echo "💡 To make this permanent, add to your ~/.bashrc:"
            echo "   export PATH=\"$location:\$PATH\""
            exit 0
        else
            echo "   ❌ Cargo found but not working"
        fi
    fi
done

# Search the entire system
echo ""
echo "🔍 Searching entire system for Rust..."
RUST_FOUND=$(find / -name "cargo" -type f 2>/dev/null | head -5)

if [ -n "$RUST_FOUND" ]; then
    echo "✅ Found Rust installations:"
    echo "$RUST_FOUND" | while read -r location; do
        echo "   📍 $location"
        if [ -x "$location" ]; then
            echo "      ✅ Executable"
            echo "      📊 Size: $(ls -lh "$location" | awk '{print $5}')"
            echo "      📅 Modified: $(ls -lh "$location" | awk '{print $6, $7, $8}')"
        else
            echo "      ❌ Not executable"
        fi
        echo ""
    done
    
    echo "💡 To use one of these, add to PATH:"
    echo "   export PATH=\"$(dirname "$(echo "$RUST_FOUND" | head -1)"):\$PATH\""
else
    echo "❌ No Rust installations found in the system"
    echo ""
    echo "🔧 Installing Rust..."
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "Then: source ~/.cargo/env"
fi
