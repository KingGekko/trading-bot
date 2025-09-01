use anyhow::Result;
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;
use tokio::time::{sleep, Duration};
use tokio::fs;
use crate::ollama::Config;

/// Interactive setup wizard for the trading bot
pub struct InteractiveSetup {
    pub trading_mode: String,
    pub model_mode: String,
    pub selected_model: String,
    pub models: Vec<String>,
    pub config: Config,
}

impl InteractiveSetup {
    /// Get the correct binary path for the current platform
    fn get_binary_path(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "./target/release/trading_bot.exe"
        } else {
            "./target/release/trading_bot"
        }
    }
    
    pub fn new() -> Self {
        Self {
            trading_mode: String::new(),
            model_mode: String::new(),
            selected_model: String::new(),
            models: Vec::new(),
            config: Config::from_env().unwrap_or_else(|_| {
                Config {
                    ollama_base_url: "http://localhost:11434".to_string(),
                    ollama_model: "tinyllama".to_string(),
                    max_timeout_seconds: 120,  // Increased from 30 to match config.env
                    log_directory: "./logs".to_string(),
                    max_prompt_length: 4000,
                }
            }),
        }
    }

    /// Run the complete interactive setup
    pub async fn run_setup(&mut self) -> Result<()> {
        println!("🚀 TRADING BOT INTERACTIVE SETUP");
        println!("{}", "=".repeat(50));
        println!("Welcome to the Elite Trading Bot Setup Wizard");
        println!("This will configure your trading environment for maximum profit multiplication");
        println!();
        
        // Ensure required directories exist
        println!("🔧 Setting up required directories...");
        let required_dirs = ["ollama_logs", "trading_portfolio", "live_data", "sandbox_data", "logs"];
        for dir in &required_dirs {
            if !std::path::Path::new(dir).exists() {
                if let Err(e) = tokio::fs::create_dir_all(dir).await {
                    println!("⚠️  Warning: Failed to create directory {}: {}", dir, e);
                } else {
                    println!("✅ Created directory: {}", dir);
                }
            } else {
                println!("✅ Directory exists: {}", dir);
            }
        }
        println!();

        // Step 1: Trading Mode Selection
        self.select_trading_mode().await?;
        
        // Step 2: Model Mode Selection
        self.select_model_mode().await?;
        
        // Step 3: Model Selection (if single mode)
        if self.model_mode == "single" {
            self.select_model().await?;
        }
        
        // Step 4: Start all servers
        self.start_servers().await?;
        
        // Step 5: Begin automatic trading
        self.start_automatic_trading().await?;
        
        Ok(())
    }

    /// Step 1: Select trading mode (Paper or Live)
    async fn select_trading_mode(&mut self) -> Result<()> {
        println!("📊 STEP 1: TRADING MODE SELECTION");
        println!("{}", "=".repeat(30));
        println!("Choose your trading mode:");
        println!("1. Paper Trading (Safe testing with virtual money)");
        println!("2. Live Trading (Real money - HIGH RISK)");
        println!();

        loop {
            print!("Enter your choice (1 or 2): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    self.trading_mode = "paper".to_string();
                    println!("✅ Selected: Paper Trading Mode");
                    println!("   - Virtual money trading");
                    println!("   - Safe for testing strategies");
                    println!("   - No real financial risk");
                    break;
                }
                "2" => {
                    self.trading_mode = "live".to_string();
                    println!("⚠️  Selected: Live Trading Mode");
                    println!("   - REAL MONEY TRADING");
                    println!("   - HIGH FINANCIAL RISK");
                    println!("   - Ensure you have proper API keys configured");
                    
                    // Warning for live trading
                    print!("\n⚠️  WARNING: You are about to trade with REAL MONEY. Continue? (yes/no): ");
                    io::stdout().flush()?;
                    
                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm)?;
                    
                    if confirm.trim().to_lowercase() == "yes" {
                        break;
                    } else {
                        println!("Live trading cancelled. Please select paper trading instead.");
                        continue;
                    }
                }
                _ => {
                    println!("❌ Invalid choice. Please enter 1 or 2.");
                }
            }
        }
        
        println!();
        Ok(())
    }

    /// Step 2: Select model mode (Single or Multi)
    async fn select_model_mode(&mut self) -> Result<()> {
        println!("🤖 STEP 2: AI MODEL MODE SELECTION");
        println!("{}", "=".repeat(30));
        println!("Choose your AI model configuration:");
        println!("1. Single Model (One AI model for all analysis)");
        println!("2. Multi Model (Multiple AI models for enhanced analysis)");
        println!();

        loop {
            print!("Enter your choice (1 or 2): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    self.model_mode = "single".to_string();
                    println!("✅ Selected: Single Model Mode");
                    println!("   - One AI model for all analysis");
                    println!("   - Faster processing");
                    println!("   - Consistent analysis style");
                    break;
                }
                "2" => {
                    self.model_mode = "multi".to_string();
                    println!("✅ Selected: Multi Model Mode");
                    println!("   - Multiple AI models for enhanced analysis");
                    println!("   - More comprehensive insights");
                    println!("   - Higher computational requirements");
                    break;
                }
                _ => {
                    println!("❌ Invalid choice. Please enter 1 or 2.");
                }
            }
        }
        
        println!();
        Ok(())
    }

    /// Step 3: Select specific model (for single mode)
    async fn select_model(&mut self) -> Result<()> {
        println!("🎯 STEP 3: AI MODEL SELECTION");
        println!("{}", "=".repeat(30));
        
        // Get available models from Ollama
        println!("Fetching available AI models from Ollama...");
        self.fetch_available_models().await?;
        
        if self.models.is_empty() {
            println!("❌ No models found. Please ensure Ollama is running and has models installed.");
            return Err(anyhow::anyhow!("No Ollama models available"));
        }
        
        println!("Available AI models:");
        for (i, model) in self.models.iter().enumerate() {
            println!("{}. {}", i + 1, model);
        }
        println!();

        loop {
            print!("Enter the number of your preferred model: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= self.models.len() {
                    self.selected_model = self.models[choice - 1].clone();
                    println!("✅ Selected: {}", self.selected_model);
                    break;
                } else {
                    println!("❌ Invalid choice. Please enter a number between 1 and {}.", self.models.len());
                }
            } else {
                println!("❌ Invalid input. Please enter a number.");
            }
        }
        
        println!();
        Ok(())
    }

    /// Fetch available models from Ollama
    async fn fetch_available_models(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/api/tags", self.config.ollama_base_url))
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;
            if let Some(models_array) = json["models"].as_array() {
                self.models = models_array
                    .iter()
                    .filter_map(|model| model["name"].as_str())
                    .map(|s| s.to_string())
                    .collect();
            }
        }
        
        Ok(())
    }

    /// Step 4: Start all required servers
    async fn start_servers(&mut self) -> Result<()> {
        println!("🌐 STEP 4: STARTING SERVERS");
        println!("{}", "=".repeat(30));
        
        // Start market data collection
        println!("📊 Starting market data collection...");
        self.start_market_data_collection().await?;
        
        // Start portfolio analysis server
        println!("🤖 Starting portfolio analysis server...");
        self.start_portfolio_analysis_server().await?;
        
        // Start streaming (only for live mode)
        if self.trading_mode == "live" {
            println!("📡 Starting live data streaming...");
            self.start_streaming().await?;
        } else {
            println!("📊 Paper trading mode - using REST API data collection");
        }
        
        println!("✅ All servers started successfully!");
        println!();
        Ok(())
    }

    /// Start market data collection
    async fn start_market_data_collection(&self) -> Result<()> {
        // Run enhanced strategy to collect initial data
        let binary_name = self.get_binary_path();
        
        // Check if binary exists
        if !std::path::Path::new(binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping market data collection.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        let mut cmd = Command::new(binary_name);
        cmd.arg("--enhanced-strategy");
        
        let output = cmd.output()?;
        if !output.status.success() {
            let error_output = String::from_utf8_lossy(&output.stderr);
            println!("⚠️  Market data collection output: {}", String::from_utf8_lossy(&output.stdout));
            println!("⚠️  Market data collection errors: {}", error_output);
            println!("⚠️  Skipping market data collection due to errors.");
            return Ok(()); // Don't fail the setup, just skip this step
        }
        
        println!("   ✅ Market data collection started");
        Ok(())
    }

    /// Start portfolio analysis server
    async fn start_portfolio_analysis_server(&self) -> Result<()> {
        // Start the portfolio analysis server in background
        let binary_name = self.get_binary_path();
        
        // Check if binary exists
        if !std::path::Path::new(binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping portfolio analysis server.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        let mut cmd = Command::new(binary_name);
        cmd.arg("--portfolio-analysis");
        
        // Run in background
        match cmd.spawn() {
            Ok(_child) => {
                // Wait a moment for server to start
                sleep(Duration::from_secs(3)).await;
                println!("   ✅ Portfolio analysis server started on port 8082");
            }
            Err(e) => {
                println!("⚠️  Failed to start portfolio analysis server: {}. Skipping.", e);
            }
        }
        
        Ok(())
    }

    /// Start streaming (live mode only)
    async fn start_streaming(&self) -> Result<()> {
        // Start streaming for live trading
        let binary_name = self.get_binary_path();
        
        // Check if binary exists
        if !std::path::Path::new(binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping live streaming.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        let mut cmd = Command::new(binary_name);
        cmd.arg("--websocket");
        
        // Run in background
        match cmd.spawn() {
            Ok(_child) => {
                // Wait a moment for streaming to start
                sleep(Duration::from_secs(3)).await;
                println!("   ✅ Live data streaming started");
            }
            Err(e) => {
                println!("⚠️  Failed to start live streaming: {}. Skipping.", e);
            }
        }
        
        Ok(())
    }

    /// Step 5: Start automatic trading
    async fn start_automatic_trading(&mut self) -> Result<()> {
        println!("🎯 STEP 5: STARTING AUTOMATIC TRADING");
        println!("{}", "=".repeat(30));
        println!("🚀 Elite Trading Bot is now active!");
        println!("📊 Monitoring market data and generating AI-powered trading decisions");
        println!("💰 Focus: Profit multiplication and market transcendence");
        println!();
        
        // Start the trading loop
        self.run_trading_loop().await?;
        
        Ok(())
    }

    /// Main trading loop
    async fn run_trading_loop(&mut self) -> Result<()> {
        println!("🔄 TRADING LOOP STARTED");
        println!("{}", "=".repeat(30));
        println!("The bot will now:");
        println!("1. 📊 Continuously monitor market data");
        println!("2. 🧠 Generate AI-powered trading decisions");
        println!("3. 🎯 Execute trades automatically");
        println!("4. 📈 Optimize portfolio for profit multiplication");
        println!();
        println!("Press Ctrl+C to stop trading and view final results");
        println!();

        let mut iteration = 0;
        loop {
            iteration += 1;
            println!("🔄 Trading Iteration #{}", iteration);
            println!("{}", "-".repeat(20));
            
            // Check if market is open (for live trading)
            if self.trading_mode == "live" && !self.is_market_open().await? {
                println!("⏰ Market is closed. Waiting for market to open...");
                sleep(Duration::from_secs(300)).await; // Wait 5 minutes
                continue;
            }
            
            // Scan for tradeable assets
            let tradeable_assets = self.scan_tradeable_assets().await?;
            if tradeable_assets.is_empty() {
                println!("⚠️ No tradeable assets found. Skipping iteration...");
                sleep(Duration::from_secs(30)).await;
                continue;
            }
            
            // Generate AI-enhanced decisions
            if self.model_mode == "single" {
                self.run_single_model_analysis().await?;
            } else {
                self.run_multi_model_analysis().await?;
            }
            
            // Execute trades
            self.execute_trades().await?;
            
            // Display portfolio status
            self.display_portfolio_status().await?;
            
            // Wait before next iteration
            println!("⏳ Waiting 30 seconds before next analysis...");
            sleep(Duration::from_secs(30)).await;
            println!();
        }
    }

    /// Run single model analysis
    async fn run_single_model_analysis(&self) -> Result<()> {
        println!("🧠 Running AI analysis with model: {}", self.selected_model);
        
        // Use the JSON streaming API server to communicate with Ollama
        let client = reqwest::Client::new();
        
        // Send portfolio analysis request to the API server
        let response = client
            .post("http://localhost:8082/api/ollama/process")
            .json(&serde_json::json!({
                "file_path": "trading_portfolio/trading_portfolio.json",
                "model": self.selected_model,
                "prompt": "You are an Elite quantitative trading analyst. Analyze the following trading data to transcend in profit multiplication. Generate specific trading recommendations with buy/sell actions, quantities, and prices."
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: Value = response.json().await?;
            
            // Save the AI analysis result to the expected file
            let ai_analysis_file = "trading_portfolio/ai_analysis_report.json";
            tokio::fs::write(ai_analysis_file, serde_json::to_string_pretty(&result)?).await?;
            
            println!("✅ AI analysis completed successfully");
            println!("📊 Analysis saved to: {}", ai_analysis_file);
        } else {
            println!("⚠️ AI analysis completed with warnings");
        }
        
        Ok(())
    }

    /// Run multi model analysis
    async fn run_multi_model_analysis(&self) -> Result<()> {
        println!("🧠 Running multi-model AI analysis");
        
        // Use portfolio analysis with multi-model conversation
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8082/api/ollama/conversation")
            .json(&serde_json::json!({
                "file_path": "trading_portfolio/trading_portfolio.json",
                "initial_prompt": "You are an Elite quantitative trading analyst. Analyze the following trading data to transcend in profit multiplication. Generate specific trading recommendations with buy/sell actions, quantities, and prices.",
                "models": self.models
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: Value = response.json().await?;
            
            // Save the AI analysis result to the expected file
            let ai_analysis_file = "trading_portfolio/ai_analysis_report.json";
            tokio::fs::write(ai_analysis_file, serde_json::to_string_pretty(&result)?).await?;
            
            println!("✅ Multi-model analysis completed successfully");
            println!("📊 Analysis saved to: {}", ai_analysis_file);
        } else {
            println!("⚠️ Multi-model analysis completed with warnings");
        }
        
        Ok(())
    }

    /// Execute trades based on AI recommendations
    async fn execute_trades(&self) -> Result<()> {
        println!("🎯 Executing trades based on AI recommendations");
        
        // Check if we have AI analysis results
        let ai_analysis_file = "trading_portfolio/ai_analysis_report.json";
        if !std::path::Path::new(ai_analysis_file).exists() {
            println!("⚠️ No AI analysis found. Skipping trade execution.");
            return Ok(());
        }

        // Read AI analysis results
        let content = tokio::fs::read_to_string(ai_analysis_file).await?;
        let analysis: Value = serde_json::from_str(&content)?;
        
        // Extract trading recommendations from AI enhanced decisions
        if let Some(decisions) = analysis["ai_enhanced_decisions"].as_array() {
            for decision in decisions {
                if let Some(symbol) = decision["symbol"].as_str() {
                    if let Some(action_obj) = decision["action"].as_object() {
                        // Extract action type
                        let action_type = if let Some(action_str) = action_obj["variant"].as_str() {
                            match action_str {
                                "OpenLong" => "buy",
                                "OpenShort" => "sell",
                                "CloseLong" => "sell",
                                "CloseShort" => "buy",
                                _ => "hold"
                            }
                        } else {
                            "hold"
                        };
                        
                        if action_type != "hold" {
                            if let Some(position_size) = decision["position_size"].as_f64() {
                                if let Some(current_price) = decision["current_price"].as_f64() {
                                    // Calculate quantity based on position size and current price
                                    let quantity = (position_size.abs() / current_price) as i32;
                                    
                                    // Execute the trade
                                    self.execute_single_trade(symbol, action_type, quantity, current_price).await?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        println!("✅ Trade execution completed");
        Ok(())
    }

    /// Execute a single trade using Alpaca API
    async fn execute_single_trade(&self, symbol: &str, action: &str, quantity: i32, price: f64) -> Result<()> {
        println!("📈 Executing {} order: {} {} shares of {} at ${:.2}", 
                 action, quantity, symbol, symbol, price);
        
        // Skip execution in paper trading mode for safety
        if self.trading_mode == "paper" {
            println!("📝 Paper trading mode - Simulating order execution");
            return Ok(());
        }

        // For live trading, execute real orders
        let client = reqwest::Client::new();
        let api_key = std::env::var("ALPACA_API_KEY").unwrap_or_default();
        let secret_key = std::env::var("ALPACA_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || secret_key.is_empty() {
            println!("⚠️ Alpaca API keys not configured. Simulating order execution.");
            return Ok(());
        }

        let base_url = if self.trading_mode == "live" {
            "https://api.alpaca.markets"
        } else {
            "https://paper-api.alpaca.markets"
        };

        // Create order request according to Alpaca API documentation
        let order_request = serde_json::json!({
            "symbol": symbol,
            "qty": quantity.abs(),
            "side": if action.to_lowercase().contains("buy") { "buy" } else { "sell" },
            "type": "limit",
            "time_in_force": "day",
            "limit_price": price
        });

        let response = client
            .post(&format!("{}/v2/orders", base_url))
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &secret_key)
            .header("Content-Type", "application/json")
            .json(&order_request)
            .send()
            .await?;

        if response.status().is_success() {
            let order_response: Value = response.json().await?;
            let order_id = order_response["id"].as_str().unwrap_or("unknown");
            println!("✅ Order executed successfully! Order ID: {}", order_id);
        } else {
            let error_text = response.text().await?;
            println!("❌ Order execution failed: {}", error_text);
        }

        Ok(())
    }

    /// Display current portfolio status
    async fn display_portfolio_status(&self) -> Result<()> {
        println!("📊 Current Portfolio Status:");
        
        // Read portfolio data
        let portfolio_file = "trading_portfolio/trading_portfolio.json";
        if std::path::Path::new(portfolio_file).exists() {
            let content = tokio::fs::read_to_string(portfolio_file).await?;
            let data: Value = serde_json::from_str(&content)?;
            
            if let Some(account_info) = data["trading_account"]["account_info"].as_object() {
                let portfolio_value = account_info["portfolio_value"].as_str().unwrap_or("0");
                let cash = account_info["cash"].as_str().unwrap_or("0");
                let equity = account_info["equity"].as_str().unwrap_or("0");
                
                println!("   💰 Portfolio Value: ${}", portfolio_value);
                println!("   💵 Cash: ${}", cash);
                println!("   📈 Equity: ${}", equity);
            }
        }
        
        Ok(())
    }

    /// Check if the market is currently open (for live trading)
    async fn is_market_open(&self) -> Result<bool> {
        if self.trading_mode == "paper" {
            return Ok(true); // Paper trading can run anytime
        }

        // For live trading, check Alpaca market hours
        let client = reqwest::Client::new();
        let api_key = std::env::var("ALPACA_API_KEY").unwrap_or_default();
        let secret_key = std::env::var("ALPACA_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || secret_key.is_empty() {
            println!("⚠️ Alpaca API keys not configured. Assuming market is open for testing.");
            return Ok(true);
        }

        // Check market clock using Alpaca API
        let base_url = if self.trading_mode == "live" {
            "https://api.alpaca.markets"
        } else {
            "https://paper-api.alpaca.markets"
        };

        let response = client
            .get(&format!("{}/v2/clock", base_url))
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let clock_data: Value = response.json().await?;
            let is_open = clock_data["is_open"].as_bool().unwrap_or(false);
            let next_open = clock_data["next_open"].as_str().unwrap_or("");
            let next_close = clock_data["next_close"].as_str().unwrap_or("");
            
            if is_open {
                println!("✅ Market is OPEN - Trading allowed");
            } else {
                println!("⏰ Market is CLOSED");
                println!("   Next open: {}", next_open);
                println!("   Next close: {}", next_close);
            }
            
            Ok(is_open)
        } else {
            println!("⚠️ Could not check market status. Assuming market is open.");
            Ok(true)
        }
    }

    /// Scan for tradeable assets from the asset universe
    async fn scan_tradeable_assets(&self) -> Result<Vec<String>> {
        println!("🔍 Scanning for tradeable assets...");
        
        // Load asset universe
        let asset_universe_file = "trading_portfolio/asset_universe.json";
        if !std::path::Path::new(asset_universe_file).exists() {
            println!("⚠️ Asset universe file not found. Creating sample assets...");
            return Ok(vec!["AAPL".to_string(), "SPY".to_string(), "TSLA".to_string()]);
        }

        let content = tokio::fs::read_to_string(asset_universe_file).await?;
        let data: Value = serde_json::from_str(&content)?;
        
        let mut tradeable_assets = Vec::new();
        if let Some(assets) = data["assets"].as_array() {
            for asset in assets {
                if let Some(symbol) = asset["symbol"].as_str() {
                    if let Some(tradeable) = asset["tradeable"].as_bool() {
                        if tradeable {
                            tradeable_assets.push(symbol.to_string());
                        }
                    }
                }
            }
        }

        if tradeable_assets.is_empty() {
            // Fallback to common tradeable assets
            tradeable_assets = vec![
                "AAPL".to_string(),
                "SPY".to_string(), 
                "TSLA".to_string(),
                "MSFT".to_string(),
                "GOOGL".to_string()
            ];
        }

        println!("✅ Found {} tradeable assets: {:?}", tradeable_assets.len(), tradeable_assets);
        Ok(tradeable_assets)
    }
}
