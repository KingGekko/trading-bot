use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use dotenv::dotenv;
use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

mod ollama;
mod api;
mod market_data;
mod trading_strategy;
mod protobuf;
mod order_execution;
mod interactive_setup;

use ollama::{OllamaClient, Config, OllamaReceipt};
use api::start_api_server;
use trading_strategy::{
    MarketDataPoint, 
    AccountData
};
use chrono::{DateTime, Utc};
use protobuf::ProtobufStorage;
use order_execution::OrderExecutor;
use interactive_setup::InteractiveSetup;

/// Ensure all required directories exist with proper permissions
async fn ensure_required_directories() -> Result<()> {
    let required_dirs = [
        "ollama_logs",
        "trading_portfolio", 
        "live_data",
        "sandbox_data",
        "logs",
    ];
    
    println!("🔧 Setting up required directories...");
    
    for dir in &required_dirs {
        let path = Path::new(dir);
        
        // Create directory if it doesn't exist
        if !path.exists() {
            match tokio::fs::create_dir_all(path).await {
                Ok(_) => println!("✅ Created directory: {}", dir),
                Err(e) => {
                    println!("⚠️  Failed to create directory {}: {}", dir, e);
                    continue;
                }
            }
        }
        
        // Set permissions on Unix systems
        #[cfg(unix)]
        {
            if let Ok(metadata) = tokio::fs::metadata(path).await {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                if let Err(e) = tokio::fs::set_permissions(path, perms).await {
                    println!("⚠️  Failed to set permissions on {}: {}", dir, e);
                }
            }
        }
    }
    
    Ok(())
}

/// Ensure config.env exists with default values
async fn ensure_config_file() -> Result<()> {
    let config_path = Path::new("config.env");
    
    if !config_path.exists() {
        println!("📝 Creating default config.env file...");
        
        let default_config = r#"# ========================================
# TRADING BOT UNIFIED CONFIGURATION
# ========================================

# ========================================
# ALPACA API CONFIGURATION
# ========================================

# Your Alpaca API credentials
APCA_API_KEY_ID=your_paper_trading_key
APCA_API_SECRET_KEY=your_paper_trading_secret

# Alpaca base URL (use paper trading for testing)
ALPACA_BASE_URL=https://paper-api.alpaca.markets

# ========================================
# OLLAMA AI CONFIGURATION
# ========================================

# Ollama base URL
OLLAMA_BASE_URL=http://localhost:11434

# Default AI model to use
DEFAULT_AI_MODEL=tinyllama

# Maximum timeout for Ollama requests in seconds
MAX_TIMEOUT_SECONDS=300

# Analysis interval in seconds
ANALYSIS_INTERVAL_SECONDS=30

# ========================================
# LOGGING CONFIGURATION
# ========================================

# Log level (debug, info, warn, error)
LOG_LEVEL=info

# Log directory
LOG_DIRECTORY=ollama_logs
"#;
        
        match tokio::fs::write(config_path, default_config).await {
            Ok(_) => println!("✅ Created config.env with default values"),
            Err(e) => println!("⚠️  Failed to create config.env: {}", e),
        }
    } else {
        println!("✅ config.env already exists");
    }
    
    Ok(())
}

