use anyhow::Result;
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;
use tokio::time::{sleep, Duration};
use regex::Regex;

use chrono;
use crate::ollama::Config;

#[derive(Debug)]
struct AIRecommendation {
    symbol: String,
    action: String,
    quantity: i32,
    price: f64,
}

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
    
    /// Get the correct binary path with fallback options
    fn get_binary_path_with_fallback(&self) -> String {
        let paths = vec![
            "./target/release/trading_bot",
            "./target/debug/trading_bot", 
            "../target/release/trading_bot",
            "../target/debug/trading_bot",
            "trading_bot",
            "/opt/trading-bot/target/release/trading_bot",
            "/opt/trading-bot/target/debug/trading_bot"
        ];
        
        for path in paths {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }
        
        // Return default if none found
        "./target/release/trading_bot".to_string()
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
        
        // Create sample JSON files for AI analysis
        self.create_sample_json_files().await?;

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
        
        // Wait for API server to be ready
        println!("⏳ Waiting for API server to be ready...");
        self.wait_for_api_server().await?;
        
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
        let binary_name = self.get_binary_path_with_fallback();
        
        // Check if binary exists
        if !std::path::Path::new(&binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping market data collection.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        let mut cmd = Command::new(&binary_name);
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
        let binary_name = self.get_binary_path_with_fallback();
        
        // Check if binary exists
        if !std::path::Path::new(&binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping portfolio analysis server.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        // Get API port from environment or use default
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
        
        let mut cmd = Command::new(&binary_name);
        cmd.arg("--portfolio-analysis");
        cmd.arg("--api-port");
        cmd.arg(&api_port.to_string());
        
        // Run in background
        match cmd.spawn() {
            Ok(child) => {
                // Wait a moment for server to start
                sleep(Duration::from_secs(5)).await;
                println!("   ✅ Portfolio analysis server started on port {}", api_port);
                println!("   🔧 Server process ID: {}", child.id());
            }
            Err(e) => {
                println!("⚠️  Failed to start portfolio analysis server: {}. Skipping.", e);
            }
        }
        
        Ok(())
    }
    
    /// Wait for API server to be ready
    async fn wait_for_api_server(&self) -> Result<()> {
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
        
        let client = reqwest::Client::new();
        let max_attempts = 60; // 60 seconds max wait
        let mut attempts = 0;
        
        while attempts < max_attempts {
            match client.get(format!("http://localhost:{}/health", api_port))
                .timeout(Duration::from_secs(2))
                .send()
                .await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("   ✅ API server is ready on port {}", api_port);
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Server not ready yet
                }
            }
            
            attempts += 1;
            if attempts % 5 == 0 {
                println!("   ⏳ Still waiting for API server... (attempt {}/{})", attempts, max_attempts);
            }
            sleep(Duration::from_secs(1)).await;
        }
        
        println!("⚠️  API server did not start within {} seconds", max_attempts);
        println!("   The setup will continue, but AI analysis may not work.");
        Ok(())
    }

    /// Start streaming (live mode only)
    async fn start_streaming(&self) -> Result<()> {
        // Start streaming for live trading
        let binary_name = self.get_binary_path_with_fallback();
        
        // Check if binary exists
        if !std::path::Path::new(&binary_name).exists() {
            println!("⚠️  Binary not found: {}. Skipping live streaming.", binary_name);
            println!("   This is normal if running from a different directory.");
            return Ok(());
        }
        
        let mut cmd = Command::new(&binary_name);
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
            
            // Check for emergency stop before executing trades
            if self.is_emergency_stop_triggered().await? {
                println!("🚨 EMERGENCY STOP TRIGGERED - Suspending trading");
                println!("   Portfolio protection activated. Waiting for recovery...");
                sleep(Duration::from_secs(60)).await; // Wait 1 minute before checking again
                continue;
            }
            
            // Monitor and liquidate profitable positions (0.25% profit target)
            self.monitor_and_liquidate_positions().await?;
            
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
        
        // Get API port from environment or use default
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
            
        // Get live account data for the prompt
        let account_data = self.get_real_account_data().await?;
        let positions = self.get_current_positions().await?;
        
        // Create enhanced prompt with live data
        let enhanced_prompt = format!(
            "You are an Elite quantitative trading analyst specializing in profit multiplication and market transcendence.

🎯 MISSION: Analyze the following LIVE trading data and provide precise trading recommendations for maximum profit generation.

💰 LIVE ACCOUNT DATA:
- Portfolio Value: ${}
- Cash Available: ${}
- Equity: ${}

📋 CURRENT POSITIONS:
{}

📊 MARKET CONDITIONS:
- Market Regime: Low Volatility
- Risk Level: Medium
- Available Assets: 70 diversified assets

🧠 ANALYSIS REQUIREMENTS:
1. Focus on profit multiplication strategies
2. Consider current positions and their performance
3. Identify high-probability entry/exit points
4. Optimize position sizing for maximum returns
5. Use technical analysis for timing

🎯 RESPONSE FORMAT:
Provide specific trading recommendations in this exact format:
- BUY [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- SELL [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- HOLD [SYMBOL] - [REASONING]

Focus on actionable trades that will multiply profits.",
            account_data["portfolio_value"].as_str().unwrap_or("100000"),
            account_data["cash"].as_str().unwrap_or("100000"),
            account_data["equity"].as_str().unwrap_or("100000"),
            if positions.is_empty() {
                "No current positions".to_string()
            } else {
                positions.iter().map(|p| {
                    format!("- {}: {} shares, P&L: ${:.2}", 
                        p["symbol"].as_str().unwrap_or("UNKNOWN"),
                        p["qty"].as_str().unwrap_or("0"),
                        p["unrealized_pl"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0)
                    )
                }).collect::<Vec<_>>().join("\n")
            }
        );

        // Send portfolio analysis request to the API server
        let response = client
            .post(format!("http://localhost:{}/api/ollama/process", api_port))
            .json(&serde_json::json!({
                "file_path": "trading_portfolio/trading_portfolio.json",
                "model": self.selected_model,
                "prompt": enhanced_prompt
            }))
            .timeout(Duration::from_secs(60))
            .send()
            .await;
            
        let response = match response {
            Ok(resp) => resp,
            Err(e) => {
                println!("⚠️  Failed to connect to API server on port {}: {}", api_port, e);
                println!("   This is normal during startup. The server may still be starting.");
                println!("   Skipping AI analysis for this iteration.");
                return Ok(());
            }
        };
        
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
        
        // Get API port from environment or use default
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
            
        // Get live account data for the prompt
        let account_data = self.get_real_account_data().await?;
        let positions = self.get_current_positions().await?;
        
        // Create enhanced prompt with live data
        let enhanced_prompt = format!(
            "You are an Elite quantitative trading analyst specializing in profit multiplication and market transcendence.

🎯 MISSION: Analyze the following LIVE trading data and provide precise trading recommendations for maximum profit generation.

💰 LIVE ACCOUNT DATA:
- Portfolio Value: ${}
- Cash Available: ${}
- Equity: ${}

📋 CURRENT POSITIONS:
{}

📊 MARKET CONDITIONS:
- Market Regime: Low Volatility
- Risk Level: Medium
- Available Assets: 70 diversified assets

🧠 ANALYSIS REQUIREMENTS:
1. Focus on profit multiplication strategies
2. Consider current positions and their performance
3. Identify high-probability entry/exit points
4. Optimize position sizing for maximum returns
5. Use technical analysis for timing

🎯 RESPONSE FORMAT:
Provide specific trading recommendations in this exact format:
- BUY [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- SELL [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- HOLD [SYMBOL] - [REASONING]

Focus on actionable trades that will multiply profits.",
            account_data["portfolio_value"].as_str().unwrap_or("100000"),
            account_data["cash"].as_str().unwrap_or("100000"),
            account_data["equity"].as_str().unwrap_or("100000"),
            if positions.is_empty() {
                "No current positions".to_string()
            } else {
                positions.iter().map(|p| {
                    format!("- {}: {} shares, P&L: ${:.2}", 
                        p["symbol"].as_str().unwrap_or("UNKNOWN"),
                        p["qty"].as_str().unwrap_or("0"),
                        p["unrealized_pl"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0)
                    )
                }).collect::<Vec<_>>().join("\n")
            }
        );

        // Use portfolio analysis with multi-model conversation
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://localhost:{}/api/ollama/conversation", api_port))
            .json(&serde_json::json!({
                "file_path": "trading_portfolio/trading_portfolio.json",
                "initial_prompt": enhanced_prompt,
                "models": self.models
            }))
            .timeout(Duration::from_secs(60))
            .send()
            .await;
            
        let response = match response {
            Ok(resp) => resp,
            Err(e) => {
                println!("⚠️  Failed to connect to API server on port {}: {}", api_port, e);
                println!("   This is normal during startup. The server may still be starting.");
                println!("   Skipping multi-model analysis for this iteration.");
                return Ok(());
            }
        };
        
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

    /// Map company names to stock symbols
    fn map_company_to_symbol(&self, company: &str) -> String {
        match company.to_uppercase().as_str() {
            "APPLE" | "AAPL" => "AAPL".to_string(),
            "MICROSOFT" | "MSFT" => "MSFT".to_string(),
            "TESLA" | "TSLA" => "TSLA".to_string(),
            "GOOGLE" | "GOOGL" | "ALPHABET" => "GOOGL".to_string(),
            "SPY" | "SPDR" => "SPY".to_string(),
            "QQQ" => "QQQ".to_string(),
            "ALL" => "AAPL".to_string(), // Default mapping for "all"
            _ => company.to_uppercase(),
        }
    }

    /// Parse AI recommendations from natural language response
    fn parse_ai_recommendations(&self, response: &str) -> Vec<AIRecommendation> {
        let mut recommendations = Vec::new();
        let response_lower = response.to_lowercase();
        
        // Parse buy recommendations - Updated patterns to match AI response format (case insensitive)
        let buy_patterns = [
            "(?i)buy (\\d+) shares of ([A-Z]+) at \\$([0-9.]+)",
            "(?i)buy (\\d+) shares of ([A-Z]+)",
            "(?i)buy ([A-Z]+) at \\$([0-9.]+)",
            "(?i)buy ([A-Z]+) shares",
            "(?i)buy ([A-Z]+):",
            "(?i)\\d+\\. buy ([A-Z]+):",
        ];
        
        println!("🔍 Testing buy patterns on: {}", response_lower);
        
        for pattern in &buy_patterns {
            println!("🔍 Testing buy pattern: {}", pattern);
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(&response_lower) {
                    println!("🔍 Buy pattern matched! Captures: {:?}", cap);
                    // Use original response for symbol extraction to preserve case
                    let cap_original = regex.captures_iter(response).next();
                    let cap_to_use = cap_original.as_ref().unwrap_or(&cap);
                    println!("🔍 Processing buy capture with {} groups", cap_to_use.len());
                    let (quantity, symbol, price) = if cap_to_use.len() == 4 {
                        // Pattern: "buy 100 shares of AAPL at $150.62"
                        if let (Some(quantity_str), Some(symbol), Some(price_str)) = (cap_to_use.get(1), cap_to_use.get(2), cap_to_use.get(3)) {
                            if let Ok(quantity) = quantity_str.as_str().parse::<i32>() {
                                if let Ok(price) = price_str.as_str().parse::<f64>() {
                                    (quantity, symbol.as_str(), price)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else if cap_to_use.len() == 2 {
                        // Pattern: "buy AAPL:" or "1. buy AAPL:" (numbered list)
                        println!("🔍 Processing 2-group buy pattern");
                        if let Some(symbol) = cap_to_use.get(1) {
                            println!("🔍 Detected symbol-only pattern: '{}'", symbol.as_str());
                            let price = self.extract_price_for_symbol(response, symbol.as_str()).unwrap_or(150.0);
                            (100, symbol.as_str(), price) // Default quantity
                        } else {
                            continue;
                        }
                    } else if cap_to_use.len() == 3 {
                        // Pattern: "buy 100 shares of AAPL" or "buy AAPL at $150.62" or "1. buy AAPL:"
                        println!("🔍 Processing 3-group buy pattern");
                        if let (Some(first), Some(second)) = (cap_to_use.get(1), cap_to_use.get(2)) {
                            if first.as_str().chars().all(|c| c.is_ascii_digit()) {
                                // Pattern: "buy 100 shares of AAPL" or "1. buy AAPL:"
                                println!("🔍 First group is numeric: '{}'", first.as_str());
                                if first.as_str().len() <= 2 && first.as_str().parse::<i32>().is_ok() {
                                    // Pattern: "1. buy AAPL:" (numbered list)
                                    println!("🔍 Detected numbered list pattern: '{}'", first.as_str());
                                    let price = self.extract_price_for_symbol(response, second.as_str()).unwrap_or(150.0);
                                    (100, second.as_str(), price) // Default quantity
                                } else {
                                    // Pattern: "buy 100 shares of AAPL"
                                    println!("🔍 Detected quantity pattern: '{}'", first.as_str());
                                    if let Ok(quantity) = first.as_str().parse::<i32>() {
                                        let price = self.extract_price_for_symbol(response, second.as_str()).unwrap_or(150.0);
                                        (quantity, second.as_str(), price)
                                    } else {
                                        continue;
                                    }
                                }
                            } else {
                                // Pattern: "buy AAPL at $150.62" or "buy AAPL:"
                                println!("🔍 Detected symbol-only pattern: '{}'", first.as_str());
                                let price = self.extract_price_for_symbol(response, first.as_str()).unwrap_or(150.0);
                                (100, first.as_str(), price) // Default quantity
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };
                    
                    let mapped_symbol = self.map_company_to_symbol(symbol);
                    println!("🔍 Adding buy recommendation: {} {} shares at ${}", mapped_symbol, quantity, price);
                    
                    // Check for duplicates before adding
                    if !recommendations.iter().any(|r: &AIRecommendation| r.symbol == mapped_symbol && r.action == "buy") {
                        recommendations.push(AIRecommendation {
                            symbol: mapped_symbol,
                            action: "buy".to_string(),
                            quantity,
                            price,
                        });
                    } else {
                        println!("🔍 Skipping duplicate buy recommendation for {}", mapped_symbol);
                    }
                }
            }
        }
        
        // Parse sell recommendations - Updated patterns to match AI response format (case insensitive)
        let sell_patterns = [
            "(?i)sell all shares of ([A-Z]+)",
            "(?i)sell (\\d+) shares of ([A-Z]+) at \\$([0-9.]+)",
            "(?i)sell (\\d+) shares of ([A-Z]+)",
            "(?i)sell ([A-Z]+) at \\$([0-9.]+)",
            "(?i)sell ([A-Z]+) shares",
            "(?i)sell ([A-Z]+):",
            "(?i)\\d+\\. sell ([A-Z]+):",
        ];
        
        println!("🔍 Testing sell patterns on: {}", response_lower);
        
        for pattern in &sell_patterns {
            println!("🔍 Testing sell pattern: {}", pattern);
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(&response_lower) {
                    println!("🔍 Sell pattern matched! Captures: {:?}", cap);
                    // Use original response for symbol extraction to preserve case
                    let cap_original = regex.captures_iter(response).next();
                    let cap_to_use = cap_original.as_ref().unwrap_or(&cap);
                    println!("🔍 Processing sell capture with {} groups", cap_to_use.len());
                    let (quantity, symbol, price) = if cap_to_use.len() == 2 && pattern.contains("sell all shares") {
                        // Pattern: "sell all shares of TSLA"
                        if let Some(symbol) = cap_to_use.get(1) {
                            let price = self.extract_price_for_symbol(response, symbol.as_str()).unwrap_or(150.0);
                            (1000, symbol.as_str(), price) // Large quantity for "all shares"
                        } else {
                            continue;
                        }
                    } else if cap_to_use.len() == 2 {
                        // Pattern: "sell AAPL:" or "1. sell AAPL:" (numbered list)
                        println!("🔍 Processing 2-group sell pattern");
                        if let Some(symbol) = cap_to_use.get(1) {
                            println!("🔍 Detected symbol-only sell pattern: '{}'", symbol.as_str());
                            let price = self.extract_price_for_symbol(response, symbol.as_str()).unwrap_or(150.0);
                            (100, symbol.as_str(), price) // Default quantity
                        } else {
                            continue;
                        }
                    } else if cap_to_use.len() == 4 {
                        // Pattern: "sell 100 shares of AAPL at $150.62"
                        if let (Some(quantity_str), Some(symbol), Some(price_str)) = (cap_to_use.get(1), cap_to_use.get(2), cap_to_use.get(3)) {
                            if let Ok(quantity) = quantity_str.as_str().parse::<i32>() {
                                if let Ok(price) = price_str.as_str().parse::<f64>() {
                                    (quantity, symbol.as_str(), price)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else if cap_to_use.len() == 3 {
                        // Pattern: "sell 100 shares of AAPL" or "sell AAPL at $150.62" or "1. sell AAPL:"
                        println!("🔍 Processing 3-group sell pattern");
                        if let (Some(first), Some(second)) = (cap_to_use.get(1), cap_to_use.get(2)) {
                            if first.as_str().chars().all(|c| c.is_ascii_digit()) {
                                // Pattern: "sell 100 shares of AAPL" or "1. sell AAPL:"
                                if first.as_str().len() <= 2 && first.as_str().parse::<i32>().is_ok() {
                                    // Pattern: "1. sell AAPL:" (numbered list)
                                    let price = self.extract_price_for_symbol(response, second.as_str()).unwrap_or(150.0);
                                    (100, second.as_str(), price) // Default quantity
                                } else {
                                    // Pattern: "sell 100 shares of AAPL"
                                    if let Ok(quantity) = first.as_str().parse::<i32>() {
                                        let price = self.extract_price_for_symbol(response, second.as_str()).unwrap_or(150.0);
                                        (quantity, second.as_str(), price)
                                    } else {
                                        continue;
                                    }
                                }
                            } else {
                                // Pattern: "sell AAPL at $150.62" or "sell AAPL:"
                                let price = self.extract_price_for_symbol(response, first.as_str()).unwrap_or(150.0);
                                (100, first.as_str(), price) // Default quantity
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };
                    
                    let mapped_symbol = self.map_company_to_symbol(symbol);
                    println!("🔍 Adding sell recommendation: {} {} shares at ${}", mapped_symbol, quantity, price);
                    
                    // Check for duplicates before adding
                    if !recommendations.iter().any(|r: &AIRecommendation| r.symbol == mapped_symbol && r.action == "sell") {
                        recommendations.push(AIRecommendation {
                            symbol: mapped_symbol,
                            action: "sell".to_string(),
                            quantity,
                            price,
                        });
                    } else {
                        println!("🔍 Skipping duplicate sell recommendation for {}", mapped_symbol);
                    }
                }
            }
        }
        
        recommendations
    }

    /// Extract price for a symbol from the AI response
    fn extract_price_for_symbol(&self, response: &str, symbol: &str) -> Option<f64> {
        let response_lower = response.to_lowercase();
        let symbol_lower = symbol.to_lowercase();
        
        // Look for price patterns like "$150.62" near the symbol
        let price_pattern = format!(r"\${}([0-9.]+)", symbol_lower);
        if let Ok(regex) = regex::Regex::new(&price_pattern) {
            if let Some(cap) = regex.captures(&response_lower) {
                if let Some(price_str) = cap.get(1) {
                    return price_str.as_str().parse::<f64>().ok();
                }
            }
        }
        
        // Look for general price patterns near the symbol
        let general_price_pattern = format!(r"{}\s*\$([0-9.]+)", symbol_lower);
        if let Ok(regex) = regex::Regex::new(&general_price_pattern) {
            if let Some(cap) = regex.captures(&response_lower) {
                if let Some(price_str) = cap.get(1) {
                    return price_str.as_str().parse::<f64>().ok();
                }
            }
        }
        
        None
    }

    /// Execute trades based on AI recommendations
    async fn execute_trades(&self) -> Result<()> {
        #[derive(Debug)]
        struct ExecutedTrade {
            symbol: String,
            action: String,
            quantity: i32,
            price: f64,
        }
        
        let mut executed_trades = Vec::new();
        println!("🎯 Executing trades based on AI recommendations");
        
        // Check if we have AI analysis results
        let ai_analysis_file = "trading_portfolio/ai_analysis_report.json";
        if !std::path::Path::new(ai_analysis_file).exists() {
            println!("⚠️ No AI analysis found. Skipping trade execution.");
            return Ok(());
        }

        // Read AI analysis results
        let content = tokio::fs::read_to_string(ai_analysis_file).await?;
        println!("🔍 File content length: {} characters", content.len());
        
        let analysis: Value = serde_json::from_str(&content)?;
        println!("🔍 JSON parsed successfully");
        
        // Debug: Print available fields in the analysis
        println!("🔍 Available fields in AI analysis:");
        if let Some(obj) = analysis.as_object() {
            for (key, _) in obj {
                println!("   - {}", key);
            }
        }
        
        // Debug: Check if trading_recommendations exists
        if let Some(trading_recs) = analysis.get("trading_recommendations") {
            println!("🔍 trading_recommendations found: {:?}", trading_recs);
                        } else {
            println!("🔍 trading_recommendations NOT found in analysis");
        }
        
        // Extract trading recommendations from AI response (natural language)
        if let Some(ollama_response) = analysis["ollama_response"].as_str() {
            println!("🔍 Found ollama_response: {}", ollama_response);
            // Parse AI recommendations from natural language
            let recommendations = self.parse_ai_recommendations(ollama_response);
            println!("🔍 Parsed {} recommendations from ollama_response", recommendations.len());
            
            for recommendation in recommendations {
                println!("🔍 Processing recommendation: {:?}", recommendation);
                if recommendation.action != "hold" && recommendation.action != "skip" {
                    // Adjust quantity based on available buying power and portfolio protection
                    let adjusted_quantity = if recommendation.action == "buy" {
                        // Calculate safe position size based on available cash and portfolio protection
                        self.calculate_safe_position_size(&recommendation.symbol, recommendation.price).await.unwrap_or(1)
                    } else {
                        recommendation.quantity
                    };
                                    
                                    // Execute the trade
                    match self.execute_single_trade(&recommendation.symbol, &recommendation.action, adjusted_quantity, recommendation.price).await {
                        Ok(true) => {
                            executed_trades.push(ExecutedTrade {
                                symbol: recommendation.symbol.clone(),
                                action: recommendation.action.clone(),
                                quantity: adjusted_quantity,
                                price: recommendation.price,
                            });
                        }
                        Ok(false) => {
                            println!("⚠️ Trade blocked: {} {} shares of {} at ${:.2}", 
                                recommendation.action, adjusted_quantity, recommendation.symbol, recommendation.price);
                        }
                        Err(e) => {
                            println!("❌ Trade failed: {} {} shares of {} at ${:.2} - Error: {}", 
                                recommendation.action, adjusted_quantity, recommendation.symbol, recommendation.price, e);
                        }
                    }
                }
            }
        } else {
            println!("🔍 ollama_response field not found or not a string");
        }
        
        // Extract structured trading recommendations
        println!("🔍 Looking for trading_recommendations in AI analysis...");
        if let Some(trading_recommendations) = analysis["trading_recommendations"].as_array() {
            println!("📊 Found {} structured trading recommendations", trading_recommendations.len());
            
            for recommendation in trading_recommendations {
                println!("🔍 Processing recommendation: {:?}", recommendation);
                if let Some(symbol) = recommendation["symbol"].as_str() {
                    if let Some(action) = recommendation["action"].as_str() {
                        println!("🔍 Found symbol: {}, action: {}", symbol, action);
                        if action == "BUY" || action == "SELL" {
                            println!("🔍 Action is BUY/SELL, proceeding with trade execution...");
                            // Get target price or use current market price
                            let target_price = recommendation["target_price"].as_f64();
                            let stop_loss = recommendation["stop_loss"].as_f64();
                            let confidence = recommendation["confidence"].as_f64().unwrap_or(0.5);
                            
                            // Get real available cash from account
                            println!("🔍 Getting account data for position sizing...");
                            let account_data = self.get_real_account_data().await?;
                            let available_funds = account_data["cash"]
                                .as_str()
                                .unwrap_or("0")
                                .parse::<f64>()
                                .unwrap_or(0.0);
                            println!("🔍 Available funds: ${:.2}", available_funds);
                            
                            // Calculate position size based on confidence and available funds
                            let position_size_pct = recommendation["position_size"].as_f64().unwrap_or(0.1);
                            let allocation_amount = available_funds * position_size_pct * confidence;
                            println!("🔍 Position size calculation: ${:.2} × {:.2} × {:.2} = ${:.2}", 
                                available_funds, position_size_pct, confidence, allocation_amount);
                            
                            // Use target price if available, otherwise use current market price
                            let execution_price = target_price.unwrap_or(150.0); // Default fallback
                            let mut quantity = (allocation_amount / execution_price) as i32;
                            println!("🔍 Quantity calculation: ${:.2} ÷ ${:.2} = {} shares", 
                                allocation_amount, execution_price, quantity);
                            
                            // If we have enough cash for at least 1 share, ensure we buy at least 1
                            if quantity == 0 && allocation_amount >= execution_price {
                                quantity = 1;
                                println!("🔍 Adjusted quantity to 1 share (minimum viable trade)");
                            }
                            
                            println!("🔍 Checking if quantity > 0: {} > 0 = {}", quantity, quantity > 0);
                            if quantity > 0 {
                                let action_type = if action == "BUY" { "buy" } else { "sell" };
                                
                                println!("🎯 Executing {}: {} shares of {} at ${:.2} (confidence: {:.2})", 
                                    action_type, quantity, symbol, execution_price, confidence);
                                println!("   💰 Allocation: ${:.2} ({}% of ${:.2} available cash)", 
                                    allocation_amount, position_size_pct * 100.0, available_funds);
                                println!("   📊 Position Size: {} shares × ${:.2} = ${:.2}", 
                                    quantity, execution_price, quantity as f64 * execution_price);
                                
                                match self.execute_single_trade(symbol, action_type, quantity, execution_price).await {
                                    Ok(true) => {
                                        executed_trades.push(ExecutedTrade {
                                            symbol: symbol.to_string(),
                                            action: action_type.to_string(),
                                            quantity,
                                            price: execution_price,
                                        });
                                    }
                                    Ok(false) => {
                                        println!("⚠️ Trade blocked: {} {} shares of {} at ${:.2}", 
                                            action_type, quantity, symbol, execution_price);
                                    }
                                    Err(e) => {
                                        println!("❌ Trade failed: {} {} shares of {} at ${:.2} - Error: {}", 
                                            action_type, quantity, symbol, execution_price, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Also check for structured recommendations in strategy_recommendations (legacy)
        if let Some(strategy_data) = analysis["strategy_recommendations"].as_object() {
            if let Some(recommendations) = strategy_data["recommendations"].as_array() {
                for recommendation in recommendations {
                    if let Some(symbol) = recommendation["symbol"].as_str() {
                        if let Some(action) = recommendation["action"].as_str() {
                            if action != "SKIP" && action != "HOLD" {
                                // Get current price from market data
                                if let Some(market_data) = analysis["market_data"]["symbols"].as_object() {
                                    if let Some(symbol_data) = market_data.get(symbol) {
                                        if let Some(price) = symbol_data["price"].as_f64() {
                                            // Calculate quantity based on available capital
                                            let available_capital = recommendation["available_capital"].as_f64().unwrap_or(10000.0);
                                            let quantity = (available_capital * 0.1 / price) as i32; // Use 10% of available capital
                                            
                                            let action_type = if action == "BUY" { "buy" } else { "sell" };
                                            
                                            match self.execute_single_trade(symbol, action_type, quantity, price).await {
                                                Ok(true) => {
                                                    executed_trades.push(ExecutedTrade {
                                                        symbol: symbol.to_string(),
                                                        action: action_type.to_string(),
                                                        quantity,
                                                        price,
                                                    });
                                                }
                                                Ok(false) => {
                                                    println!("⚠️ Trade blocked: {} {} shares of {} at ${:.2}", 
                                                        action_type, quantity, symbol, price);
                                                }
                                                Err(e) => {
                                                    println!("❌ Trade failed: {} {} shares of {} at ${:.2} - Error: {}", 
                                                        action_type, quantity, symbol, price, e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Display trade execution summary
        if !executed_trades.is_empty() {
            println!("📊 Trade Execution Summary:");
            println!("   📈 Trades Executed: {}", executed_trades.len());
            for (i, trade) in executed_trades.iter().enumerate() {
                let trade_type = if trade.action == "sell" {
                    "💰 LIQUIDATION"
                } else {
                    "📈 NEW POSITION"
                };
                println!("   {}. {} {} shares of {} at ${:.2} ({})", 
                    i + 1, trade.action, trade.quantity, trade.symbol, trade.price, trade_type);
            }
        } else {
            println!("📊 No trades executed - All recommendations were HOLD or SKIP");
            println!("🔍 Debug: trading_recommendations field not found or empty in AI analysis");
        }
        
        println!("✅ Trade execution completed");
        Ok(())
    }

    /// Calculate safe position size based on available cash and portfolio protection
    async fn calculate_safe_position_size(&self, symbol: &str, price: f64) -> Result<i32> {
        // Get real-time account data from Alpaca API
        let account_data = self.get_real_account_data().await?;
        
        // Get current cash from real account data
        let current_cash = account_data["cash"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        // Get starting portfolio value for protection
        let starting_value = self.get_starting_portfolio_value().await?;
        
        // Calculate maximum safe position size
        // Use only 5% of available cash to be conservative
        let max_allocation = current_cash * 0.05;
        let max_shares = (max_allocation / price) as i32;
        
        // Ensure we don't go below starting portfolio value
        let protection_buffer = (current_cash - starting_value).max(0.0);
        let protected_allocation = protection_buffer * 0.1; // Use only 10% of protection buffer
        let protected_shares = (protected_allocation / price) as i32;
        
        // Use the smaller of the two calculations
        let safe_shares = std::cmp::min(max_shares, protected_shares).max(1);
        
        println!("🛡️ Safe Position Size for {}: {} shares (Max: {}, Protected: {}, Cash: ${:.2})", 
            symbol, safe_shares, max_shares, protected_shares, current_cash);
        
        Ok(safe_shares)
    }

    /// Check portfolio protection to ensure portfolio doesn't go below starting value
    async fn check_portfolio_protection(&self, action: &str, quantity: i32, price: f64) -> Result<()> {
        // Get real-time account data from Alpaca API
        let account_data = self.get_real_account_data().await?;
        
        // Get starting portfolio value
        let starting_value = self.get_starting_portfolio_value().await?;
        
        // Get current portfolio value from real account data
        let current_value = account_data["portfolio_value"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        // Get current cash from real account data
        let current_cash = account_data["cash"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        // Calculate trade impact
        let trade_value = quantity.abs() as f64 * price;
        
        // For buy orders: Check if we have enough cash
        if action.to_lowercase().contains("buy") {
            if trade_value > current_cash {
                return Err(anyhow::anyhow!("Insufficient cash: Need ${:.2}, have ${:.2}", trade_value, current_cash));
            }
            
            // Buy orders don't reduce portfolio value - they convert cash to positions
            // Portfolio protection doesn't apply to buy orders since they don't reduce total value
            println!("🛡️ Portfolio Protection: Buy order approved - Using ${:.2} of ${:.2} available cash", 
                trade_value, current_cash);
        }
        
        // For sell orders: Check if selling would reduce portfolio below starting value
        if action.to_lowercase().contains("sell") {
            // Estimate portfolio value after selling (simplified - assumes we're selling existing positions)
            let projected_portfolio_value = current_value - trade_value;
            if projected_portfolio_value < starting_value {
                return Err(anyhow::anyhow!("Portfolio protection: Selling would reduce portfolio to ${:.2}, below starting value ${:.2}", 
                    projected_portfolio_value, starting_value));
            }
        }
        
        println!("🛡️ Portfolio Protection: Trade approved - Current: ${:.2}, Starting: ${:.2}", 
            current_value, starting_value);
        Ok(())
    }

    /// Execute a single trade using Alpaca API
    async fn execute_single_trade(&self, symbol: &str, action: &str, quantity: i32, price: f64) -> Result<bool> {
        println!("📈 Executing {} order: {} {} shares of {} at ${:.2}", 
                 action, quantity, symbol, symbol, price);
        
        // Portfolio Protection: Check if trade would reduce portfolio below starting value
        if let Err(e) = self.check_portfolio_protection(action, quantity, price).await {
            println!("🛡️ Portfolio Protection: {}", e);
            return Ok(false);
        }
        
        // Check if market is open (for paper trading, we can trade anytime)
        if self.trading_mode == "live" {
            if !self.is_market_open().await? {
                println!("⚠️ Market is closed. Skipping order execution.");
                return Ok(false);
            }
        }
        
        // Execute real orders in paper trading mode
        if self.trading_mode == "paper" {
            println!("📝 Paper trading mode - Executing real paper trading orders");
            // Continue with order execution
        }

        // For live trading, execute real orders
        let client = reqwest::Client::new();
        let api_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || secret_key.is_empty() {
            println!("⚠️ Alpaca API keys not configured. Simulating order execution.");
            return Ok(true); // Return true for simulated execution
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
            "type": "market",
            "time_in_force": "day"
        });

        println!("📤 Sending order to Alpaca:");
        println!("   Symbol: {}", symbol);
        println!("   Quantity: {} shares", quantity.abs());
        println!("   Side: {}", if action.to_lowercase().contains("buy") { "buy" } else { "sell" });
        println!("   Type: market");
        println!("   Estimated Value: ${:.2}", quantity.abs() as f64 * price);

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
            Ok(true)
        } else {
            let error_text = response.text().await?;
            println!("❌ Order execution failed: {}", error_text);
            Ok(false)
        }
    }

    /// Monitor positions and liquidate at 0.25% profit
    async fn monitor_and_liquidate_positions(&self) -> Result<()> {
        const PROFIT_TARGET_PERCENTAGE: f64 = 0.25; // 0.25% profit target
        println!("🔍 Monitoring positions for {:.2}% profit liquidation...", PROFIT_TARGET_PERCENTAGE);
        
        // Get current positions from Alpaca API
        let positions = self.get_current_positions().await?;
        
        if positions.is_empty() {
            println!("📊 No positions to monitor");
            return Ok(());
        }
        
        println!("📊 Monitoring {} positions", positions.len());
        
        for position in positions {
            let symbol = position["symbol"].as_str().unwrap_or("UNKNOWN");
            let qty = position["qty"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let market_value = position["market_value"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let cost_basis = position["cost_basis"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let unrealized_pl = position["unrealized_pl"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            
            // Only monitor long positions (positive quantity)
            if qty <= 0.0 {
                continue;
            }
            
            // Calculate profit percentage
            let profit_percentage = if cost_basis > 0.0 {
                (unrealized_pl / cost_basis) * 100.0
            } else {
                0.0
            };
            
            println!("📈 Position {}: {} shares, P&L: ${:.2} ({:.3}%)", 
                symbol, qty, unrealized_pl, profit_percentage);
            
            // Liquidate if profit >= target percentage
            if profit_percentage >= PROFIT_TARGET_PERCENTAGE {
                println!("💰 PROFIT TARGET HIT! Liquidating {} at {:.3}% profit", symbol, profit_percentage);
                
                // Execute liquidation order
                if self.execute_single_trade(symbol, "sell", qty as i32, market_value / qty).await? {
                    println!("✅ Successfully liquidated {} shares of {} at {:.3}% profit", 
                        qty, symbol, profit_percentage);
                } else {
                    println!("❌ Failed to liquidate {} shares of {}", qty, symbol);
                }
            } else {
                println!("⏳ {} not ready for liquidation (need {:.2}%, currently {:.3}%)", 
                    symbol, PROFIT_TARGET_PERCENTAGE, profit_percentage);
            }
        }

        Ok(())
    }

    /// Get real-time account data from Alpaca API
    async fn get_real_account_data(&self) -> Result<Value> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || secret_key.is_empty() {
            println!("⚠️ Alpaca API keys not configured. Using simulated account data.");
            // Return simulated data structure
            return Ok(serde_json::json!({
                "portfolio_value": "100000",
                "cash": "100000", 
                "equity": "100000"
            }));
        }

        let base_url = if self.trading_mode == "live" {
            "https://api.alpaca.markets"
        } else {
            "https://paper-api.alpaca.markets"
        };

        let response = client
            .get(&format!("{}/v2/account", base_url))
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let account_data: Value = response.json().await?;
            println!("✅ Fetched real account data from Alpaca API");
            Ok(account_data)
        } else {
            let error_text = response.text().await?;
            println!("⚠️ Failed to get account data: {}", error_text);
            // Return simulated data as fallback
            Ok(serde_json::json!({
                "portfolio_value": "100000",
                "cash": "100000",
                "equity": "100000"
            }))
        }
    }

    /// Get starting portfolio value from static file
    async fn get_starting_portfolio_value(&self) -> Result<f64> {
        let portfolio_file = "trading_portfolio/trading_portfolio.json";
        if std::path::Path::new(portfolio_file).exists() {
            let content = tokio::fs::read_to_string(portfolio_file).await?;
            let data: Value = serde_json::from_str(&content)?;
            
            let starting_value = data["portfolio_summary"]["portfolio_value"]
                .as_f64()
                .unwrap_or(100000.0);
            
            Ok(starting_value)
        } else {
            Ok(100000.0) // Default starting value
        }
    }

    /// Get current positions from Alpaca API
    async fn get_current_positions(&self) -> Result<Vec<Value>> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || secret_key.is_empty() {
            println!("⚠️ Alpaca API keys not configured. Using simulated positions.");
            return Ok(vec![]); // Return empty positions for simulation
        }

        let base_url = if self.trading_mode == "live" {
            "https://api.alpaca.markets"
        } else {
            "https://paper-api.alpaca.markets"
        };

        let response = client
            .get(&format!("{}/v2/positions", base_url))
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let positions: Vec<Value> = response.json().await?;
            Ok(positions)
        } else {
            let error_text = response.text().await?;
            println!("⚠️ Failed to get positions: {}", error_text);
            Ok(vec![])
        }
    }

    /// Check if emergency stop is triggered (portfolio below 95% of starting value)
    async fn is_emergency_stop_triggered(&self) -> Result<bool> {
        // Get real-time account data from Alpaca API
        let account_data = self.get_real_account_data().await?;
        
        // Get starting and current portfolio values
        let starting_value = self.get_starting_portfolio_value().await?;
        
        let current_value = account_data["portfolio_value"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);
        
        // Emergency stop if portfolio is 5% below starting value
        let emergency_threshold = starting_value * 0.95;
        let is_emergency = current_value < emergency_threshold;
        
        if is_emergency {
            println!("🚨 Emergency Stop Check: Current ${:.2} < Threshold ${:.2}", 
                current_value, emergency_threshold);
        }
        
        Ok(is_emergency)
    }

    /// Display current portfolio status with protection monitoring
    async fn display_portfolio_status(&self) -> Result<()> {
        println!("📊 Current Portfolio Status:");
        
        // Get real-time account data from Alpaca API
        let account_data = self.get_real_account_data().await?;
        
        if let Some(account_info) = account_data.as_object() {
                let portfolio_value = account_info["portfolio_value"].as_str().unwrap_or("0");
                let cash = account_info["cash"].as_str().unwrap_or("0");
                let equity = account_info["equity"].as_str().unwrap_or("0");
                
            // Get starting portfolio value for comparison (from static file)
            let starting_value = self.get_starting_portfolio_value().await?;
            
            let current_value = portfolio_value.parse::<f64>().unwrap_or(0.0);
            let current_cash = cash.parse::<f64>().unwrap_or(0.0);
            
            // Calculate protection status
            let protection_status = if current_value >= starting_value {
                "🛡️ PROTECTED"
            } else {
                "⚠️ BELOW STARTING VALUE"
            };
            
            let performance = ((current_value - starting_value) / starting_value) * 100.0;
            
            println!("   💰 Portfolio Value: ${} ({})", portfolio_value, protection_status);
                println!("   💵 Cash: ${}", cash);
                println!("   📈 Equity: ${}", equity);
            println!("   🎯 Starting Value: ${:.2}", starting_value);
            println!("   📊 Performance: {:.2}%", performance);
            
            // Display current positions
            let positions = self.get_current_positions().await?;
            if !positions.is_empty() {
                println!("   📋 Active Positions:");
                for position in positions {
                    let symbol = position["symbol"].as_str().unwrap_or("UNKNOWN");
                    let qty = position["qty"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    let unrealized_pl = position["unrealized_pl"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    let cost_basis = position["cost_basis"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    
                    if qty > 0.0 {
                        let profit_percentage = if cost_basis > 0.0 {
                            (unrealized_pl / cost_basis) * 100.0
                        } else {
                            0.0
                        };
                        
                        let status = if profit_percentage >= 0.25 {
                            "💰 READY TO LIQUIDATE"
                        } else {
                            "⏳ Monitoring"
                        };
                        
                        println!("     • {}: {} shares, P&L: ${:.2} ({:.3}%) {}", 
                            symbol, qty, unrealized_pl, profit_percentage, status);
                    }
                }
            } else {
                println!("   📋 No active positions");
            }
            
            // Emergency stop if portfolio is significantly below starting value
            if current_value < starting_value * 0.95 {
                println!("🚨 EMERGENCY STOP: Portfolio is 5% below starting value!");
                println!("   Current: ${:.2}, Starting: ${:.2}", current_value, starting_value);
                println!("   Trading will be suspended until portfolio recovers.");
            }
        } else {
            println!("⚠️ Could not fetch account data. Using fallback values.");
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
        let api_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
        let secret_key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
        
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
    
    /// Create sample JSON files for AI analysis
    async fn create_sample_json_files(&self) -> Result<()> {
        println!("📝 Creating sample JSON files for AI analysis...");
        
        // Create trading_portfolio.json
        let trading_portfolio = serde_json::json!({
            "portfolio_name": "Trading Portfolio",
            "data_source": "alpaca_paper_trading",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "portfolio_summary": {
                "total_assets": 70,
                "tradable_assets": 70,
                "marginable_assets": 70,
                "shortable_assets": 69,
                "current_positions": 0,
                "portfolio_value": 100000.0,
                "cash": 100000.0,
                "equity": 100000.0,
                "buying_power": 100000.0
            },
            "trading_account": {
                "account_info": {
                    "account_blocked": false,
                    "account_number": "PA3HC1LKUQJ8",
                    "accrued_fees": "0",
                    "cash": "100000",
                    "created_at": "2025-04-14T17:31:37.243089Z",
                    "crypto_status": "ACTIVE",
                    "currency": "USD",
                    "daytrade_count": 0,
                    "daytrading_buying_power": "0",
                    "equity": "100000",
                    "id": "2fa6408c-f6c6-4956-a838-ac49a65f5379",
                    "initial_margin": "0",
                    "last_equity": "100000",
                    "long_market_value": "0",
                    "maintenance_margin": "0",
                    "multiplier": "2",
                    "non_marginable_buying_power": "100000",
                    "pattern_day_trader": false,
                    "pending_transfer_in": "0",
                    "pending_transfer_out": "0",
                    "portfolio_value": "100000",
                    "regt_buying_power": "200000",
                    "short_market_value": "0",
                    "shorting_enabled": true,
                    "status": "ACTIVE",
                    "trade_suspended_by_user": false,
                    "trading_blocked": false,
                    "transfers_blocked": false
                },
                "market_status": {
                    "current_time": chrono::Utc::now().to_rfc3339(),
                    "is_open": false,
                    "market_timezone": "America/New_York",
                    "next_close": "2025-08-31T20:00:00+00:00",
                    "next_open": "2025-08-31T13:30:00+00:00",
                    "trading_hours": "9:30 AM - 4:00 PM ET (Monday-Friday)"
                }
            },
            "asset_universe": [
                {
                    "id": "b28f4066-5c6d-479b-a2af-85dc1a8f16fb",
                    "class": "us_equity",
                    "exchange": "ARCA",
                    "symbol": "SPY",
                    "name": "SPDR S&P 500 ETF Trust",
                    "status": "active",
                    "tradable": true,
                    "marginable": true,
                    "shortable": true,
                    "easy_to_borrow": true,
                    "fractionable": true
                },
                {
                    "id": "2d9e926c-e17c-47c3-ad8c-26c7a594e48f",
                    "class": "us_equity",
                    "exchange": "NASDAQ",
                    "symbol": "QQQ",
                    "name": "Invesco QQQ Trust, Series 1",
                    "status": "active",
                    "tradable": true,
                    "marginable": true,
                    "shortable": true,
                    "easy_to_borrow": true,
                    "fractionable": true
                },
                {
                    "id": "ef2b4bb6-00eb-494a-ba4d-aa40dd9a701c",
                    "class": "us_equity",
                    "exchange": "ARCA",
                    "symbol": "IWM",
                    "name": "iShares Russell 2000 ETF",
                    "status": "active",
                    "tradable": true,
                    "marginable": true,
                    "shortable": true,
                    "easy_to_borrow": true,
                    "fractionable": true
                }
            ],
            "current_positions": [],
            "market_data": {
                "data_source": "alpaca_basic_plan",
                "historical_limit": "15_minutes",
                "last_update": chrono::Utc::now().to_rfc3339(),
                "update_interval": "30_seconds",
                "symbols": {
                    "AAPL": {
                        "exchange": "alpaca",
                        "high": 150.59,
                        "last_updated": chrono::Utc::now().to_rfc3339(),
                        "low": 149.17,
                        "open": 150.0,
                        "price": 150.62,
                        "source": "alpaca_rest_fallback",
                        "symbol": "AAPL",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "volume": 198.46
                    },
                    "SPY": {
                        "exchange": "alpaca",
                        "high": 453.62,
                        "last_updated": chrono::Utc::now().to_rfc3339(),
                        "low": 449.36,
                        "open": 450.0,
                        "price": 450.96,
                        "source": "alpaca_rest_fallback",
                        "symbol": "SPY",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "volume": 742.85
                    }
                }
            },
            "market_regime_analysis": {
                "current_regime": "LowVolatility",
                "confidence_score": 0.6,
                "volatility_regime": {
                    "regime_type": "Low",
                    "vix_equivalent": 9.02,
                    "volatility_percentile": 0.25,
                    "volatility_trend": "Decreasing"
                },
                "trend_strength": 0.56,
                "correlation_regime": {
                    "average_correlation": 0.6,
                    "correlation_trend": "Stable",
                    "diversification_benefit": 0.4
                },
                "regime_duration": 5,
                "regime_probability": 0.2,
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "strategy_recommendations": {
                "strategy": "Enhanced Mathematical Portfolio Optimization",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "account_info": {
                    "cash": 100000.0,
                    "equity": 100000.0,
                    "buying_power": 100000.0,
                    "daytrade_count": 0,
                    "starting_portfolio_value": 100000.0,
                    "current_portfolio_value": 100000.0
                },
                "portfolio_protection": {
                    "enabled": true,
                    "protection_level": 100000.0,
                    "current_value": 100000.0,
                    "protection_triggered": false,
                    "risk_reduction_factor": 1.0
                },
                "recommendations": [
                    {
                        "action": "BUY",
                        "asset_type": "Stock",
                        "available_capital": 100000.0,
                        "reason": "Strong momentum and low volatility",
                        "required_capital": 5000.0,
                        "symbol": "SPY"
                    },
                    {
                        "action": "BUY",
                        "asset_type": "Stock",
                        "available_capital": 95000.0,
                        "reason": "Technology sector strength",
                        "required_capital": 3000.0,
                        "symbol": "AAPL"
                    }
                ],
                "risk_metrics": {
                    "portfolio_volatility": 0.15,
                    "portfolio_sharpe_ratio": 1.2,
                    "portfolio_sortino_ratio": 1.5,
                    "max_drawdown": 0.05,
                    "value_at_risk_95": 0.08,
                    "expected_shortfall": 0.10,
                    "risk_level": "MEDIUM",
                    "current_positions_count": 0,
                    "asset_universe_size": 70
                }
            }
        });
        
        // Write trading_portfolio.json
        let portfolio_file = "trading_portfolio/trading_portfolio.json";
        if let Err(e) = tokio::fs::write(portfolio_file, serde_json::to_string_pretty(&trading_portfolio)?).await {
            println!("⚠️  Failed to create {}: {}", portfolio_file, e);
        } else {
            println!("✅ Created {}", portfolio_file);
        }
        
        // Create asset_universe.json
        let asset_universe = serde_json::json!({
            "assets": [
                {
                    "symbol": "SPY",
                    "name": "SPDR S&P 500 ETF Trust",
                    "type": "ETF",
                    "sector": "Broad Market",
                    "price": 450.96,
                    "volume": 742.85,
                    "market_cap": 450000000000i64,
                    "tradable": true,
                    "marginable": true,
                    "shortable": true
                },
                {
                    "symbol": "QQQ",
                    "name": "Invesco QQQ Trust, Series 1",
                    "type": "ETF",
                    "sector": "Technology",
                    "price": 380.25,
                    "volume": 1250.30,
                    "market_cap": 200000000000i64,
                    "tradable": true,
                    "marginable": true,
                    "shortable": true
                },
                {
                    "symbol": "IWM",
                    "name": "iShares Russell 2000 ETF",
                    "type": "ETF",
                    "sector": "Small Cap",
                    "price": 185.50,
                    "volume": 890.45,
                    "market_cap": 50000000000i64,
                    "tradable": true,
                    "marginable": true,
                    "shortable": true
                },
                {
                    "symbol": "AAPL",
                    "name": "Apple Inc.",
                    "type": "Stock",
                    "sector": "Technology",
                    "price": 150.62,
                    "volume": 198.46,
                    "market_cap": 2500000000000i64,
                    "tradable": true,
                    "marginable": true,
                    "shortable": true
                },
                {
                    "symbol": "MSFT",
                    "name": "Microsoft Corporation",
                    "type": "Stock",
                    "sector": "Technology",
                    "price": 320.15,
                    "volume": 156.78,
                    "market_cap": 2400000000000i64,
                    "tradable": true,
                    "marginable": true,
                    "shortable": true
                }
            ],
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "total_assets": 5,
            "tradable_assets": 5,
            "marginable_assets": 5,
            "shortable_assets": 5
        });
        
        let asset_file = "trading_portfolio/asset_universe.json";
        if let Err(e) = tokio::fs::write(asset_file, serde_json::to_string_pretty(&asset_universe)?).await {
            println!("⚠️  Failed to create {}: {}", asset_file, e);
        } else {
            println!("✅ Created {}", asset_file);
        }
        
        // Create current_positions.json
        let current_positions = serde_json::json!({
            "positions": [],
            "total_positions": 0,
            "total_value": 0.0,
            "cash_balance": 100000.0,
            "buying_power": 100000.0,
            "last_updated": chrono::Utc::now().to_rfc3339()
        });
        
        let positions_file = "trading_portfolio/current_positions.json";
        if let Err(e) = tokio::fs::write(positions_file, serde_json::to_string_pretty(&current_positions)?).await {
            println!("⚠️  Failed to create {}: {}", positions_file, e);
        } else {
            println!("✅ Created {}", positions_file);
        }
        
        // Create portfolio_analysis.json
        let portfolio_analysis = serde_json::json!({
            "portfolio_summary": {
                "total_value": 100000.0,
                "cash_balance": 100000.0,
                "invested_amount": 0.0,
                "total_positions": 0,
                "unrealized_pnl": 0.0,
                "realized_pnl": 0.0
            },
            "risk_metrics": {
                "portfolio_volatility": 0.15,
                "sharpe_ratio": 1.2,
                "sortino_ratio": 1.5,
                "max_drawdown": 0.05,
                "value_at_risk_95": 0.08,
                "expected_shortfall": 0.10,
                "risk_level": "MEDIUM"
            },
            "performance_metrics": {
                "total_return": 0.0,
                "annualized_return": 0.0,
                "volatility": 0.15,
                "sharpe_ratio": 1.2,
                "max_drawdown": 0.05,
                "win_rate": 0.0,
                "profit_factor": 0.0
            },
            "asset_allocation": {
                "cash": 100.0,
                "stocks": 0.0,
                "etfs": 0.0,
                "options": 0.0,
                "crypto": 0.0
            },
            "last_updated": chrono::Utc::now().to_rfc3339()
        });
        
        let analysis_file = "trading_portfolio/portfolio_analysis.json";
        if let Err(e) = tokio::fs::write(analysis_file, serde_json::to_string_pretty(&portfolio_analysis)?).await {
            println!("⚠️  Failed to create {}: {}", analysis_file, e);
        } else {
            println!("✅ Created {}", analysis_file);
        }
        
        // Create ai_analysis_report.json
        let ai_analysis = serde_json::json!({
            "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            "model_used": "deepseek-r1:32b",
            "analysis_type": "portfolio_optimization",
            "market_regime": {
                "current_regime": "LowVolatility",
                "confidence_score": 0.75,
                "volatility_level": "Low",
                "trend_direction": "Sideways",
                "market_sentiment": "Neutral"
            },
            "portfolio_assessment": {
                "current_allocation": {
                    "cash": 100.0,
                    "stocks": 0.0,
                    "etfs": 0.0,
                    "options": 0.0,
                    "crypto": 0.0
                },
                "risk_level": "LOW",
                "diversification_score": 0.0,
                "concentration_risk": "HIGH",
                "cash_efficiency": 0.0
            },
            "trading_recommendations": [
                {
                    "symbol": "SPY",
                    "action": "BUY",
                    "confidence": 0.85,
                    "reasoning": "Strong momentum, low volatility environment, broad market exposure",
                    "target_price": 455.0,
                    "stop_loss": 445.0,
                    "position_size": 0.25,
                    "time_horizon": "Medium-term (3-6 months)",
                    "risk_level": "LOW"
                },
                {
                    "symbol": "QQQ",
                    "action": "BUY", 
                    "confidence": 0.78,
                    "reasoning": "Technology sector strength, growth potential, good liquidity",
                    "target_price": 385.0,
                    "stop_loss": 375.0,
                    "position_size": 0.20,
                    "time_horizon": "Medium-term (3-6 months)",
                    "risk_level": "MEDIUM"
                },
                {
                    "symbol": "AAPL",
                    "action": "BUY",
                    "confidence": 0.72,
                    "reasoning": "Strong fundamentals, stable earnings, defensive position",
                    "target_price": 155.0,
                    "stop_loss": 145.0,
                    "position_size": 0.15,
                    "time_horizon": "Long-term (6-12 months)",
                    "risk_level": "LOW"
                }
            ],
            "risk_management": {
                "max_position_size": 0.25,
                "max_portfolio_risk": 0.15,
                "stop_loss_strategy": "Trailing stops at 2% below entry",
                "position_sizing": "Kelly Criterion based",
                "rebalancing_frequency": "Monthly",
                "emergency_exit_trigger": "5% portfolio drawdown"
            },
            "market_opportunities": {
                "sector_rotation": {
                    "overweight": ["Technology", "Healthcare"],
                    "underweight": ["Energy", "Financials"],
                    "neutral": ["Consumer Discretionary", "Industrials"]
                },
                "style_preferences": {
                    "growth": 0.6,
                    "value": 0.3,
                    "momentum": 0.1
                },
                "geographic_allocation": {
                    "domestic": 0.8,
                    "international": 0.2
                }
            },
            "performance_metrics": {
                "expected_return": 0.08,
                "expected_volatility": 0.12,
                "sharpe_ratio": 0.67,
                "max_drawdown": 0.08,
                "var_95": 0.06,
                "expected_shortfall": 0.08
            },
            "execution_notes": {
                "entry_strategy": "Dollar-cost averaging over 3 days",
                "exit_strategy": "Trailing stops with profit taking at 15%",
                "monitoring_frequency": "Daily",
                "adjustment_triggers": [
                    "5% position loss",
                    "Market regime change",
                    "Earnings announcements",
                    "Fed policy changes"
                ]
            },
            "ai_confidence": {
                "overall_confidence": 0.78,
                "data_quality_score": 0.85,
                "model_performance": 0.82,
                "market_conditions": "Favorable",
                "recommendation_strength": "STRONG"
            }
        });
        
        let ai_analysis_file = "trading_portfolio/ai_analysis_report.json";
        if let Err(e) = tokio::fs::write(ai_analysis_file, serde_json::to_string_pretty(&ai_analysis)?).await {
            println!("⚠️  Failed to create {}: {}", ai_analysis_file, e);
        } else {
            println!("✅ Created {}", ai_analysis_file);
        }
        
        println!("✅ All sample JSON files created successfully!");
        println!();
        
        Ok(())
    }
}
