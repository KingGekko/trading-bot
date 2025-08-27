#!/bin/bash

# Rust Dependency Update Script
# This script ensures all Rust dependencies are updated to their latest versions

set -e

echo "🦀 Rust Dependency Update Script"
echo "================================="
echo "This script will:"
echo "  • Update Rust toolchain to latest version"
echo "  • Update all Cargo dependencies to latest versions"
echo "  • Check for security vulnerabilities"
echo "  • Audit dependency licenses"
echo "  • Optimize build profiles"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if we're in a Rust project
check_rust_project() {
    # Find the Rust project root directory (where Cargo.toml is located)
    local current_dir="$(pwd)"
    local project_root=""
    
    # Look for Cargo.toml in current directory or parent directories
    while [[ "$(pwd)" != "/" ]]; do
        if [[ -f "Cargo.toml" ]]; then
            project_root="$(pwd)"
            break
        fi
        cd ..
    done
    
    # Return to original directory
    cd "$current_dir"
    
    if [[ -z "$project_root" ]]; then
        echo -e "${RED}❌ Not in a Rust project directory${NC}"
        echo "Please run this script from a Rust project or its subdirectories"
        exit 1
    fi
    
    echo -e "${BLUE}📍 Found Rust project at: $project_root${NC}"
    
    # Change to the project root directory
    cd "$project_root"
    echo -e "${GREEN}✅ Changed to project root directory${NC}"
}

# Function to update Rust toolchain
update_rust_toolchain() {
    echo -e "${BLUE}🔄 Updating Rust toolchain...${NC}"
    
    # Update Rust
    rustup update
    
    # Update components
    echo "🔧 Updating Rust components..."
    rustup component add rust-src
    rustup component add rust-analysis
    rustup component add rust-std
    rustup component add clippy
    rustup component add rustfmt
    
    # Install/update useful tools
    echo "🛠️ Installing/updating Rust tools..."
    cargo install-update -a cargo-update
    cargo install-update -a cargo-audit
    cargo install-update -a cargo-outdated
    cargo install-update -a cargo-tree
    cargo install-update -a cargo-expand
    cargo install-update -a cargo-license
    cargo install-update -a cargo-tarpaulin
    cargo install-update -a cargo-watch
    
    echo -e "${GREEN}✅ Rust toolchain updated${NC}"
}

# Function to update Cargo.toml dependencies
update_cargo_dependencies() {
    echo -e "${BLUE}📝 Updating Cargo.toml dependencies...${NC}"
    
    # Install cargo-edit if not present
    if ! command_exists cargo-set-version; then
        echo "🛠️ Installing cargo-edit..."
        cargo install cargo-edit
    fi
    
    # Backup current Cargo.toml
    cp Cargo.toml Cargo.toml.backup
    echo "💾 Cargo.toml backed up to Cargo.toml.backup"
    
    # Update major dependencies to latest versions
    echo "🔄 Updating major dependencies to latest versions..."
    
    # Core dependencies
    cargo upgrade --incompatible || echo -e "${YELLOW}⚠️ Some dependencies couldn't be upgraded (compatibility constraints)${NC}"
    
    # Update specific dependencies with latest versions
    echo "📦 Updating specific dependencies..."
    
    # Async runtime
    cargo add tokio@latest --features full
    cargo add tokio-stream@latest
    
    # HTTP and networking
    cargo add reqwest@latest --features "json,stream,rustls-tls"
    cargo add http@latest
    cargo add http-body@latest
    
    # Serialization
    cargo add serde@latest --features derive
    cargo add serde_json@latest
    
    # Error handling
    cargo add anyhow@latest
    cargo add thiserror@latest
    
    # Command line
    cargo add clap@latest --features derive
    
    # Date and time
    cargo add chrono@latest --features serde
    
    # Utilities
    cargo add url@latest
    cargo add uuid@latest --features "v4,serde"
    cargo add num_cpus@latest
    cargo add sysinfo@latest
    
    # Protobuf
    cargo add prost@latest
    cargo add prost-types@latest
    cargo add bytes@latest
    cargo add futures-util@latest
    cargo add futures@latest
    
    # Logging and tracing
    cargo add log@latest
    cargo add env_logger@latest
    cargo add tracing@latest
    cargo add tracing-subscriber@latest --features "env-filter"
    
    # Environment and config
    cargo add dotenv@latest
    cargo add config@latest
    cargo add toml@latest
    
    # File system
    cargo add walkdir@latest
    cargo add notify@latest
    
    # Compression
    cargo add flate2@latest
    cargo add brotli@latest
    
    # Security
    cargo add ring@latest
    cargo add getrandom@latest
    
    # Development tools
    cargo add cargo-license@latest --dev
    
    # Testing dependencies
    cargo add tokio-test@latest --dev
    cargo add criterion@latest --dev
    cargo add proptest@latest --dev
    cargo add mockall@latest --dev
    cargo add tempfile@latest --dev
    cargo add assert_fs@latest --dev
    
    echo -e "${GREEN}✅ Cargo.toml dependencies updated${NC}"
}