/// Ensure log directory exists with proper permissions
async fn ensure_log_directory(log_directory: &str) -> Result<()> {
    let path = Path::new(log_directory);
    
    // Create directory if it doesn't exist
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
        log::info!("Created log directory: {}", log_directory);
    }
    
    // Ensure directory is writable
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Log path is not a directory: {}", log_directory));
    }
    
    // Check if directory is writable by trying to create a test file
    let test_file = path.join(".test_write");
    if let Err(e) = tokio::fs::write(&test_file, "test").await {
        log::warn!("Log directory not writable: {} ({}). Trying fallback locations...", log_directory, e);
        
        // Try fallback locations
        let _temp_dir = std::env::var("TEMP").unwrap_or_else(|_| "./temp".to_string());
        let fallback_locations = [
            "./logs", 
            "./ollama_logs", 
            #[cfg(unix)]
            "/tmp/ollama_logs",
            #[cfg(windows)]
            "./temp/ollama_logs",
            #[cfg(windows)]
            _temp_dir.as_str(),
        ];
        for fallback in &fallback_locations {
            let fallback_path = Path::new(fallback);
            if !fallback_path.exists() {
                if let Ok(_) = tokio::fs::create_dir_all(fallback_path).await {
                    log::info!("Using fallback log directory: {}", fallback);
                    return Ok(());
                }
            } else if fallback_path.is_dir() {
                let test_fallback = fallback_path.join(".test_write");
                if tokio::fs::write(&test_fallback, "test").await.is_ok() {
                    tokio::fs::remove_file(test_fallback).await.ok();
                    log::info!("Using fallback log directory: {}", fallback);
                    return Ok(());
                }
            }
        }
        
        log::warn!("All log directories failed. Logging will be limited.");
    } else {
        // Clean up test file
        tokio::fs::remove_file(test_file).await.ok();
        log::info!("Log directory is writable: {}", log_directory);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup required directories and config file
    if let Err(e) = ensure_required_directories().await {
        println!("⚠️  Warning: Failed to create some directories: {}", e);
    }
    
    if let Err(e) = ensure_config_file().await {
        println!("⚠️  Warning: Failed to create config file: {}", e);
    }
    
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    let app = Command::new("Trading Bot")
        .version("0.1.0")
        .author("Your Name")
        .about("A Rust-based trading bot with Ollama integration")
        // Interactive argument moved to the end to avoid conflicts
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .value_name("TEXT")
                .help("Send a single prompt to Ollama"),
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .value_name("PROMPT")
                .help("Run performance test with detailed timing analysis"),
        )
        .arg(
            Arg::new("logs")
                .short('l')
                .long("logs")
                .help("View prettified receipt logs")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stream")
                .short('s')
                .long("stream")
                .value_name("PROMPT")
                .help("Send a prompt with real-time streaming response"),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL_NAME")
                .help("Override auto-detection and use specific Ollama model"),
        )

        .arg(
            Arg::new("api")
                .long("api")
                .help("Start the JSON streaming API server")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("api-port")
                .long("api-port")
                .value_name("PORT")
                .help("Port for the API server (default: 8080)")
                .default_value("8080"),
        )
        .arg(
            Arg::new("websocket")
                .long("websocket")
                .help("Start WebSocket-based market data streaming")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stream-types")
                .long("stream-types")
                .value_name("TYPES")
                .help("Comma-separated stream types: stocks,crypto,options,news (default: all)")
                .default_value("stocks,crypto,options,news"),
        )
        .arg(
            Arg::new("simulated")
                .long("simulated")
                .help("Run in simulated mode with generated market data (no real API calls)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("trading")
                .long("trading")
                .help("Start trading screen with account monitoring and market hours")
                .action(clap::ArgAction::SetTrue),
        )

        .arg(
            Arg::new("enhanced-strategy")
                .long("enhanced-strategy")
                .help("Enhanced Trading Strategy with Asset Universe and Positions - Includes current positions, asset universe, and rebalancing recommendations")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("market-regime")
                .long("market-regime")
                .help("Market Regime Analysis - Analyze current market environment using asset universe data")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("portfolio-analysis")
                .long("portfolio-analysis")
                .help("Analyze portfolio with Ollama AI - Stream portfolio insights and recommendations")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("protobuf-storage")
                .long("protobuf-storage")
                .help("Test protobuf-based trading data storage")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("view-protobuf")
                .long("view-protobuf")
                .help("View stored protobuf data in readable format")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("export-protobuf")
                .long("export-protobuf")
                .help("Export protobuf data to JSON format")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("append-test")
                .long("append-test")
                .help("Test appending various data types to protobuf storage")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("protobuf-stats")
                .long("protobuf-stats")
                .help("Show statistics about stored protobuf data")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("execute-orders")
                .long("execute-orders")
                .help("Execute orders based on strategy recommendations and liquidation triggers")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("test-orders")
                .long("test-orders")
                .help("Test order execution system without actually placing orders (demo mode)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("enhanced-decisions")
                .long("enhanced-decisions")
                .help("Test enhanced AI decision engine with long/short capabilities")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ai-decisions")
                .long("ai-decisions")
                .help("Test AI-enhanced decision engine combining mathematical analysis with AI insights")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .short('i')
                .help("Start interactive setup wizard for automatic trading")
                .action(clap::ArgAction::SetTrue)
        );

    let matches = app.get_matches();

    // Load configuration
    let mut config = Config::from_env()?;
    
    println!("🚀 Trading Bot initialized successfully!");
    println!("📁 Working directory: {}", std::env::current_dir()?.display());
    println!("🔧 Configuration loaded from: config.env");
    
    // Auto-detect Ollama model if set to "auto"
    if let Err(e) = config.auto_detect_model().await {
        log::warn!("Failed to auto-detect model: {}. Using configured model: {}", e, config.ollama_model);
    }
    
    // Check if user wants to override the model
    if let Some(model_override) = matches.get_one::<String>("model") {
        log::info!("🔄 Overriding auto-detected model with: {}", model_override);
        config.ollama_model = model_override.clone();
    }
    
    // Display detected model information
    println!("🤖 Trading Bot - AI Model Detected");
    println!("{}", "=".repeat(50));
    println!("Model: {}", config.ollama_model);
    println!("Performance: {}", config.get_model_info());
    println!("Base URL: {}", config.ollama_base_url);
    println!("{}", "=".repeat(50));
    println!();
    
    // Ensure log directory is properly set up
    if let Err(e) = ensure_log_directory(&config.log_directory).await {
        log::warn!("Failed to set up log directory: {}. Continuing with limited logging.", e);
    }
    
    // Initialize Ollama client with configurable timeout
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);

    if matches.get_flag("logs") {
        // View logs mode
        println!("📋 OLLAMA RECEIPT LOGS");
        println!("{}", "=".repeat(50));
        
        // Display success receipts
        println!("\n✅ SUCCESS RECEIPTS:");
        let success_path = format!("{}/success_receipts.jsonl", config.log_directory);
        match OllamaReceipt::load_receipts_from_file(&success_path) {
            Ok(receipts) => {
                if receipts.is_empty() {
                    println!("   No success receipts found.");
                } else {
                    for (i, receipt) in receipts.iter().enumerate() {
                        receipt.display_receipt_summary(Some(i));
                    }
                    println!("   Total: {} successful transactions", receipts.len());
                }
            }
            Err(e) => println!("   Error reading success receipts: {}", e),
        }
        
        // Display failure receipts
        println!("\n❌ FAILURE RECEIPTS:");
        let failure_path = format!("{}/failure_receipts.jsonl", config.log_directory);
        match OllamaReceipt::load_receipts_from_file(&failure_path) {
            Ok(receipts) => {
                if receipts.is_empty() {
                    println!("   No failure receipts found.");
                } else {
                    for (i, receipt) in receipts.iter().enumerate() {
                        receipt.display_receipt_summary(Some(i));
                    }
                    println!("   Total: {} failed transactions", receipts.len());
                }
            }
            Err(e) => println!("   Error reading failure receipts: {}", e),
        }
        
        println!("\n📁 Log files location:");
        println!("   Success: {}/success_receipts.jsonl", config.log_directory);
        println!("   Failure: {}/failure_receipts.jsonl", config.log_directory);
        
    } else if let Some(prompt) = matches.get_one::<String>("stream") {
        // Streaming mode with real-time response
        // Sanitize input for security
        let sanitized_prompt = match config.sanitize_input(prompt) {
            Ok(sanitized) => sanitized,
            Err(e) => {
                eprintln!("Input validation error: {}", e);
                return Ok(());
            }
        };
        
        println!("🌊 STREAMING MODE");
        println!("Prompt: {}", sanitized_prompt);
        println!("Model: {} ({})", config.ollama_model, config.get_model_info());
        println!("{}",  "=".repeat(50));
        
        match ollama_client.generate_stream_with_timing(&config.ollama_model, &sanitized_prompt).await {
            Ok((chunks, receipt)) => {
                println!("Response: ");
                print!("Bot: ");
                io::stdout().flush().unwrap();
                
                let mut full_response = String::new();
                
                for chunk in chunks {
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                    full_response.push_str(&chunk);
                    
                    // Small delay to simulate real-time streaming
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                
                println!(); // New line after streaming
                println!();
                
                receipt.log_summary(&config.log_directory);
                
                println!("✅ Streaming completed!");
            }
            Err(e) => {
                println!("❌ Streaming failed!");
                eprintln!("Error: {}", e);
            }
        }
        

        
    } else if let Some(prompt) = matches.get_one::<String>("test") {
        // Test mode with detailed timing
        println!("🧪 PERFORMANCE TEST MODE");
        
        // Sanitize input for security
        let sanitized_prompt = match config.sanitize_input(prompt) {
            Ok(sanitized) => sanitized,
            Err(e) => {
                eprintln!("Input validation error: {}", e);
                return Ok(());
            }
        };
        
        println!("Testing prompt: {}", sanitized_prompt);
        println!("Model: {} ({})", config.ollama_model, config.get_model_info());
        println!("{}",  "=".repeat(50));
        
        match ollama_client.generate_stream_with_timing(&config.ollama_model, &sanitized_prompt).await {
            Ok((chunks, receipt)) => {
                println!("✅ Test completed successfully!");
                print!("Response: ");
                
                let mut full_response = String::new();
                for chunk in chunks {
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                    full_response.push_str(&chunk);
                    
                    // Minimal delay for ultra-fast responses
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                }
                println!(); // New line after response
                println!();
                
                receipt.log_detailed();
                receipt.log_summary(&config.log_directory);
            }
            Err(e) => {
                println!("❌ Test failed!");
                eprintln!("Error: {}", e);
            }
        }
    } else if let Some(prompt) = matches.get_one::<String>("prompt") {
        // Single prompt mode
        // Sanitize input for security
        let sanitized_prompt = match config.sanitize_input(prompt) {
            Ok(sanitized) => sanitized,
            Err(e) => {
                eprintln!("Input validation error: {}", e);
                return Ok(());
            }
        };
        
        println!("🌊 Sending prompt to Ollama (streaming)...");
        match ollama_client.generate_stream_with_timing(&config.ollama_model, &sanitized_prompt).await {
            Ok((chunks, receipt)) => {
                print!("Response: ");
                
                let mut full_response = String::new();
                for chunk in chunks {
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                    full_response.push_str(&chunk);
                    
                    // Minimal delay for ultra-fast responses  
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                }
                println!(); // New line after response
                
                receipt.log_summary(&config.log_directory);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    } else if matches.get_flag("interactive") {
        // Redirect to Interactive Setup Wizard
        println!("🚀 STARTING INTERACTIVE SETUP WIZARD");
        println!("{}", "=".repeat(50));
        
        let mut setup = InteractiveSetup::new();
        if let Err(e) = setup.run_setup().await {
            eprintln!("❌ Setup failed: {}", e);
            return Err(e);
        }
        
        println!("🎉 Interactive setup completed successfully!");
        println!("Your Elite Trading Bot is now running automatically!");
    } else if matches.get_flag("api") {
        // API server mode
        let port: u16 = matches.get_one::<String>("api-port")
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        println!("🚀 STARTING JSON STREAMING API SERVER");
        println!("{}", "=".repeat(50));
        println!("Port: {}", port);
        println!("Base URL: http://localhost:{}", port);
        println!();
        println!("📡 Available endpoints:");
        println!("   GET  /health                    - Health check");
        println!("   POST /api/watch                 - Start watching a JSON file");
        println!("   GET  /api/watch/:file_path     - Stop watching a file");
        println!("   GET  /api/files                - List watched files");
        println!("   GET  /api/content/:file_path   - Get file content");
        println!("   GET  /api/stream/:file_path    - WebSocket stream for real-time updates");
        println!("   POST /api/ollama/process       - Process JSON file with Ollama AI (ULTRA-FAST THREADING - DEFAULT)");
        println!("   POST /api/ollama/process/threaded - Process JSON file with Ollama AI (threaded, non-blocking)");
        println!("   POST /api/ollama/process/ultra-fast - Process JSON file with Ollama AI (maximum speed, direct async)");
        println!("   POST /api/ollama/process/ultra-threaded - Process JSON file with Ollama AI (maximum threading, parallel operations)");
        println!("   POST /api/ollama/conversation - Multi-model AI conversation (models talk to each other)");
        println!("   GET  /api/available-files      - List available JSON files in directory");
        println!();
        println!("💡 Example usage:");
        println!("   curl http://localhost:{}/health", port);
        println!("   curl -X POST http://localhost:{}/api/watch -d '{{\"file_path\":\"/path/to/file.json\"}}'", port);
        println!("   curl http://localhost:{}/api/stream/file.json", port);
        println!("   curl -X POST http://localhost:{}/api/ollama/process -d '{{\"file_path\":\"./sample_data.json\",\"prompt\":\"Analyze this trading data\"}}'", port);
        println!("   curl -X POST http://localhost:{}/api/ollama/process/threaded -d '{{\"file_path\":\"./sample_data.json\",\"prompt\":\"Analyze this trading data\"}}'", port);
        println!("   curl -X POST http://localhost:{}/api/ollama/process/ultra-fast -d '{{\"file_path\":\"./sample_data.json\",\"prompt\":\"Analyze this trading data\"}}'", port);
        println!("   curl -X POST http://localhost:{}/api/ollama/process/ultra-threaded -d '{{\"file_path\":\"./sample_data.json\",\"prompt\":\"Analyze this trading data\"}}'", port);
        println!("   curl -X POST http://localhost:{}/api/ollama/conversation -d '{{\"file_path\":\"./sample_data.json\",\"initial_prompt\":\"Analyze this trading data\",\"models\":[\"phi:latest\",\"qwen2.5:0.5b\"],\"conversation_type\":\"debate\"}}'", port);
        println!("   curl http://localhost:{}/api/available-files", port);
        println!();
        println!("🌐 Starting server...");
        
        match start_api_server(port).await {
            Ok(_) => println!("✅ API server started successfully"),
            Err(e) => {
                eprintln!("❌ Failed to start API server: {}", e);
                return Err(anyhow::anyhow!("API server error: {}", e));
            }
        }
    } else if matches.get_flag("websocket") {
        // WebSocket streaming mode
        println!("🚀 STARTING WEBSOCKET MARKET DATA STREAMING");
        println!("{}", "=".repeat(50));
        
        // Parse stream types
        let stream_types_str = matches.get_one::<String>("stream-types").unwrap();
        let stream_types: Vec<market_data::StreamType> = stream_types_str
            .split(',')
                            .filter_map(|s| {
                match s.trim().to_lowercase().as_str() {
                    "stocks" | "crypto" | "options" | "news" => Some(market_data::StreamType::MarketData),
                    "trade_updates" => Some(market_data::StreamType::TradeUpdates),
                    "account_updates" => Some(market_data::StreamType::AccountUpdates),
                    "order_updates" => Some(market_data::StreamType::OrderUpdates),
                    _ => None,
                }
            })
            .collect();
        
        if stream_types.is_empty() {
            eprintln!("❌ No valid stream types specified. Use: stocks,crypto,options,news");
            return Err(anyhow::anyhow!("Invalid stream types"));
        }
        
        println!("📡 Stream Types: {}", stream_types.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(", "));
        println!("🌐 WebSocket URL: Will be determined based on feed type");
        println!("📁 Data Directory: live_data/");
        println!("");
        println!("💡 This will connect to Alpaca WebSocket streams for real-time data");
        println!("   Make sure ALPACA_API_KEY and ALPACA_SECRET_KEY are set in your environment");
        println!("");
        println!("🚀 Starting WebSocket streaming...");
        
        // Load WebSocket configuration
        let (market_data_config, trading_config, _) = match market_data::load_unified_websocket_config() {
            Ok(configs) => {
                println!("✅ Unified WebSocket configuration loaded successfully");
                configs
            }
            Err(e) => {
                eprintln!("❌ Failed to load unified WebSocket configuration: {}", e);
                return Err(anyhow::anyhow!("Unified WebSocket config error: {}", e));
            }
        };
        
        // Create data directory based on operation mode
        let data_dir = if let Ok(mode) = std::env::var("OPERATION_MODE") {
            match mode.as_str() {
                "paper" => std::path::PathBuf::from("trading_portfolio"),
                "live" => std::path::PathBuf::from("trading_portfolio"),
                _ => std::path::PathBuf::from("trading_portfolio"),
            }
        } else {
            std::path::PathBuf::from("trading_portfolio") // Default to trading portfolio
        };
        std::fs::create_dir_all(&data_dir)?;
        
        // Create and start unified WebSocket streamer
        let is_paper_trading = std::env::var("ALPACA_PAPER_TRADING")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
            
        let streamer = match market_data::UnifiedAlpacaWebSocket::new(
            market_data_config, 
            trading_config, 
            data_dir, 
            stream_types,
            is_paper_trading
        ) {
            Ok(streamer) => {
                println!("✅ Unified WebSocket streamer created successfully");
                streamer
            }
            Err(e) => {
                eprintln!("❌ Failed to create unified WebSocket streamer: {}", e);
                return Err(anyhow::anyhow!("Unified WebSocket streamer error: {}", e));
            }
        };
        
        // Start streaming
        match streamer.start_streaming().await {
            Ok(_) => println!("✅ WebSocket streaming completed successfully"),
            Err(e) => {
                eprintln!("❌ WebSocket streaming failed: {}", e);
                return Err(anyhow::anyhow!("WebSocket streaming error: {}", e));
            }
        }
    } else if matches.get_flag("simulated") {
        // Create data directory based on operation mode
        let data_dir = if let Ok(mode) = std::env::var("OPERATION_MODE") {
            match mode.as_str() {
                "paper" => std::path::PathBuf::from("trading_portfolio"),
                "live" => std::path::PathBuf::from("trading_portfolio"),
                _ => std::path::PathBuf::from("trading_portfolio"),
            }
        } else {
            std::path::PathBuf::from("trading_portfolio") // Default to trading portfolio
        };
        std::fs::create_dir_all(&data_dir)?;
        
        // Normal Mode - REST API calls (Basic plan compatible)
        println!("🎮 NORMAL MODE - REST API Market Data (Basic Plan Compatible)");
        println!("{}", "=".repeat(50));
        println!("📊 Fetching market data via REST API calls...");
        println!("🔄 Update Interval: 30 seconds");
        println!("💱 Symbols: AAPL, SPY, BTC/USD, ETH/USD");
        println!("📈 Data Types: Price, Volume, High/Low, Change %");
        println!("💾 Saving to: {}/ directory", data_dir.to_string_lossy());
        println!("");
        println!("💡 This mode uses REST API calls - perfect for Basic plan accounts");
        println!("   No WebSocket streaming required - just simple HTTP requests");
        println!("");
        
        // Try to start Normal Mode with UnifiedAlpacaWebSocket
        match market_data::unified_websocket::load_unified_websocket_config() {
            Ok((market_config, trading_config, data_dir_str)) => {
                println!("✅ Alpaca configuration loaded successfully");
                println!("🔑 API Key: {}...", &market_config.api_key[..8]);
                println!("🌐 Feed: {}", market_config.feed);
                println!("📁 Data Directory: {}", data_dir_str);
                
                // Create UnifiedAlpacaWebSocket and start Normal Mode
                let websocket = market_data::unified_websocket::UnifiedAlpacaWebSocket::new(
                    market_config,
                    trading_config,
                    std::path::PathBuf::from(&data_dir_str),
                    vec![market_data::unified_websocket::StreamType::MarketData],
                    true, // is_paper_trading
                )?;
                
                println!("✅ UnifiedAlpacaWebSocket created successfully");
                
                // Start Normal Mode (REST API calls)
                if let Err(e) = websocket.start_normal_mode().await {
                    eprintln!("❌ Failed to start Normal Mode: {}", e);
                    return Err(anyhow::anyhow!("Normal Mode error: {}", e));
                }
                println!("✅ Normal Mode started successfully");
            }
            Err(e) => {
                println!("⚠️ Could not load Alpaca configuration: {}", e);
                println!("🔄 Falling back to simulated data...");
                
                // Create and start market stream as fallback
                let mut market_stream = market_data::SimulatedMarketStream::new();
                println!("✅ Market stream created successfully");
                
                // Start the stream
                if let Err(e) = market_stream.start().await {
                    eprintln!("❌ Failed to start market stream: {}", e);
                    return Err(anyhow::anyhow!("Market stream error: {}", e));
                }
                println!("✅ Market stream started successfully");
            }
        }
        
        println!("");
        println!("📊 Monitor the data:");
        println!("   ./scripts.sh watch                    # Interactive stream viewer");
        println!("   cat {}/basic_plan_market_data.json   # All Basic plan market data (15-min historical)", data_dir.to_string_lossy());
        println!("");
        println!("💡 This creates ONE consolidated file with all symbols:");
        println!("   - AAPL, SPY (stocks)");
        println!("   - BTC/USD, ETH/USD (crypto)");
        println!("   - Updates every 30 seconds");
        println!("   - 15-minute historical data limit (Basic plan)");
        println!("");
        println!("🛑 Press Ctrl+C to stop the data collection");
        
        // Keep the main thread alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    } else if matches.get_flag("trading") {
        // Trading Screen - Account Monitoring and Market Hours
        println!("💼 TRADING SCREEN - Account Monitoring & Market Hours");
        println!("{}", "=".repeat(50));
        println!("📊 Real-time account information from Alpaca Trading API");
        println!("🌍 Market hours checking (New York timezone)");
        println!("🔄 Update Interval: 60 seconds");
        println!("💾 Saving to: trading_account.json");
        println!("");
        
        // Try to load Alpaca configuration
        match market_data::unified_websocket::load_unified_websocket_config() {
            Ok((market_config, trading_config, data_dir_str)) => {
                println!("✅ Alpaca configuration loaded successfully");
                println!("🔑 API Key: {}...", &market_config.api_key[..8]);
                println!("🌐 Trading API: {}", if trading_config.base_url.contains("paper") { "Paper Trading" } else { "Live Trading" });
                println!("📁 Data Directory: {}", data_dir_str);
                
                // Create data directory
                let data_dir = std::path::PathBuf::from(&data_dir_str);
                std::fs::create_dir_all(&data_dir)?;
                
                // Create Trading Account Manager
                let account_manager = market_data::TradingAccountManager::new(
                    market_config.api_key,
                    market_config.secret_key,
                    trading_config.base_url.contains("paper"), // is_paper_trading
                    data_dir,
                );
                
                println!("✅ Trading Account Manager created successfully");
                
                // Start account monitoring
                println!("🚀 Starting account monitoring...");
                if let Err(e) = account_manager.start_account_monitoring().await {
                    eprintln!("❌ Failed to start account monitoring: {}", e);
                    return Err(anyhow::anyhow!("Account monitoring error: {}", e));
                }
                println!("✅ Account monitoring started successfully");
            }
            Err(e) => {
                println!("⚠️ Could not load Alpaca configuration: {}", e);
                println!("❌ Trading screen requires valid Alpaca API credentials");
                return Err(anyhow::anyhow!("Trading screen error: {}", e));
            }
        }
        
        println!("");
        println!("📊 Monitor the trading account:");
        println!("   cat trading_account.json              # Real-time account information");
        println!("   ./scripts.sh watch                   # Interactive data viewer");
        println!("");
        println!("💡 Trading screen provides:");
        println!("   - Real-time account balance and positions");
        println!("   - Market hours status (NY timezone)");
        println!("   - Trading permissions and risk management");
        println!("   - Pattern day trader status");
        println!("   - Updates every 60 seconds");
        println!("");
        println!("🛑 Press Ctrl+C to stop account monitoring");
        
        // Keep the main thread alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

    } else if matches.get_flag("enhanced-strategy") {
        // Run Enhanced Trading Strategy with Asset Universe and Positions
        println!("🚀 ENHANCED TRADING STRATEGY WITH ASSET UNIVERSE & POSITIONS");
        println!("==================================================");
        println!("📊 Portfolio Optimization with Current Positions");
        println!("🌍 Asset Universe Analysis");
        println!("⚖️ Rebalancing Recommendations");
        println!("🎯 Enhanced Risk Management");
        println!("💾 Saving enhanced recommendations to: enhanced_strategy_recommendations.json");

        // Load configuration
        let (market_config, trading_config, data_dir) = market_data::load_unified_websocket_config()?;
        
        println!("✅ Alpaca configuration loaded successfully");
        println!("🔑 API Key: {}...", &market_config.api_key[..8]);
        println!("📁 Data Directory: {}", data_dir);

        // Create Asset Universe Manager
        let asset_manager = market_data::AssetUniverseManager::new(
            market_config.api_key.clone(),
            market_config.secret_key.clone(),
            trading_config.base_url.clone(),
        );

        // Get asset universe (try to load from file first, then fetch if needed)
        let asset_universe = match market_data::AssetUniverseManager::load_asset_universe(&data_dir).await {
            Ok(assets) => {
                println!("✅ Loaded {} assets from cache", assets.len());
                assets
            }
            Err(_) => {
                println!("📈 Fetching popular assets from Alpaca...");
                let assets = asset_manager.get_popular_assets().await?;
                asset_manager.save_asset_universe(&assets, &data_dir).await?;
                assets
            }
        };

        // Get current positions
        let current_positions = match market_data::AssetUniverseManager::load_positions(&data_dir).await {
            Ok(positions) => {
                println!("✅ Loaded {} positions from cache", positions.len());
                positions
            }
            Err(_) => {
                println!("📊 Fetching current positions from Alpaca...");
                let positions = asset_manager.get_positions().await?;
                asset_manager.save_positions(&positions, &data_dir).await?;
                positions
            }
        };

        // Load market data
        let market_data = load_market_data(&data_dir).await?;
        println!("✅ Loaded market data for {} symbols", market_data.len());

        // Load account data
        let account_data = load_account_data(&data_dir).await?;
        println!("✅ Loaded account data");

        // Create historical data
        let historical_data = create_historical_data(&market_data).await?;
        println!("✅ Created historical data for analysis");

        // Get portfolio history (optional)
        let portfolio_history = asset_manager.get_portfolio_history().await.ok();

        // Create enhanced strategy data
        let enhanced_data = trading_strategy::EnhancedStrategyData {
            market_data,
            account_data: account_data.clone(),
            historical_data,
            current_positions,
            asset_universe,
            portfolio_history,
        };

        // Create advanced trading strategy with portfolio protection and options
        let strategy = trading_strategy::AdvancedTradingStrategy::with_parameters(
            0.04,   // 4% risk-free rate
            0.12,   // 12% target return
            0.20,   // 20% max volatility
            1.0,    // 100% portfolio protection (never go below starting value)
            0.05,   // 5% profit target
            true,   // Enable options trading
        );
        println!("✅ Enhanced trading strategy with portfolio protection and options initialized");
        println!("🛡️ Portfolio Protection: Never go below starting value of ${:.2}", account_data.starting_portfolio_value);
        println!("🎯 Profit Target: 5% on each position");
        println!("📈 Options Trading: Enabled (max 30% allocation)");

        // Generate enhanced recommendations
        let enhanced_recommendations = strategy.generate_enhanced_recommendations(&enhanced_data)?;

        // Save enhanced recommendations to file
        let enhanced_strategy_file = std::path::PathBuf::from(&data_dir).join("enhanced_strategy_recommendations.json");
        let json_content = serde_json::to_string_pretty(&enhanced_recommendations)?;
        tokio::fs::write(&enhanced_strategy_file, json_content).await?;

        println!("✅ Saved enhanced strategy recommendations to: {}", enhanced_strategy_file.display());

        // Display enhanced summary
        display_enhanced_strategy_summary(&enhanced_recommendations).await?;
    } else if matches.get_flag("market-regime") {
        // Run Market Regime Analysis
        println!("🌍 MARKET REGIME ANALYSIS");
        println!("==================================================");
        println!("📊 Analyzing current market environment");
        println!("🎯 Determining optimal trading strategy");
        println!("📈 Sector rotation analysis");
        println!("💾 Saving regime analysis to: market_regime_analysis.json");

        // Load configuration
        let (market_config, trading_config, data_dir) = market_data::load_unified_websocket_config()?;
        
        println!("✅ Alpaca configuration loaded successfully");
        println!("🔑 API Key: {}...", &market_config.api_key[..8]);
        println!("📁 Data Directory: {}", data_dir);

        // Load asset universe
        let asset_universe = match market_data::AssetUniverseManager::load_asset_universe(&data_dir).await {
            Ok(assets) => {
                println!("✅ Loaded {} assets from cache", assets.len());
                assets
            }
            Err(_) => {
                println!("📈 Fetching popular assets from Alpaca...");
                let asset_manager = market_data::AssetUniverseManager::new(
                    market_config.api_key.clone(),
                    market_config.secret_key.clone(),
                    trading_config.base_url.clone(),
                );
                let assets = asset_manager.get_popular_assets().await?;
                asset_manager.save_asset_universe(&assets, &data_dir).await?;
                assets
            }
        };

        // Load market data
        let market_data = load_market_data(&data_dir).await?;
        println!("✅ Loaded market data for {} symbols", market_data.len());

        // Convert market data to price format for regime analysis
        let mut price_data = HashMap::new();
        for (symbol, data_point) in &market_data {
            price_data.insert(symbol.clone(), data_point.price);
        }

        // Create market regime detector
        let mut regime_detector = market_data::MarketRegimeDetector::new(asset_universe);
        println!("✅ Market regime detector initialized");

        // Analyze market regime
        let regime_analysis = regime_detector.analyze_market_regime(&price_data).await?;
        println!("✅ Market regime analysis complete");

        // Get regime recommendations
        let regime_recommendations = regime_detector.get_regime_recommendations(&regime_analysis)?;
        println!("✅ Regime recommendations generated");

        // Save regime analysis to file
        regime_detector.save_regime_analysis(&regime_analysis, &data_dir).await?;

        // Display regime analysis summary
        display_market_regime_summary(&regime_analysis, &regime_recommendations).await?;
    } else if matches.get_flag("portfolio-analysis") {
        // Portfolio Analysis with JSON Streaming API
        println!("🤖 PORTFOLIO ANALYSIS WITH JSON STREAMING API");
        println!("{}", "=".repeat(50));
        println!("📊 AI-Powered Portfolio Insights via API");
        println!("🎯 Real-time Trading Recommendations");
        println!("📈 Market Analysis and Predictions");
        println!("🔄 Continuous Portfolio Monitoring");

        // Load configuration
        let (market_config, _trading_config, data_dir) = market_data::load_unified_websocket_config()?;
        
        println!("✅ Alpaca configuration loaded successfully");
        println!("🔑 API Key: {}...", &market_config.api_key[..8]);
        println!("📁 Data Directory: {}", data_dir);

        // Load portfolio data
        let portfolio_file = std::path::PathBuf::from(&data_dir).join("trading_portfolio.json");
        
        if !portfolio_file.exists() {
            return Err(anyhow!("Portfolio file not found: {}. Please run --enhanced-strategy first to generate portfolio data.", portfolio_file.display()));
        }

        let portfolio_content = tokio::fs::read_to_string(&portfolio_file).await?;
        println!("✅ Loaded portfolio data from: {}", portfolio_file.display());

        // Start JSON streaming API server for portfolio analysis
        println!("\n🌐 Starting JSON Streaming API Server for Portfolio Analysis");
        println!("Model: {}", config.ollama_model);
        println!("Base URL: {}", config.ollama_base_url);
        println!("API Port: 8082");
        println!("{}", "=".repeat(50));

        // Create a temporary portfolio analysis file for the API to stream
        let analysis_file = std::path::PathBuf::from(&data_dir).join("portfolio_analysis.json");
        let analysis_data = serde_json::json!({
            "portfolio_data": serde_json::from_str::<serde_json::Value>(&portfolio_content)?,
            "analysis_type": "portfolio_insights",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "model": config.ollama_model,
            "status": "ready_for_analysis"
        });
        
        tokio::fs::write(&analysis_file, serde_json::to_string_pretty(&analysis_data)?).await?;
        println!("✅ Created portfolio analysis file: {}", analysis_file.display());

        // Start the API server
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
            
        println!("\n🚀 Starting API server on http://localhost:{}", api_port);
        println!("📊 Portfolio analysis will be available at:");
        println!("   • GET /portfolio-analysis - Get current analysis");
        println!("   • GET /stream/portfolio-analysis - Stream real-time analysis");
        println!("   • GET /files - List all available files");
        println!("   • GET /stream/{{filename}} - Stream any JSON file");
        println!("\n🔄 The API server will continuously analyze the portfolio data");
        println!("Press Ctrl+C to stop the server");
        println!("{}", "=".repeat(50));

        // Start the API server (this will run indefinitely)
        if let Err(e) = start_api_server(api_port).await {
            eprintln!("❌ API server error: {}", e);
            return Err(anyhow!("API server error: {}", e));
        }
    } else if matches.get_flag("protobuf-storage") {
        // Protobuf Storage Test
        println!("🗄️ PROTOBUF STORAGE SYSTEM TEST");
        println!("{}", "=".repeat(50));
        println!("📊 Testing protobuf-based trading data storage");
        println!("🔐 Binary serialization for efficiency");
        println!("📈 Compact and fast data storage");
        println!("⚡ Production-ready data format");

        // Create protobuf storage
        let storage = ProtobufStorage::new("trading_data.pb");
        
        // Create sample data
        let trading_data = ProtobufStorage::create_sample_data();
        
        // Save to file
        println!("\n💾 TEST: Saving Trading Data");
        storage.save_trading_data(&trading_data)?;
        
        // Load from file
        println!("\n📂 TEST: Loading Trading Data");
        let loaded_data = storage.load_trading_data()?;
        println!("✅ Trading data loaded successfully");

        // Display summary
        println!("\n📋 DATA SUMMARY:");
        println!("   • API Keys: {}", loaded_data.api_keys.len());
        println!("   • Assets: {}", loaded_data.assets.len());
        println!("   • Trades: {}", loaded_data.trades.len());
        println!("   • Ollama Receipts: {}", loaded_data.ollama_receipts.len());
        println!("   • Market Regime Analyses: {}", loaded_data.market_regime_analyses.len());
        println!("   • Strategy Recommendations: {}", loaded_data.strategy_recommendations.len());
        println!("   • Execution Signals: {}", loaded_data.execution_signals.len());
        println!("   • Portfolio Snapshots: {}", loaded_data.portfolio_snapshots.len());
        println!("   • Version: {}", loaded_data.version);
        
        if let Some(timestamp) = &loaded_data.last_updated {
            let dt = DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32).unwrap();
            println!("   • Last Updated: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
        }

        println!("\n🎉 PROTOBUF STORAGE TEST COMPLETE!");
        println!("📁 Data saved to: trading_data.pb");
        println!("📊 Binary format for efficiency");
        println!("🔒 Compact and secure storage");
        println!("📈 Production-ready implementation");
        println!("💡 Optimized for high-frequency trading data");

         } else if matches.get_flag("view-protobuf") {
         // View Protobuf Data
         println!("🗄️ VIEWING PROTOBUF STORAGE DATA");
         println!("{}", "=".repeat(50));
         println!("📊 Loading and displaying protobuf data from trading_data.pb");

         let storage = ProtobufStorage::new("trading_data.pb");
         storage.display_detailed_data()?;

     } else if matches.get_flag("export-protobuf") {
         // Export Protobuf Data to JSON
         println!("🗄️ EXPORTING PROTOBUF DATA TO JSON");
         println!("{}", "=".repeat(50));
         println!("📊 Converting protobuf data to JSON format");

         let storage = ProtobufStorage::new("trading_data.pb");
         storage.export_to_json("trading_data_export.json")?;
         
         println!("\n🎉 EXPORT COMPLETE!");
         println!("📁 JSON file created: trading_data_export.json");
         println!("📊 Human-readable format for debugging");
         println!("🔒 Original protobuf file preserved");
         println!("💡 Use any text editor to view the JSON file");

    } else if matches.get_flag("append-test") {
        // Test Appending Data
        println!("🗄️ TESTING PROTOBUF APPEND FUNCTIONALITY");
        println!("{}", "=".repeat(50));
        println!("📊 Testing appending various data types to protobuf storage");

        let storage = ProtobufStorage::new("trading_data.pb");
        
        // Test appending different types of data
        println!("\n🔑 Testing API Key Append:");
        storage.append_api_key("alpaca", "live_key_456", "live")?;
        
        println!("\n📈 Testing Asset Append:");
        storage.append_asset("MSFT", "Microsoft Corporation", "stock", "NASDAQ", 320.50)?;
        storage.append_asset("GOOGL", "Alphabet Inc.", "stock", "NASDAQ", 2800.75)?;
        
        println!("\n💼 Testing Trade Append:");
        storage.append_trade("MSFT", "buy", 5.0, 320.50)?;
        storage.append_trade("GOOGL", "sell", 2.0, 2800.75)?;
        
        println!("\n🤖 Testing Ollama Receipt Append:");
        storage.append_ollama_receipt("llama3.2", "Analyze MSFT", "MSFT shows strong cloud growth", true)?;
        storage.append_ollama_receipt("llama3.2", "Analyze GOOGL", "GOOGL has AI leadership", true)?;
        
        println!("\n📊 Testing Market Regime Append:");
        storage.append_market_regime("bull_market", 0.75)?;
        storage.append_market_regime("sideways_market", 0.60)?;
        
        println!("\n🎯 Testing Strategy Recommendation Append:");
        storage.append_strategy_recommendation("MSFT", "buy", 330.0, 0.85)?;
        storage.append_strategy_recommendation("GOOGL", "hold", 2850.0, 0.70)?;
        
        println!("\n🚦 Testing Execution Signal Append:");
        storage.append_execution_signal("buy", "MSFT", 0.85)?;
        storage.append_execution_signal("sell", "GOOGL", 0.60)?;
        
        println!("\n💰 Testing Portfolio Snapshot Append:");
        storage.append_portfolio_snapshot(105000.0, 45000.0, 105000.0)?;
        storage.append_portfolio_snapshot(107500.0, 42000.0, 107500.0)?;
        
        println!("\n🎉 APPEND TEST COMPLETE!");
        println!("📁 Data appended to: trading_data.pb");
        println!("📊 Multiple records added for each data type");
        println!("💡 Use --view-protobuf to see all data");
        println!("📈 Use --protobuf-stats to see statistics");

    } else if matches.get_flag("protobuf-stats") {
        // Show Protobuf Statistics
        println!("🗄️ PROTOBUF STORAGE STATISTICS");
        println!("{}", "=".repeat(50));
        
        let storage = ProtobufStorage::new("trading_data.pb");
        storage.get_statistics()?;
    } else if matches.get_flag("execute-orders") {
        // Execute Orders Based on Strategy and Liquidation Triggers
        println!("🎯 ORDER EXECUTION SYSTEM");
        println!("{}", "=".repeat(50));
        println!("📊 Reading portfolio analysis, market regime, and strategy recommendations");
        println!("🔄 Executing orders via Alpaca Trading API");
        println!("🛡️ Liquidation triggers: 5% profit target, portfolio protection");
        
        // Load environment variables
        let api_key = std::env::var("ALPACA_API_KEY")
            .map_err(|_| anyhow!("ALPACA_API_KEY not found in environment"))?;
        let secret_key = std::env::var("ALPACA_SECRET_KEY")
            .map_err(|_| anyhow!("ALPACA_SECRET_KEY not found in environment"))?;
        
        // Determine base URL based on operation mode
        let base_url = match std::env::var("OPERATION_MODE").unwrap_or_else(|_| "paper".to_string()).as_str() {
            "paper" => "https://paper-api.alpaca.markets".to_string(),
            "live" => "https://api.alpaca.markets".to_string(),
            _ => "https://paper-api.alpaca.markets".to_string(),
        };
        
        // Load current portfolio value for stop loss calculation
        let data_dir = "trading_portfolio";
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        let portfolio_content = tokio::fs::read_to_string(&portfolio_file).await?;
        let portfolio_data: Value = serde_json::from_str(&portfolio_content)?;
        let starting_portfolio_value = portfolio_data["trading_account"]["account_info"]["portfolio_value"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        // Create order executor
        let order_executor = OrderExecutor::new(
            base_url,
            api_key,
            secret_key,
            5.0, // 5% profit target
            10.0, // 10% stop loss
            starting_portfolio_value,
        );
        
        // Execute orders
        let results = order_executor.execute_orders_from_analysis(data_dir).await?;
        
        // Display results
        println!("\n📊 ORDER EXECUTION RESULTS:");
        println!("{}", "=".repeat(30));
        
        let mut successful_orders = 0;
        let mut failed_orders = 0;
        
        for result in &results {
            if result.success {
                successful_orders += 1;
                println!("✅ Order executed successfully");
                if let Some(order_id) = &result.order_id {
                    println!("   Order ID: {}", order_id);
                }
            } else {
                failed_orders += 1;
                println!("❌ Order execution failed");
                if let Some(error) = &result.error_message {
                    println!("   Error: {}", error);
                }
            }
            println!("   Time: {}", result.execution_time.format("%Y-%m-%d %H:%M:%S UTC"));
            println!();
        }
        
        println!("📈 SUMMARY:");
        println!("   Successful orders: {}", successful_orders);
        println!("   Failed orders: {}", failed_orders);
        println!("   Total orders: {}", results.len());
        
        if successful_orders > 0 {
            println!("\n🎉 Orders executed successfully! Check your Alpaca dashboard for details.");
        } else if failed_orders > 0 {
            println!("\n⚠️ No orders were executed. Check the error messages above.");
        } else {
            println!("\n💤 No orders were generated. Portfolio may already be optimally balanced.");
        }
    } else if matches.get_flag("test-orders") {
        // Test Order Execution System (Demo Mode)
        println!("🧪 ORDER EXECUTION SYSTEM TEST (DEMO MODE)");
        println!("{}", "=".repeat(50));
        println!("📊 Reading portfolio analysis, market regime, and strategy recommendations");
        println!("🔄 Simulating order execution (no actual orders placed)");
        println!("🛡️ Liquidation triggers: 5% profit target, portfolio protection");
        
        let data_dir = "trading_portfolio";
        
        // Load portfolio data
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        let portfolio_content = tokio::fs::read_to_string(&portfolio_file).await?;
        let portfolio_data: Value = serde_json::from_str(&portfolio_content)?;
        let current_portfolio_value = portfolio_data["trading_account"]["account_info"]["portfolio_value"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        println!("\n📊 PORTFOLIO ANALYSIS:");
        println!("   Current Portfolio Value: ${:.2}", current_portfolio_value);
        println!("   Cash Balance: ${}", portfolio_data["trading_account"]["account_info"]["cash"]);
        println!("   Market Status: {}", if portfolio_data["trading_account"]["market_status"]["is_open"].as_bool().unwrap_or(false) { "OPEN" } else { "CLOSED" });
        
        // Load strategy recommendations
        let strategy_file = format!("{}/enhanced_strategy_recommendations.json", data_dir);
        if std::path::Path::new(&strategy_file).exists() {
            let strategy_content = tokio::fs::read_to_string(&strategy_file).await?;
            let strategy_data: Value = serde_json::from_str(&strategy_content)?;
            
            if let Some(recommendations) = strategy_data["recommendations"].as_array() {
                println!("\n🎯 STRATEGY RECOMMENDATIONS:");
                for (i, rec) in recommendations.iter().enumerate() {
                    if let (Some(symbol), Some(action)) = (
                        rec["symbol"].as_str(),
                        rec["action"].as_str()
                    ) {
                        let confidence = rec["confidence_score"].as_f64().unwrap_or(0.0);
                        let reason = rec["reason"].as_str().unwrap_or("No reason provided");
                        
                        println!("   {}. {} - {} (confidence: {:.2})", i + 1, symbol, action, confidence);
                        
                        match action {
                            "buy" | "sell" => {
                                if confidence >= 0.7 {
                                    println!("      ✅ Would execute: {} {} shares", action, symbol);
                                } else {
                                    println!("      ⚠️ Confidence too low: {:.2}", confidence);
                                }
                            },
                            "SKIP" => {
                                println!("      ⏭️ Skipped: {}", reason);
                            },
                            _ => {
                                println!("      ℹ️ Action: {} - {}", action, reason);
                            }
                        }
                    }
                }
            }
        }
        
        // Load market regime analysis
        let regime_file = format!("{}/market_regime_analysis.json", data_dir);
        if std::path::Path::new(&regime_file).exists() {
            let regime_content = tokio::fs::read_to_string(&regime_file).await?;
            let regime_data: Value = serde_json::from_str(&regime_content)?;
            
            println!("\n🌍 MARKET REGIME ANALYSIS:");
            println!("   Current Regime: {}", regime_data["market_regime"].as_str().unwrap_or("unknown"));
            println!("   Confidence: {:.2}", regime_data["confidence_level"].as_f64().unwrap_or(0.0));
            println!("   Volatility: {}", regime_data["volatility_regime"].as_str().unwrap_or("unknown"));
        }
        
        // Simulate liquidation analysis
        println!("\n🛡️ LIQUIDATION ANALYSIS:");
        let starting_value = 100000.0; // Default starting value
        let profit_target = 5.0; // 5%
        let stop_loss = 10.0; // 10%
        
        if current_portfolio_value < starting_value {
            let loss_percentage = ((starting_value - current_portfolio_value) / starting_value) * 100.0;
            if loss_percentage >= stop_loss {
                println!("   🚨 PORTFOLIO STOP LOSS TRIGGERED: {:.2}% loss", loss_percentage);
                println!("   📤 Would liquidate all positions for portfolio protection");
            } else {
                println!("   ⚠️ Portfolio down {:.2}% but within stop loss threshold", loss_percentage);
            }
        } else {
            let gain_percentage = ((current_portfolio_value - starting_value) / starting_value) * 100.0;
            println!("   📈 Portfolio up {:.2}% from starting value", gain_percentage);
        }
        
        // Load current positions for profit target analysis
        if let Some(positions) = portfolio_data["current_positions"].as_array() {
            println!("\n📊 POSITION ANALYSIS:");
            for position in positions {
                if let (Some(symbol), Some(qty), Some(market_value), Some(avg_cost)) = (
                    position["symbol"].as_str(),
                    position["qty"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    position["market_value"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    position["avg_entry_price"].as_str().and_then(|s| s.parse::<f64>().ok())
                ) {
                    if qty > 0.0 {
                        let current_price = market_value / qty;
                        let profit_percentage = ((current_price - avg_cost) / avg_cost) * 100.0;
                        
                        if profit_percentage >= profit_target {
                            println!("   🎯 {}: {:.2}% profit - WOULD LIQUIDATE", symbol, profit_percentage);
                        } else if profit_percentage < -stop_loss {
                            println!("   🛑 {}: {:.2}% loss - WOULD STOP LOSS", symbol, profit_percentage);
                        } else {
                            println!("   📊 {}: {:.2}% P&L - HOLD", symbol, profit_percentage);
                        }
                    }
                }
            }
        }
        
        println!("\n🎉 DEMO COMPLETE!");
        println!("💡 To execute real orders, set ALPACA_API_KEY and ALPACA_SECRET_KEY in config.env");
        println!("🔧 Then run: ./target/release/trading_bot.exe --execute-orders");
    } else if matches.get_flag("enhanced-decisions") {
        // Enhanced AI Decision Engine Test
        println!("🧠 ENHANCED AI DECISION ENGINE");
        println!("{}", "=".repeat(50));
        println!("📊 Sophisticated long/short decision making");
        println!("🎯 Market regime analysis and technical indicators");
        println!("⚖️ Kelly Criterion position sizing");
        
        use trading_strategy::{EnhancedDecisionEngine, TradingAction};
        
        // Load market data
        let data_dir = "trading_portfolio";
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        let portfolio_content = tokio::fs::read_to_string(&portfolio_file).await?;
        let portfolio_data: Value = serde_json::from_str(&portfolio_content)?;
        
        // Create sample market data for demonstration
        let mut market_data = std::collections::HashMap::new();
        market_data.insert("AAPL".to_string(), trading_strategy::MarketDataPoint {
            symbol: "AAPL".to_string(),
            price: 150.0,
            open: 148.0,
            high: 152.0,
            low: 147.0,
            volume: 50000000.0,
            timestamp: Utc::now(),
        });
        market_data.insert("SPY".to_string(), trading_strategy::MarketDataPoint {
            symbol: "SPY".to_string(),
            price: 450.0,
            open: 448.0,
            high: 451.0,
            low: 447.0,
            volume: 80000000.0,
            timestamp: Utc::now(),
        });
        market_data.insert("BTC/USD".to_string(), trading_strategy::MarketDataPoint {
            symbol: "BTC/USD".to_string(),
            price: 45000.0,
            open: 44000.0,
            high: 46000.0,
            low: 43500.0,
            volume: 1000000.0,
            timestamp: Utc::now(),
        });
        
        // Create account data
        let account_data = trading_strategy::AccountData {
            cash: 100000.0,
            equity: 100000.0,
            buying_power: 100000.0,
            portfolio_value: 100000.0,
            daytrade_count: 0,
            pattern_day_trader: false,
            shorting_enabled: true,
            margin_multiplier: 2.0,
            starting_portfolio_value: 100000.0,
        };
        
        // Create enhanced decision engine
        let mut decision_engine = EnhancedDecisionEngine::new(0.05); // 5% risk-free rate
        
        // Analyze market regime
        decision_engine.analyze_market_regime(&market_data)?;
        
        println!("\n🌍 MARKET REGIME ANALYSIS:");
        println!("   Current Regime: {}", decision_engine.market_regime);
        println!("   Volatility Regime: {}", decision_engine.volatility_regime);
        println!("   Regime Confidence: {:.2}", decision_engine.regime_confidence);
        
        // Generate enhanced decisions
        let decisions = decision_engine.generate_enhanced_decisions(
            &market_data,
            &account_data,
            &[], // No current positions for demo
        )?;
        
        println!("\n🎯 ENHANCED TRADING DECISIONS:");
        println!("{}", "=".repeat(40));
        
        for (i, decision) in decisions.iter().enumerate() {
            println!("\n{}. {} - {}", i + 1, decision.symbol, decision.action);
            println!("   Expected Return: {:.2}%", decision.expected_return * 100.0);
            println!("   Confidence: {:.2}", decision.confidence_score);
            println!("   Position Size: ${:.2}", decision.position_size.abs());
            println!("   Reasoning: {}", decision.reasoning);
            println!("   Stop Loss: ${:.2}", decision.stop_loss);
            println!("   Take Profit: ${:.2}", decision.take_profit);
            
            // Show action interpretation
            match decision.action {
                TradingAction::OpenLong => println!("   📈 Action: BUY {} shares (Long Position)", 
                    (decision.position_size / 150.0) as i32),
                TradingAction::OpenShort => println!("   📉 Action: SELL {} shares (Short Position)", 
                    (decision.position_size.abs() / 150.0) as i32),
                TradingAction::CloseLong => println!("   🔄 Action: SELL existing long position"),
                TradingAction::CloseShort => println!("   🔄 Action: BUY to cover short position"),
                TradingAction::Hold => println!("   ⏸️ Action: HOLD - No position change"),
            }
        }
        
        println!("\n🧠 DECISION ENGINE FEATURES:");
        println!("   ✅ Market Regime Analysis (Bull/Bear/Sideways/Volatility)");
        println!("   ✅ Technical Indicators (Momentum, Volatility, Volume)");
        println!("   ✅ Long/Short Position Capabilities");
        println!("   ✅ Kelly Criterion Position Sizing");
        println!("   ✅ Risk-Adjusted Expected Returns");
        println!("   ✅ Stop Loss & Take Profit Calculations");
        println!("   ✅ Mean Reversion & Momentum Strategies");
        println!("   ✅ Contrarian & Trend Following Logic");
        
        println!("\n🎉 Enhanced decision engine analysis complete!");
        println!("💡 This system can now make sophisticated long/short decisions");
        println!("🔧 Integrate with --execute-orders for live trading");
    } else if matches.get_flag("ai-decisions") {
        // AI-Enhanced Decision Engine Test
        println!("🤖 AI-ENHANCED DECISION ENGINE");
        println!("{}", "=".repeat(50));
        println!("🧠 Combining Mathematical Analysis with AI Insights");
        println!("📊 Real-time AI-powered trading recommendations");
        println!("🎯 Enhanced confidence scoring and risk assessment");
        
        use trading_strategy::{AIDecisionEngine, TradingAction};
        
        // Load portfolio data
        let data_dir = "trading_portfolio";
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        let portfolio_content = tokio::fs::read_to_string(&portfolio_file).await?;
        let portfolio_data: Value = serde_json::from_str(&portfolio_content)?;
        
        // Create sample market data for demonstration
        let mut market_data = std::collections::HashMap::new();
        market_data.insert("AAPL".to_string(), trading_strategy::MarketDataPoint {
            symbol: "AAPL".to_string(),
            price: 150.0,
            open: 148.0,
            high: 152.0,
            low: 147.0,
            volume: 50000000.0,
            timestamp: Utc::now(),
        });
        market_data.insert("SPY".to_string(), trading_strategy::MarketDataPoint {
            symbol: "SPY".to_string(),
            price: 450.0,
            open: 448.0,
            high: 451.0,
            low: 447.0,
            volume: 80000000.0,
            timestamp: Utc::now(),
        });
        market_data.insert("BTC/USD".to_string(), trading_strategy::MarketDataPoint {
            symbol: "BTC/USD".to_string(),
            price: 45000.0,
            open: 44000.0,
            high: 46000.0,
            low: 43500.0,
            volume: 1000000.0,
            timestamp: Utc::now(),
        });
        
        // Create account data
        let account_data = trading_strategy::AccountData {
            cash: 100000.0,
            equity: 100000.0,
            buying_power: 100000.0,
            portfolio_value: 100000.0,
            daytrade_count: 0,
            pattern_day_trader: false,
            shorting_enabled: true,
            margin_multiplier: 2.0,
            starting_portfolio_value: 100000.0,
        };
        
        // Create AI decision engine
        let mut ai_decision_engine = AIDecisionEngine::new(
            ollama_client.clone(),
            config.ollama_model.clone(),
            0.05, // 5% risk-free rate
        );
        
        println!("\n🧠 GENERATING AI-ENHANCED DECISIONS...");
        println!("Model: {}", config.ollama_model);
        println!("Processing mathematical analysis + AI insights...");
        
        // Generate AI-enhanced decisions
        let ai_decisions = ai_decision_engine.generate_ai_enhanced_decisions(
            &market_data,
            &account_data,
            &[], // No current positions for demo
            &portfolio_data,
        ).await?;
        
        println!("\n🎯 AI-ENHANCED TRADING DECISIONS:");
        println!("{}", "=".repeat(50));
        
        for (i, decision) in ai_decisions.iter().enumerate() {
            println!("\n{}. {} - {}", i + 1, decision.symbol, decision.action);
            println!("   📊 Mathematical Confidence: {:.2}", decision.mathematical_confidence);
            println!("   🤖 AI Confidence Boost: {:.2}", decision.ai_confidence_boost);
            println!("   🎯 Combined Confidence: {:.2}", decision.combined_confidence);
            println!("   💰 Position Size: ${:.2}", decision.position_size.abs());
            println!("   📈 Expected Return: {:.2}%", decision.expected_return * 100.0);
            println!("   ⚠️ AI Risk Assessment: {:.2}", decision.ai_risk_assessment);
            println!("   🧠 Mathematical Reasoning: {}", decision.mathematical_reasoning);
            println!("   🤖 AI Reasoning: {}", decision.ai_reasoning);
            println!("   🛡️ Stop Loss: ${:.2}", decision.stop_loss);
            println!("   🎯 Take Profit: ${:.2}", decision.take_profit);
            
            // Show action interpretation
            match decision.action {
                TradingAction::OpenLong => println!("   📈 Action: BUY {} shares (Long Position)", 
                    (decision.position_size / 150.0) as i32),
                TradingAction::OpenShort => println!("   📉 Action: SELL {} shares (Short Position)", 
                    (decision.position_size.abs() / 150.0) as i32),
                TradingAction::CloseLong => println!("   🔄 Action: SELL existing long position"),
                TradingAction::CloseShort => println!("   🔄 Action: BUY to cover short position"),
                TradingAction::Hold => println!("   ⏸️ Action: HOLD - No position change"),
            }
        }
        
        // Generate comprehensive AI analysis report
        println!("\n📊 GENERATING COMPREHENSIVE AI ANALYSIS REPORT...");
        let ai_report = ai_decision_engine.generate_ai_analysis_report(
            &market_data,
            &account_data,
            &[], // No current positions for demo
            &portfolio_data,
        ).await?;
        
        // Save AI analysis report
        let report_file = format!("{}/ai_analysis_report.json", data_dir);
        tokio::fs::write(&report_file, serde_json::to_string_pretty(&ai_report)?).await?;
        println!("✅ Saved AI analysis report to: {}", report_file);
        
        println!("\n🧠 AI-ENHANCED DECISION ENGINE FEATURES:");
        println!("   ✅ Mathematical Analysis (Modern Portfolio Theory, Kelly Criterion)");
        println!("   ✅ AI-Powered Market Insights and Sentiment Analysis");
        println!("   ✅ Combined Confidence Scoring (Math + AI)");
        println!("   ✅ AI Risk Assessment and Opportunity Identification");
        println!("   ✅ Enhanced Reasoning (Mathematical + AI Explanations)");
        println!("   ✅ Real-time Market Regime Analysis");
        println!("   ✅ Long/Short Position Capabilities");
        println!("   ✅ Dynamic Position Sizing Based on AI Risk Assessment");
        
        println!("\n🎉 AI-enhanced decision engine analysis complete!");
        println!("💡 This system combines the best of mathematical analysis and AI insights");
        println!("🔧 Integrate with --execute-orders for live AI-powered trading");
    // Duplicate interactive setup removed - handled above

    } else {
        println!("Trading Bot started. Use --help for usage information.");
        println!("Available modes:");
        println!("  -i, --interactive     Interactive streaming chat mode (default)");
        println!("  -p, --prompt \"text\"   Single prompt with streaming response (default)");
        println!("  -s, --stream \"text\"   Real-time streaming response mode (same as -p)");
        println!("  -t, --test \"text\"     Performance test with streaming and detailed timing");

        println!("  -l, --logs            View receipt logs summary");
        println!("  -m, --model \"name\"    Override auto-detection with specific model");
        println!("  --api                 Start JSON streaming API server");
        println!("  --api-port PORT       Custom port for API server (default: 8080)");
        println!("  --websocket           Start WebSocket-based market data streaming");
        println!("  --stream-types TYPES  Comma-separated stream types (default: stocks,crypto,options,news)");
        println!("  --simulated           Start Normal Mode with REST API market data (Basic plan compatible)");
        println!("  --trading             Start trading screen with account monitoring and market hours");
        println!("  --enhanced-strategy   Run Enhanced Strategy with Asset Universe & Positions");
        println!("  --portfolio-analysis  Analyze Portfolio with Ollama AI");
        println!("  --market-regime       Run Market Regime Analysis using Asset Universe");
        println!("  -i, --interactive     Start Interactive Setup Wizard for Automatic Trading");
        println!("  --protobuf-storage    Test protobuf-based trading data storage");
        println!("  --view-protobuf       View stored protobuf data in readable format");
        println!("  --export-protobuf     Export protobuf data to JSON format");
        println!("  --append-test         Test appending various data types to protobuf storage");
        println!("  --protobuf-stats      Show statistics about stored protobuf data");
        println!();
        println!("💡 Streaming is now the default for all modes for enhanced responsiveness!");
        println!("🤖 Model auto-detection is enabled by default for optimal performance!");
        println!();
        println!("⚡ Performance Tips:");
        println!("   • For fastest responses (3-5s), try models: phi, qwen2.5:0.5b, gemma2:2b");
        println!("   • Current model optimization: Reduced tokens, faster sampling");
        println!("   • Connection pooling and TCP keep-alive enabled for better throughput");
        println!();
        println!("🌐 API Server Features:");
        println!("   • Real-time JSON file streaming");
        println!("   • WebSocket support for live updates");
        println!("   • File change monitoring");
        println!("   • RESTful API endpoints");
        println!("   • Ollama AI integration for JSON analysis");
        println!();
        println!("📡 WebSocket Streaming Features:");
        println!("   • Real-time Alpaca market data via WebSocket");
        println!("   • Support for Stocks, Crypto, Options, and News streams");
        println!("   • Automatic reconnection and error handling");
        println!("   • Live data persistence to JSON files");
        println!("   • Ultra-threading architecture for concurrent streams");
        println!("   • Unified streaming for market data, trade updates, and order updates");
        println!("   • Sub-100ms latency for real-time trading decisions");
    }

    Ok(())
}

/// Run Normal Mode with REST API market data
#[allow(dead_code)]
async fn run_normal_mode() -> Result<()> {
    println!("🎮 NORMAL MODE - REST API Market Data (Basic Plan Compatible)");
    println!("==================================================");
    println!("📊 Fetching market data via REST API calls...");
    println!("🔄 Update Interval: 30 seconds");
    println!("💱 Symbols: AAPL, SPY, BTC/USD, ETH/USD");
    println!("📈 Data Types: Price, Volume, High/Low, Change %");
    println!("💾 Saving to: sandbox_data/ directory");
    println!("");
    println!("💡 This mode uses REST API calls - perfect for Basic plan accounts");
    println!("   No WebSocket streaming required - just simple HTTP requests");

    // Load configuration
    let (market_config, trading_config, data_dir) = market_data::load_unified_websocket_config()?;
    
    println!("✅ Alpaca configuration loaded successfully");
    println!("🔑 API Key: {}...", &market_config.api_key[..8]);
    println!("🌐 Feed: {}", market_config.feed);
    println!("📁 Data Directory: {}", data_dir);

    // Create unified WebSocket instance
    let websocket = market_data::UnifiedAlpacaWebSocket::new(
        market_config,
        trading_config,
        std::path::PathBuf::from(&data_dir),
        vec![], // Empty stream types for Normal Mode
        data_dir.contains("sandbox"),
    )?;

    println!("✅ UnifiedAlpacaWebSocket created successfully");

    // Start Normal Mode
    if let Err(e) = websocket.start_normal_mode().await {
        error!("❌ Failed to start Normal Mode: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Run Trading Screen for account monitoring
#[allow(dead_code)]
async fn run_trading_screen() -> Result<()> {
    println!("💼 TRADING SCREEN - Account Monitoring & Market Hours");
    println!("==================================================");
    println!("📊 Real-time account information from Alpaca Trading API");
    println!("🌍 Market hours checking (New York timezone)");
    println!("🔄 Update Interval: 60 seconds");
    println!("💾 Saving to: trading_account.json");

    // Load configuration
    let (market_config, _trading_config, data_dir) = market_data::load_unified_websocket_config()?;
    
    println!("✅ Alpaca configuration loaded successfully");
    println!("🔑 API Key: {}...", &market_config.api_key[..8]);
    println!("🌐 Trading API: {}", if data_dir.contains("sandbox") { "Paper Trading" } else { "Live Trading" });
    println!("📁 Data Directory: {}", data_dir);

    // Create trading account manager
            let account_manager = market_data::TradingAccountManager::new(
            market_config.api_key,
            market_config.secret_key,
            data_dir.contains("sandbox"),
            std::path::PathBuf::from(&data_dir),
        );

    println!("✅ Trading Account Manager created successfully");
    println!("🚀 Starting account monitoring...");

    // Start account monitoring
    if let Err(e) = account_manager.start_account_monitoring().await {
        error!("❌ Failed to start account monitoring: {}", e);
        return Err(e);
    }

    Ok(())
}



/// Load market data from consolidated JSON file
async fn load_market_data(data_dir: &str) -> Result<HashMap<String, MarketDataPoint>> {
    let portfolio_file = std::path::PathBuf::from(data_dir).join("trading_portfolio.json");
    
    if !portfolio_file.exists() {
        return Err(anyhow!("Portfolio file not found: {}", portfolio_file.display()));
    }

    let content = tokio::fs::read_to_string(&portfolio_file).await?;
    let data: Value = serde_json::from_str(&content)?;

    let mut market_data = HashMap::new();

    if let Some(symbols) = data["market_data"]["symbols"].as_object() {
        for (symbol, symbol_data) in symbols {
            let market_point = MarketDataPoint {
                symbol: symbol.clone(),
                price: symbol_data["price"].as_f64().unwrap_or(0.0),
                volume: symbol_data["volume"].as_f64().unwrap_or(0.0),
                high: symbol_data["high"].as_f64().unwrap_or(0.0),
                low: symbol_data["low"].as_f64().unwrap_or(0.0),
                open: symbol_data["open"].as_f64().unwrap_or(0.0),
                timestamp: Utc::now(), // Use current time as approximation
            };
            market_data.insert(symbol.clone(), market_point);
        }
    }

    Ok(market_data)
}

/// Load account data from consolidated JSON file
async fn load_account_data(data_dir: &str) -> Result<AccountData> {
    let portfolio_file = std::path::PathBuf::from(data_dir).join("trading_portfolio.json");
    
    if !portfolio_file.exists() {
        return Err(anyhow!("Portfolio file not found: {}", portfolio_file.display()));
    }

    let content = tokio::fs::read_to_string(&portfolio_file).await?;
    let data: Value = serde_json::from_str(&content)?;

    let account_info = &data["trading_account"]["account_info"];
    
            let portfolio_value = account_info["portfolio_value"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let account_data = AccountData {
            cash: account_info["cash"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            equity: account_info["equity"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            buying_power: account_info["buying_power"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            portfolio_value,
            daytrade_count: account_info["daytrade_count"].as_i64().unwrap_or(0) as i32,
            pattern_day_trader: account_info["pattern_day_trader"].as_bool().unwrap_or(false),
            shorting_enabled: account_info["shorting_enabled"].as_bool().unwrap_or(false),
            margin_multiplier: account_info["multiplier"].as_str().unwrap_or("1").parse().unwrap_or(1.0),
            starting_portfolio_value: portfolio_value, // Use current value as starting value for protection
        };

    Ok(account_data)
}

/// Create historical data for analysis (simplified)
async fn create_historical_data(market_data: &HashMap<String, MarketDataPoint>) -> Result<Vec<MarketDataPoint>> {
    let mut historical_data = Vec::new();
    
    // Create some historical data points (in practice would load from database)
    for (symbol, current_data) in market_data {
        // Create 30 days of historical data with some variation
        for i in 0..30 {
            let price_variation = 1.0 + (i as f64 * 0.01) * if i % 2 == 0 { 1.0 } else { -1.0 };
            let historical_point = MarketDataPoint {
                symbol: symbol.clone(),
                price: current_data.price * price_variation,
                volume: current_data.volume * (0.8 + (i as f64 * 0.01)),
                high: current_data.high * price_variation,
                low: current_data.low * price_variation,
                open: current_data.open * price_variation,
                timestamp: Utc::now() - chrono::Duration::days(30 - i),
            };
            historical_data.push(historical_point);
        }
    }

    Ok(historical_data)
}



/// Display enhanced strategy summary with positions and asset universe
async fn display_enhanced_strategy_summary(recommendations: &Value) -> Result<()> {
    println!("\n🚀 ENHANCED STRATEGY SUMMARY");
    println!("{}", "=".repeat(50));
    
    // Display asset universe summary
    if let Some(asset_universe_summary) = recommendations.get("asset_universe_summary") {
        println!("🌍 ASSET UNIVERSE:");
        if let Some(total_assets) = asset_universe_summary.get("total_assets").and_then(|v| v.as_u64()) {
            println!("   📊 Total Assets: {}", total_assets);
        }
        if let Some(tradable) = asset_universe_summary.get("tradable_assets").and_then(|v| v.as_u64()) {
            println!("   ✅ Tradable Assets: {}", tradable);
        }
        if let Some(marginable) = asset_universe_summary.get("marginable_assets").and_then(|v| v.as_u64()) {
            println!("   💰 Marginable Assets: {}", marginable);
        }
        if let Some(shortable) = asset_universe_summary.get("shortable_assets").and_then(|v| v.as_u64()) {
            println!("   📉 Shortable Assets: {}", shortable);
        }
    }
    
    // Display current positions
    if let Some(position_analysis) = recommendations.get("position_analysis") {
        if let Some(positions) = position_analysis.as_object() {
            if !positions.is_empty() {
                println!("\n📊 CURRENT POSITIONS:");
                for (symbol, value) in positions {
                    if let Some(market_value) = value.as_f64() {
                        println!("   {}: ${:.2}", symbol, market_value);
                    }
                }
            } else {
                println!("\n📊 CURRENT POSITIONS: None");
            }
        }
    }
    
    // Display portfolio protection and profit targets
    if let Some(portfolio_protection) = recommendations.get("portfolio_protection") {
        println!("\n🛡️ PORTFOLIO PROTECTION:");
        if let Some(protection_level) = portfolio_protection.get("protection_level").and_then(|v| v.as_f64()) {
            println!("   🛡️ Protection Level: ${:.2}", protection_level);
        }
        if let Some(current_value) = portfolio_protection.get("current_value").and_then(|v| v.as_f64()) {
            println!("   💰 Current Value: ${:.2}", current_value);
        }
        if let Some(triggered) = portfolio_protection.get("protection_triggered").and_then(|v| v.as_bool()) {
            println!("   ⚠️ Protection Triggered: {}", if triggered { "YES" } else { "NO" });
        }
        if let Some(reduction_factor) = portfolio_protection.get("risk_reduction_factor").and_then(|v| v.as_f64()) {
            println!("   📉 Risk Reduction Factor: {:.1}%", reduction_factor * 100.0);
        }
    }

    if let Some(profit_targets) = recommendations.get("profit_targets") {
        println!("\n🎯 PROFIT TARGETS:");
        if let Some(target_percentage) = profit_targets.get("target_percentage").and_then(|v| v.as_f64()) {
            println!("   🎯 Target Percentage: {:.1}%", target_percentage);
        }
        if let Some(take_profit_levels) = profit_targets.get("take_profit_levels").and_then(|v| v.as_array()) {
            println!("   📊 Take Profit Levels: {} positions", take_profit_levels.len());
        }
    }

    if let Some(options_strategy) = recommendations.get("options_strategy") {
        println!("\n📈 OPTIONS STRATEGY:");
        if let Some(enabled) = options_strategy.get("enabled").and_then(|v| v.as_bool()) {
            println!("   📈 Options Trading: {}", if enabled { "ENABLED" } else { "DISABLED" });
        }
        if let Some(max_allocation) = options_strategy.get("max_allocation").and_then(|v| v.as_f64()) {
            println!("   📊 Max Options Allocation: {:.1}%", max_allocation);
        }
        if let Some(options_positions) = options_strategy.get("options_positions").and_then(|v| v.as_array()) {
            println!("   🎯 Options Positions: {} recommended", options_positions.len());
        }
    }

    // Display portfolio summary
    if let Some(portfolio_summary) = recommendations.get("portfolio_summary") {
        println!("\n💰 PORTFOLIO SUMMARY:");
        if let Some(total_allocation) = portfolio_summary.get("total_allocation").and_then(|v| v.as_f64()) {
            println!("   💰 Total Allocation: {:.1}%", total_allocation);
        }
        
        if let Some(expected_return) = portfolio_summary.get("expected_portfolio_return").and_then(|v| v.as_f64()) {
            println!("   📈 Expected Portfolio Return: {:.2}%", expected_return * 100.0);
        }
        
        if let Some(volatility) = portfolio_summary.get("portfolio_volatility").and_then(|v| v.as_f64()) {
            println!("   📊 Portfolio Volatility: {:.2}%", volatility * 100.0);
        }
        
        if let Some(sharpe_ratio) = portfolio_summary.get("portfolio_sharpe_ratio").and_then(|v| v.as_f64()) {
            println!("   🎯 Portfolio Sharpe Ratio: {:.2}", sharpe_ratio);
        }
        
        if let Some(var_95) = portfolio_summary.get("value_at_risk_95").and_then(|v| v.as_f64()) {
            println!("   🛡️ Value at Risk (95%): {:.2}%", var_95 * 100.0);
        }
        
        if let Some(max_drawdown) = portfolio_summary.get("max_drawdown").and_then(|v| v.as_f64()) {
            println!("   📉 Max Drawdown: {:.2}%", max_drawdown * 100.0);
        }
        
        if let Some(risk_level) = portfolio_summary.get("risk_level").and_then(|v| v.as_str()) {
            println!("   ⚠️ Risk Level: {}", risk_level);
        }
        
        if let Some(positions_count) = portfolio_summary.get("current_positions_count").and_then(|v| v.as_u64()) {
            println!("   📊 Current Positions: {}", positions_count);
        }
        
        if let Some(universe_size) = portfolio_summary.get("asset_universe_size").and_then(|v| v.as_u64()) {
            println!("   🌍 Asset Universe Size: {}", universe_size);
        }

        if let Some(options_allocation) = portfolio_summary.get("options_allocation_percentage").and_then(|v| v.as_f64()) {
            println!("   📈 Options Allocation: {:.1}%", options_allocation);
        }

        if let Some(protection_active) = portfolio_summary.get("portfolio_protection_active").and_then(|v| v.as_bool()) {
            println!("   🛡️ Portfolio Protection: {}", if protection_active { "ACTIVE" } else { "INACTIVE" });
        }
    }
    
    // Display recommendations
    if let Some(recommendations_array) = recommendations.get("recommendations").and_then(|v| v.as_array()) {
        println!("\n🎯 ENHANCED TRADING RECOMMENDATIONS");
        println!("{}", "=".repeat(50));
        
        if recommendations_array.is_empty() {
            println!("✅ No new recommendations - portfolio is optimally balanced!");
        } else {
            println!("📋 Found {} recommendations for portfolio optimization:", recommendations_array.len());
            
            for (i, rec) in recommendations_array.iter().take(5).enumerate() {
                if let (Some(symbol), Some(action), Some(difference)) = (
                    rec.get("symbol").and_then(|v| v.as_str()),
                    rec.get("action").and_then(|v| v.as_str()),
                    rec.get("difference").and_then(|v| v.as_f64())
                ) {
                    println!("   {}. {} {}: ${:.2}", i + 1, action, symbol, difference);
                }
            }
            
            if recommendations_array.len() > 5 {
                println!("   ... and {} more recommendations", recommendations_array.len() - 5);
            }
        }
        
        println!("✅ Enhanced strategy analysis complete! Check enhanced_strategy_recommendations.json for full details.");
    }
    
    Ok(())
}

/// Display market regime analysis summary
async fn display_market_regime_summary(
    analysis: &market_data::MarketRegimeAnalysis,
    recommendations: &Value,
) -> Result<()> {
    println!("\n🌍 MARKET REGIME ANALYSIS SUMMARY");
    println!("{}", "=".repeat(50));
    
    // Display current regime
    println!("🎯 CURRENT MARKET REGIME: {:?}", analysis.current_regime);
    println!("📊 Confidence Score: {:.1}%", analysis.confidence_score * 100.0);
    println!("⏱️ Regime Duration: {} days", analysis.regime_duration);
    println!("📈 Regime Probability: {:.1}%", analysis.regime_probability * 100.0);
    
    // Display volatility regime
    println!("\n📊 VOLATILITY REGIME:");
    println!("   Type: {}", analysis.volatility_regime.regime_type);
    println!("   VIX Equivalent: {:.1}", analysis.volatility_regime.vix_equivalent);
    println!("   Volatility Percentile: {:.1}%", analysis.volatility_regime.volatility_percentile * 100.0);
    println!("   Trend: {}", analysis.volatility_regime.volatility_trend);
    
    // Display correlation regime
    println!("\n🔗 CORRELATION REGIME:");
    println!("   Average Correlation: {:.2}", analysis.correlation_regime.average_correlation);
    println!("   Trend: {}", analysis.correlation_regime.correlation_trend);
    println!("   Diversification Benefit: {:.1}%", analysis.correlation_regime.diversification_benefit * 100.0);
    
    // Display sector analysis
    println!("\n📈 SECTOR ANALYSIS:");
    println!("   Leading Sectors: {}", analysis.sector_analysis.leading_sectors.join(", "));
    println!("   Lagging Sectors: {}", analysis.sector_analysis.lagging_sectors.join(", "));
    println!("   Sector Rotation: {}", analysis.sector_analysis.sector_rotation);
    
    // Display key indicators
    println!("\n📊 KEY INDICATORS:");
    println!("   SP500 Trend: {:.2}%", analysis.regime_indicators.sp500_trend * 100.0);
    println!("   VIX Level: {:.1}", analysis.regime_indicators.vix_level);
    println!("   Treasury Yield: {:.2}%", analysis.regime_indicators.treasury_yield * 100.0);
    println!("   Market Breadth: {:.1}%", analysis.regime_indicators.market_breadth * 100.0);
    println!("   Momentum Score: {:.2}", analysis.regime_indicators.momentum_score);
    println!("   Volatility Score: {:.2}", analysis.regime_indicators.volatility_score);
    
    // Display regime recommendations
    if let Some(strategy) = recommendations.get("strategy").and_then(|v| v.as_str()) {
        println!("\n🎯 REGIME RECOMMENDATIONS:");
        println!("   Strategy: {}", strategy);
        
        if let Some(asset_allocation) = recommendations.get("asset_allocation") {
            println!("   Asset Allocation:");
            if let Some(stocks) = asset_allocation.get("stocks").and_then(|v| v.as_f64()) {
                println!("     Stocks: {:.1}%", stocks * 100.0);
            }
            if let Some(bonds) = asset_allocation.get("bonds").and_then(|v| v.as_f64()) {
                println!("     Bonds: {:.1}%", bonds * 100.0);
            }
            if let Some(cash) = asset_allocation.get("cash").and_then(|v| v.as_f64()) {
                println!("     Cash: {:.1}%", cash * 100.0);
            }
        }
        
        if let Some(sectors) = recommendations.get("sectors").and_then(|v| v.as_str()) {
            println!("   Sectors: {}", sectors);
        }
        
        if let Some(risk_management) = recommendations.get("risk_management").and_then(|v| v.as_str()) {
            println!("   Risk Management: {}", risk_management);
        }
        
        if let Some(position_sizing) = recommendations.get("position_sizing").and_then(|v| v.as_str()) {
            println!("   Position Sizing: {}", position_sizing);
        }
    }
    
    println!("\n✅ Market regime analysis complete! Check market_regime_analysis.json for full details.");
    Ok(())
}

/// Check for AI model availability
#[allow(dead_code)]
async fn check_ai_model() -> Result<()> {
    // ... existing AI model check code ...
    Ok(())
}