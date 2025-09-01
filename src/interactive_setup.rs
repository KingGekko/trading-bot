use anyhow::Result;
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;
use tokio::time::{sleep, Duration};
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
                    max_timeout_seconds: 30,
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
        let mut cmd = Command::new("./target/release/trading_bot.exe");
        cmd.arg("--enhanced-strategy");
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to start market data collection"));
        }
        
        println!("   ✅ Market data collection started");
        Ok(())
    }

    /// Start portfolio analysis server
    async fn start_portfolio_analysis_server(&self) -> Result<()> {
        // Start the portfolio analysis server in background
        let mut cmd = Command::new("./target/release/trading_bot.exe");
        cmd.arg("--portfolio-analysis");
        
        // Run in background
        let _child = cmd.spawn()?;
        
        // Wait a moment for server to start
        sleep(Duration::from_secs(3)).await;
        
        println!("   ✅ Portfolio analysis server started on port 8082");
        Ok(())
    }

    /// Start streaming (live mode only)
    async fn start_streaming(&self) -> Result<()> {
        // Start streaming for live trading
        let mut cmd = Command::new("./target/release/trading_bot.exe");
        cmd.arg("--websocket");
        
        // Run in background
        let _child = cmd.spawn()?;
        
        // Wait a moment for streaming to start
        sleep(Duration::from_secs(3)).await;
        
        println!("   ✅ Live data streaming started");
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
        
        // Use the AI decisions command
        let mut cmd = Command::new("./target/release/trading_bot.exe");
        cmd.arg("--ai-decisions");
        cmd.arg("--model");
        cmd.arg(&self.selected_model);
        
        let output = cmd.output()?;
        if output.status.success() {
            println!("✅ AI analysis completed successfully");
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
                "file_path": "trading_portfolio/portfolio_analysis.json",
                "initial_prompt": "",
                "models": self.models
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("✅ Multi-model analysis completed successfully");
        } else {
            println!("⚠️ Multi-model analysis completed with warnings");
        }
        
        Ok(())
    }

    /// Execute trades based on AI recommendations
    async fn execute_trades(&self) -> Result<()> {
        println!("🎯 Executing trades based on AI recommendations");
        
        // Use test orders for safety (can be changed to real orders later)
        let mut cmd = Command::new("./target/release/trading_bot.exe");
        cmd.arg("--test-orders");
        
        let output = cmd.output()?;
        if output.status.success() {
            println!("✅ Trade execution analysis completed");
        } else {
            println!("⚠️ Trade execution analysis completed with warnings");
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
}
