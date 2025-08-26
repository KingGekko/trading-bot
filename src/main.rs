use anyhow::Result;
use clap::{Arg, Command};
use dotenv::dotenv;
use std::io::{self, Write};

mod ollama;

use ollama::{OllamaClient, Config, OllamaReceipt};

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
        );

    let matches = app.get_matches();

    // Load configuration
    let config = Config::from_env()?;
    
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
        println!("Model: {}", config.ollama_model);
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
        println!("Model: {}", config.ollama_model);
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
        println!("Trading Bot Interactive Mode (Streaming Enabled)");
        println!("Commands:");
        println!("  Type a message for streaming response (default)");
        println!("  Type '/regular <message>' for non-streaming response");
        println!("  Type 'quit' or 'exit' to stop");
        println!("Using model: {}", config.ollama_model);
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
    } else {
        println!("Trading Bot started. Use --help for usage information.");
        println!("Available modes:");
        println!("  -i, --interactive     Interactive streaming chat mode (default)");
        println!("  -p, --prompt \"text\"   Single prompt with streaming response (default)");
        println!("  -s, --stream \"text\"   Real-time streaming response mode (same as -p)");
        println!("  -t, --test \"text\"     Performance test with streaming and detailed timing");
        println!("  -l, --logs            View receipt logs summary");
        println!();
        println!("💡 Streaming is now the default for all modes for enhanced responsiveness!");
        println!();
        println!("⚡ Performance Tips:");
        println!("   • For fastest responses (3-5s), try models: phi, qwen2.5:0.5b, gemma2:2b");
        println!("   • Current model optimization: Reduced tokens, faster sampling");
        println!("   • Connection pooling and TCP keep-alive enabled for better throughput");
    }

    Ok(())
}