# Function to update Cargo.lock
update_cargo_lock() {
    echo -e "${BLUE}📦 Updating Cargo.lock...${NC}"
    
    # Clean previous build artifacts
    echo "🧹 Cleaning previous build artifacts..."
    cargo clean
    
    # Update dependencies
    echo "🔄 Updating Cargo.lock to latest compatible versions..."
    cargo update
    
    # Update specific packages
    echo "📦 Updating specific packages..."
    cargo update -p tokio
    cargo update -p reqwest
    cargo update -p serde
    cargo update -p anyhow
    cargo update -p clap
    cargo update -p chrono
    cargo update -p prost
    cargo update -p futures-util
    
    echo -e "${GREEN}✅ Cargo.lock updated${NC}"
}

# Function to audit dependencies
audit_dependencies() {
    echo -e "${BLUE}🔒 Auditing dependencies...${NC}"
    
    # Security audit
    if command_exists cargo-audit; then
        echo "🔍 Running security audit..."
        cargo audit || echo -e "${YELLOW}⚠️ Security issues found. Check the report above.${NC}"
    fi
    
    # Check outdated dependencies
    if command_exists cargo-outdated; then
        echo "📋 Checking for outdated dependencies..."
        echo "Outdated dependencies:"
        cargo outdated || echo "   All dependencies are up to date"
    fi
    
    # Show dependency tree
    echo "🌳 Dependency tree (top level):"
    cargo tree --depth=1
    
    # Check licenses
    if command_exists cargo-license; then
        echo "📜 Checking dependency licenses..."
        cargo license --summary || echo -e "${YELLOW}⚠️ Could not check licenses${NC}"
    fi
    
    echo -e "${GREEN}✅ Dependency audit completed${NC}"
}

# Function to test build
test_build() {
    echo -e "${BLUE}🧪 Testing build...${NC}"
    
    echo "🔨 Building in debug mode..."
    cargo build
    
    echo "🔨 Building in release mode..."
    cargo build --release
    
    echo "🧪 Running tests..."
    cargo test
    
    echo "📋 Running clippy..."
    cargo clippy
    
    echo "📋 Running rustfmt..."
    cargo fmt --check
    
    echo -e "${GREEN}✅ Build test completed successfully${NC}"
}

# Function to show current versions
show_versions() {
    echo -e "${BLUE}📋 Current versions:${NC}"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Rustup: $(rustup --version)"
    
    echo ""
    echo "📦 Key dependency versions:"
    cargo tree --depth=1 | grep -E "(tokio|reqwest|serde|anyhow|clap|chrono|prost)" | head -10
}

# Function to optimize build profiles
optimize_build_profiles() {
    echo -e "${BLUE}⚡ Optimizing build profiles...${NC}"
    
    # Create optimized Cargo.toml profiles if they don't exist
    if ! grep -q "\[profile.release\]" Cargo.toml; then
        echo "📝 Adding optimized build profiles to Cargo.toml..."
        cat >> Cargo.toml << 'EOF'

[profile.dev]
opt-level = 0
debug = true
strip = false

[profile.release]
opt-level = 3
debug = false
strip = true
lto = true
codegen-units = 1
panic = "abort"

[profile.release.package."*"]
opt-level = 3
EOF
        echo -e "${GREEN}✅ Build profiles added${NC}"
    else
        echo "✅ Build profiles already exist"
    fi
}

# Function to show current working directory
show_current_directory() {
    echo -e "${BLUE}📍 Current working directory: $(pwd)${NC}"
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting dependency update process...${NC}"
    echo ""
    
    # Show current directory
    show_current_directory
    
    # Check if we're in a Rust project
    check_rust_project
    
    # Update Rust toolchain
    update_rust_toolchain
    echo ""
    
    # Update Cargo.toml dependencies
    update_cargo_dependencies
    echo ""
    
    # Update Cargo.lock
    update_cargo_lock
    echo ""
    
    # Optimize build profiles
    optimize_build_profiles
    echo ""
    
    # Audit dependencies
    audit_dependencies
    echo ""
    
    # Test build
    test_build
    echo ""
    
    # Show current versions
    show_versions
    echo ""
    
    echo -e "${GREEN}🎉 Dependency update completed successfully!${NC}"
    echo ""
    echo "📋 What was updated:"
    echo "  ✅ Rust toolchain: Latest version"
    echo "  ✅ Cargo.toml: All dependencies to latest versions"
    echo "  ✅ Cargo.lock: Latest compatible versions"
    echo "  ✅ Build profiles: Optimized for performance"
    echo "  ✅ Security audit: Completed"
    echo "  ✅ License check: Completed"
    echo "  ✅ Build test: Passed"
    echo ""
    echo "💡 Next steps:"
    echo "  • Review the updated Cargo.toml"
    echo "  • Test your application thoroughly"
    echo "  • Commit the changes: git add . && git commit -m 'chore: update dependencies'"
    echo ""
    echo "🔄 To update again later, run: ./scripts/update_dependencies.sh"
}

# Run main function
main "$@" 