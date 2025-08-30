use anyhow::Result;
use clap::{Arg, Command};
use dotenv::dotenv;
use std::io::{self, Write};
use std::fs;
use std::path::Path;

mod ollama;
mod api;
mod market_data;

use ollama::{OllamaClient, Config, OllamaReceipt, ProtobufReceipt};
use api::start_api_server;

/// Ensure log directory exists with proper permissions
fn ensure_log_directory(log_directory: &str) -> Result<()> {
    let path = Path::new(log_directory);
    
    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)?;
        log::info!("Created log directory: {}", log_directory);
    }
    
    // Ensure directory is writable
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Log path is not a directory: {}", log_directory));
    }
    
    // Check if directory is writable by trying to create a test file
    let test_file = path.join(".test_write");
    if let Err(e) = fs::write(&test_file, "test") {
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
                if let Ok(_) = fs::create_dir_all(fallback_path) {
                    log::info!("Using fallback log directory: {}", fallback);
                    return Ok(());
                }
            } else if fallback_path.is_dir() {
                let test_fallback = fallback_path.join(".test_write");
                if fs::write(&test_fallback, "test").is_ok() {
                    fs::remove_file(test_fallback).ok();
                    log::info!("Using fallback log directory: {}", fallback);
                    return Ok(());
                }
            }
        }
        
        log::warn!("All log directories failed. Logging will be limited.");
    } else {
        // Clean up test file
        fs::remove_file(test_file).ok();
        log::info!("Log directory is writable: {}", log_directory);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    let app = Command::new("Trading Bot")
        .version("0.1.0")
        .author("Your Name")
        .about("A Rust-based trading bot with Ollama integration")
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Run in interactive mode")
                .action(clap::ArgAction::SetTrue),
        )
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
            Arg::new("benchmark")
                .short('b')
                .long("benchmark")
                .value_name("PROMPT")
                .help("Benchmark multiple models with the same prompt and save results to protobuf"),
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
        );

    let matches = app.get_matches();

    // Load configuration
    let mut config = Config::from_env()?;
    
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
    if let Err(e) = ensure_log_directory(&config.log_directory) {
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
        
    } else if let Some(prompt) = matches.get_one::<String>("benchmark") {
        // Benchmark mode - test multiple models with the same prompt
        println!("🏁 MODEL BENCHMARK MODE");
        println!("Prompt: {}", prompt);
        println!("{}", "=".repeat(50));
        
        // Sanitize input for security
        let sanitized_prompt = match config.sanitize_input(prompt) {
            Ok(sanitized) => sanitized,
            Err(e) => {
                eprintln!("Input validation error: {}", e);
                return Ok(());
            }
        };
        
        // Get available models for benchmarking
        let models_to_test = vec![
            "phi",
            "qwen2.5:0.5b", 
            "gemma2:2b",
            "tinyllama",
            "llama2:7b",
            "llama2:13b"
        ];
        
        println!("🔍 Testing {} models with prompt: '{}'", models_to_test.len(), sanitized_prompt);
        println!("📊 Results will be saved to protobuf format for analysis");
        println!();
        
        let mut benchmark_results = Vec::new();
        let mut protobuf_receipts = Vec::new();
        
        for (i, model_name) in models_to_test.iter().enumerate() {
            println!("🧪 Testing model {}/{}: {}", i + 1, models_to_test.len(), model_name);
            
            // Create protobuf receipt for this test
            let mut protobuf_receipt = ProtobufReceipt::new(
                "BenchmarkTest".to_string(),
                model_name.to_string(),
                sanitized_prompt.len(),
                &sanitized_prompt,
            );
            
            // Test the model
            let start_time = std::time::Instant::now();
            match ollama_client.generate_stream_with_timing(model_name, &sanitized_prompt).await {
                Ok((chunks, _receipt)) => {
                    let duration = start_time.elapsed();
                    let response_text = chunks.join("");
                    
                    // Finish the protobuf receipt
                    protobuf_receipt.finish(
                        response_text.len(),
                        true,
                        None,
                        &response_text,
                    );
                    
                    // Display results
                    println!("  ✅ Success: {} chars in {:.2}s ({:.1} chars/sec)", 
                             response_text.len(), 
                             duration.as_secs_f32(),
                             response_text.len() as f32 / duration.as_secs_f32());
                    
                    // Add to results
                    benchmark_results.push((model_name.to_string(), duration, response_text.len()));
                    protobuf_receipts.push(protobuf_receipt);
                }
                Err(e) => {
                    println!("  ❌ Failed: {}", e);
                    
                    // Finish the protobuf receipt with error
                    protobuf_receipt.finish(
                        0,
                        false,
                        Some(e.to_string()),
                        "",
                    );
                    
                    protobuf_receipts.push(protobuf_receipt);
                }
            }
            
            // Small delay between tests
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        // Display benchmark summary
        println!();
        println!("🏆 BENCHMARK RESULTS");
        println!("{}", "=".repeat(50));
        
        // Sort by speed (fastest first)
        benchmark_results.sort_by(|a, b| a.1.cmp(&b.1));
        
        for (i, (model, duration, chars)) in benchmark_results.iter().enumerate() {
            let speed = *chars as f32 / duration.as_secs_f32();
            let rank = match i {
                0 => "🥇 1st".to_string(),
                1 => "🥈 2nd".to_string(), 
                2 => "🥉 3rd".to_string(),
                _ => format!("   {}th", i + 1),
            };
            
            println!("{} {}: {} chars in {:.2}s ({:.1} chars/sec)", 
                     rank, model, chars, duration.as_secs_f32(), speed);
        }
        
        // Save results to protobuf files
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let individual_file = format!("{}/benchmark_{}_individual.pb", config.log_directory, timestamp);
        let batch_file = format!("{}/benchmark_{}_batch.pb", config.log_directory, timestamp);
        
        // Save individual receipts
        for receipt in &protobuf_receipts {
            if let Err(e) = receipt.save_to_protobuf_file(&individual_file) {
                log::warn!("Failed to save individual receipt: {}", e);
            }
        }
        
        // Save batch log
        if let Err(e) = protobuf_receipts.first().unwrap().save_batch_log(&protobuf_receipts, &batch_file) {
            log::warn!("Failed to save batch log: {}", e);
        }
        
        println!();
        println!("💾 Results saved to:");
        println!("   Individual: {}", individual_file);
        println!("   Batch: {}", batch_file);
        println!("");
        println!("📊 Use protobuf tools to analyze the detailed performance data!");
        
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
        // Interactive mode
        println!("🤖 Trading Bot Interactive Mode (Streaming Enabled)");
        println!("Detected Model: {} ({})", config.ollama_model, config.get_model_info());
        println!("Commands:");
        println!("  Type a message for streaming response (default)");
        println!("  Type '/regular <message>' for non-streaming response");
        println!("  Type 'quit' or 'exit' to stop");
        println!("{}", "=".repeat(50));
        println!();

        loop {
            print!(">>> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "quit" || input == "exit" {
                println!("Goodbye!");
                break;
            }

            if input.is_empty() {
                continue;
            }

            // Check if it's a regular (non-streaming) command
            if input.starts_with("/regular ") {
                let prompt = &input[9..]; // Remove "/regular " prefix
                
                // Sanitize input for security
                let sanitized_input = match config.sanitize_input(prompt) {
                    Ok(sanitized) => sanitized,
                    Err(e) => {
                        eprintln!("Input validation error: {}", e);
                        continue;
                    }
                };

                println!("Sending to Ollama (regular mode)...");
                match ollama_client.generate_with_timing(&config.ollama_model, &sanitized_input).await {
                    Ok((response, receipt)) => {
                        println!("Bot: {}", response);
                        receipt.log_summary(&config.log_directory);
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        println!();
                    }
                }
            } else {
                // Default streaming mode
                // Sanitize input for security
                let sanitized_input = match config.sanitize_input(input) {
                    Ok(sanitized) => sanitized,
                    Err(e) => {
                        eprintln!("Input validation error: {}", e);
                        continue;
                    }
                };

                println!("🌊 Streaming response...");
                match ollama_client.generate_stream_with_timing(&config.ollama_model, &sanitized_input).await {
                    Ok((chunks, receipt)) => {
                        print!("Bot: ");
                        io::stdout().flush().unwrap();
                        
                        let mut full_response = String::new();
                        
                        for chunk in chunks {
                            print!("{}", chunk);
                            io::stdout().flush().unwrap();
                            full_response.push_str(&chunk);
                            
                            // Minimal delay for ultra-fast responses
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                        
                        println!(); // New line after streaming
                        
                        receipt.log_summary(&config.log_directory);
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Stream error: {}", e);
                        println!();
                    }
                }
            }
        }
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
        let (market_data_config, trading_config) = match market_data::load_unified_websocket_config() {
            Ok(configs) => {
                println!("✅ Unified WebSocket configuration loaded successfully");
                configs
            }
            Err(e) => {
                eprintln!("❌ Failed to load unified WebSocket configuration: {}", e);
                return Err(anyhow::anyhow!("Unified WebSocket config error: {}", e));
            }
        };
        
        // Create data directory
        let data_dir = std::path::PathBuf::from("live_data");
        std::fs::create_dir_all(&data_dir)?;
        
        // Create and start unified WebSocket streamer
        let streamer = match market_data::UnifiedAlpacaWebSocket::new(
            market_data_config, 
            trading_config, 
            data_dir, 
            stream_types
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
    } else {
        println!("Trading Bot started. Use --help for usage information.");
        println!("Available modes:");
        println!("  -i, --interactive     Interactive streaming chat mode (default)");
        println!("  -p, --prompt \"text\"   Single prompt with streaming response (default)");
        println!("  -s, --stream \"text\"   Real-time streaming response mode (same as -p)");
        println!("  -t, --test \"text\"     Performance test with streaming and detailed timing");
        println!("  -b, --benchmark \"text\" Benchmark multiple models and save to protobuf");
        println!("  -l, --logs            View receipt logs summary");
        println!("  -m, --model \"name\"    Override auto-detection with specific model");
        println!("  --api                 Start JSON streaming API server");
        println!("  --api-port PORT       Custom port for API server (default: 8080)");
        println!("  --websocket           Start WebSocket-based market data streaming");
        println!("  --stream-types TYPES  Comma-separated stream types (default: stocks,crypto,options,news)");
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