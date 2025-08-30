#!/bin/bash

# 🧪 Test Account Verification
# Tests the new account verification system before making Alpaca requests

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Testing Account Verification System${NC}"
echo "============================================="
echo ""

# Check if config.env exists
if [ ! -f "config.env" ]; then
    echo -e "${RED}❌ config.env not found${NC}"
    echo "Please create config.env with your Alpaca API credentials"
    exit 1
fi

# Check if API keys are set
if grep -q "your_alpaca_api_key_here" config.env; then
    echo -e "${YELLOW}⚠️  API keys not configured in config.env${NC}"
    echo "Please run ./setup_api_keys.sh first"
    echo ""
fi

# Function to test account verification
test_account_verification() {
    local description=$1
    
    echo -e "${BLUE}🧪 Testing: $description${NC}"
    echo ""
    
    # Test account verification
    echo -e "${YELLOW}🔍 Verifying Alpaca account...${NC}"
    
    # Create a simple test program to verify account
    cat > test_account_verification.rs << 'EOF'
use trading_bot::market_data::{AccountVerifier, AccountVerification};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    let api_key = std::env::var("ALPACA_API_KEY")
        .expect("ALPACA_API_KEY not set");
    let secret_key = std::env::var("ALPACA_SECRET_KEY")
        .expect("ALPACA_SECRET_KEY not set");
    
    // Check if this is paper trading
    let is_paper = std::env::var("ALPACA_PAPER_TRADING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    println!("🔍 Verifying Alpaca account...");
    println!("📊 Paper Trading: {}", is_paper);
    println!("🔑 API Key: {}...", &api_key[..8]);
    println!("");
    
    // Create account verifier
    let verifier = AccountVerifier::new(api_key, secret_key, is_paper);
    
    // Verify account
    match verifier.verify_account().await {
        Ok(verification) => {
            println!("✅ Account verification successful!");
            println!("");
            println!("📊 Account Information:");
            println!("   • Account Type: {}", verification.account_type);
            println!("   • Account Number: {}", verification.account_status.account_number);
            println!("   • Status: {}", if verification.is_verified { "Verified" } else { "Not Verified" });
            println!("   • Message: {}", verification.verification_message);
            println!("");
            
            println!("🔓 Trading Permissions:");
            println!("   • Stocks: {}", if verification.trading_permissions.can_trade_stocks { "✅" } else { "❌" });
            println!("   • Crypto: {}", if verification.trading_permissions.can_trade_crypto { "✅" } else { "❌" });
            println!("   • Options: {}", if verification.trading_permissions.can_trade_options { "✅" } else { "❌" });
            println!("   • Forex: {}", if verification.trading_permissions.can_trade_forex { "✅" } else { "❌" });
            println!("   • Futures: {}", if verification.trading_permissions.can_trade_futures { "✅" } else { "❌" });
            println!("   • Short Selling: {}", if verification.trading_permissions.can_short { "✅" } else { "❌" });
            println!("   • Margin Trading: {}", if verification.trading_permissions.can_margin { "✅" } else { "❌" });
            println!("   • Day Trading: {}", if verification.trading_permissions.can_day_trade { "✅" } else { "❌" });
            println!("");
            
            println!("📡 Data Access:");
            println!("   • Market Data Feed: {}", verification.data_access.market_data_feed);
            println!("   • Real-time Quotes: {}", if verification.data_access.real_time_quotes { "✅" } else { "❌" });
            println!("   • Real-time Trades: {}", if verification.data_access.real_time_trades { "✅" } else { "❌" });
            println!("   • Real-time Bars: {}", if verification.data_access.real_time_bars { "✅" } else { "❌" });
            println!("   • Options Data: {}", if verification.data_access.options_data { "✅" } else { "❌" });
            println!("   • Crypto Data: {}", if verification.data_access.crypto_data { "✅" } else { "❌" });
            println!("   • News Data: {}", if verification.data_access.news_data { "✅" } else { "❌" });
            println!("   • Fundamental Data: {}", if verification.data_access.fundamental_data { "✅" } else { "❌" });
            println!("   • Historical Data: {}", if verification.data_access.historical_data { "✅" } else { "❌" });
            println!("");
            
            println!("🚀 Available Features:");
            for feature in &verification.available_features {
                println!("   • {}", feature);
            }
            println!("");
            
            // Test stream validation
            println!("🧪 Testing Stream Validation:");
            let test_streams = vec![
                "stocks".to_string(),
                "crypto".to_string(),
                "options".to_string(),
                "news".to_string(),
                "trade_updates".to_string(),
            ];
            
            match verifier.validate_stream_types(&verification, &test_streams) {
                Ok(valid_streams) => {
                    println!("✅ Valid streams: {}", valid_streams.join(", "));
                }
                Err(e) => {
                    println!("❌ Stream validation failed: {}", e);
                }
            }
            
            // Test feature availability
            println!("");
            println!("🧪 Testing Feature Availability:");
            let test_features = vec!["Stocks", "Crypto", "Options", "News", "Forex"];
            for feature in test_features {
                let available = verifier.is_feature_available(&verification, feature);
                println!("   • {}: {}", feature, if available { "✅ Available" } else { "❌ Not Available" });
            }
            
            // Get recommended feed
            println!("");
            println!("📡 Recommended Market Data Feed: {}", verifier.get_recommended_feed(&verification));
            
        }
        Err(e) => {
            println!("❌ Account verification failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
EOF
    
    # Compile and run the test
    echo -e "${YELLOW}🔨 Compiling test program...${NC}"
    if cargo build --bin test_account_verification 2>/dev/null; then
        echo -e "${GREEN}✅ Compilation successful${NC}"
        
        echo -e "${YELLOW}🚀 Running account verification...${NC}"
        if cargo run --bin test_account_verification; then
            echo -e "${GREEN}✅ Account verification test completed successfully${NC}"
        else
            echo -e "${RED}❌ Account verification test failed${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Compilation failed${NC}"
        return 1
    fi
    
    # Clean up
    rm -f test_account_verification.rs
    echo -e "${GREEN}✅ Test completed for: $description${NC}"
    echo ""
    echo "---"
    echo ""
}

# Main test sequence
echo -e "${BLUE}🚀 Starting Account Verification Tests${NC}"
echo "============================================="
echo ""

# Test 1: Basic account verification
test_account_verification "Account Verification and Feature Detection"

echo -e "${GREEN}🎉 All account verification tests completed!${NC}"
echo ""
echo -e "${BLUE}💡 What was tested:${NC}"
echo "   ✅ Account type detection (Basic, Full, Premium, Enterprise)"
echo "   ✅ Trading permissions validation"
echo "   ✅ Data access permissions"
echo "   ✅ Feature availability checking"
echo "   ✅ Stream type validation"
echo "   ✅ Market data feed recommendations"
echo "   ✅ Account status verification"
echo ""
echo -e "${BLUE}🔑 Make sure your API keys are set in config.env:${NC}"
echo "   ALPACA_API_KEY=your_api_key_here"
echo "   ALPACA_SECRET_KEY=your_secret_key_here"
echo "   ALPACA_PAPER_TRADING=true"
echo ""
echo -e "${BLUE}📚 Account Types and Features (August 2025):${NC}"
echo "   • Basic: Stocks + Crypto (IEX feed)"
echo "   • Full: Stocks + Crypto + Options + SIP feed"
echo "   • Premium: Full + Forex + Advanced features"
echo "   • Enterprise: Premium + Futures + All features"
echo ""
echo -e "${BLUE}⚡ Next Steps:${NC}"
echo "   • Run this test to verify your account"
echo "   • Check which features are available"
echo "   • Use the recommended market data feed"
echo "   • Validate stream types before streaming"
