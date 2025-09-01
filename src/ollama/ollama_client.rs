use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use tokio::time::timeout;
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::ollama::ollama_receipt::OllamaReceipt;

// Connection pool configuration
const MAX_CONCURRENT_REQUESTS: usize = 10;
const CONNECTION_TIMEOUT: u64 = 15;  // Increased from 5 to 15 seconds
const REQUEST_TIMEOUT: u64 = 120;  // Increased from 30 to 120 seconds
const KEEP_ALIVE_DURATION: u64 = 60;
const MAX_IDLE_PER_HOST: usize = 20;

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    // Speed optimizations
    num_predict: i32,          // Limit max tokens for faster responses
    temperature: f32,          // Lower temp = faster, more focused responses
    top_k: i32,               // Reduce sampling space for speed
    top_p: f32,               // Nucleus sampling for efficiency
    
    // Performance optimizations
    num_ctx: i32,             // Context window size
    num_batch: i32,           // Batch size for processing
    num_thread: i32,          // Use more CPU threads
    repeat_penalty: f32,      // Prevent repetition
    
    // Response quality vs speed balance
    mirostat: i32,            // Use mirostat for consistent quality
    mirostat_eta: f32,        // Learning rate for mirostat
    mirostat_tau: f32,        // Target entropy for mirostat
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    response: String,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    semaphore: Arc<Semaphore>,
}

impl OllamaClient {
    pub fn new(base_url: &str, timeout_seconds: u64) -> Self {
        println!("🔧 Creating Ollama client with timeout: {} seconds", timeout_seconds);
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .pool_idle_timeout(Duration::from_secs(KEEP_ALIVE_DURATION))
            .pool_max_idle_per_host(MAX_IDLE_PER_HOST)
            .tcp_keepalive(Duration::from_secs(KEEP_ALIVE_DURATION))
            .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
            // Removed http2_prior_knowledge() for better compatibility
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
        }
    }
    
    // Create balanced options for good analysis quality at reasonable speed
    fn create_balanced_options() -> GenerateOptions {
        GenerateOptions {
            // BALANCED MODE: Better analysis quality at moderate speed (8-12s)
            num_predict: 200,          // Longer responses: ~150 words
            temperature: 0.3,          // Some creativity for better analysis
            top_k: 20,                // More diverse sampling for insights
            top_p: 0.9,               // Standard nucleus sampling
            
            // Balanced performance settings
            num_ctx: 2048,            // More context for complex reasoning
            num_batch: 16,            // Moderate batch size
            num_thread: -1,           // Use all CPU cores
            repeat_penalty: 1.1,      // Prevent repetition
            
            // Enable quality features
            mirostat: 2,              // Better quality control
            mirostat_eta: 0.1,        
            mirostat_tau: 5.0,
        }
    }

    // High-performance generate with connection pooling and concurrency control
    pub async fn generate_optimized(&self, model: &str, prompt: &str) -> Result<String> {
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|e| anyhow!("Semaphore error: {}", e))?;
        
        // Create optimized request
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaClient::create_default_options(),
        };
        
        println!("🧠 Using model: {}", model);
        let generate_url = format!("{}/api/generate", self.base_url);
        println!("🔗 Attempting to connect to: {}", generate_url);
        
        // Use timeout for the entire request
        let response_future = self.client.post(&generate_url)
            .json(&request)
            .send();
        
        match timeout(Duration::from_secs(REQUEST_TIMEOUT), response_future).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    let generate_response: GenerateResponse = response.json().await?;
                    Ok(generate_response.response)
                } else {
                    Err(anyhow!("HTTP error: {}", response.status()))
                }
            }
            Ok(Err(e)) => {
                println!("❌ HTTP request failed: {}", e);
                Err(anyhow!("Request failed: {}", e))
            }
            Err(_) => {
                println!("⏰ Request timeout after {} seconds (REQUEST_TIMEOUT: {}s). Consider increasing REQUEST_TIMEOUT or checking Ollama server performance.", REQUEST_TIMEOUT, REQUEST_TIMEOUT);
                Err(anyhow!("Request timeout after {} seconds. Check Ollama server status and consider increasing timeout values.", REQUEST_TIMEOUT))
            }
        }
    }
    
    // Ultra-fast options for maximum performance (simplified for compatibility)
    fn create_ultra_fast_options() -> GenerateOptions {
        GenerateOptions {
            // PERFORMANCE OPTIMIZED: Faster responses for trading analysis (5-15s)
            num_predict: 300,          // Balanced response length
            temperature: 0.2,          // More focused responses
            top_k: 15,                 // Balanced sampling
            top_p: 0.85,               // Efficient nucleus sampling
            
            // Performance optimizations for speed
            num_ctx: 1024,             // Smaller context for speed
            num_batch: 8,              // Smaller batch for faster processing
            num_thread: -1,            // Use all CPU cores
            repeat_penalty: 1.05,      // Minimal repetition penalty
            
            // Disable advanced features for speed
            mirostat: 0,               // Disable for speed
            mirostat_eta: 0.0,        
            mirostat_tau: 0.0,
        }
    }
    
    // Default options (like ollama run)
    fn create_default_options() -> GenerateOptions {
        GenerateOptions {
            // DEFAULT MODE: Like ollama run (compatible with all models)
            num_predict: 128,          // Shorter responses for compatibility
            temperature: 0.7,          // Standard temperature
            top_k: 40,                // Standard sampling
            top_p: 0.9,               // Standard nucleus sampling
            
            // Standard settings
            num_ctx: 2048,            // Standard context window
            num_batch: 512,           // Standard batch size
            num_thread: -1,           // Use all CPU cores
            repeat_penalty: 1.1,      // Standard repetition penalty
            
            // Standard features
            mirostat: 0,              // Disabled by default
            mirostat_eta: 0.0,        
            mirostat_tau: 0.0,
        }
    }



    pub async fn generate_with_timing(&self, model: &str, prompt: &str) -> Result<(String, OllamaReceipt)> {
        let (mut receipt, start_instant) = OllamaReceipt::new(
            "Generate".to_string(),
            model.to_string(),
            prompt.len(),
        );

        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false, // Non-streaming for compatibility
            options: Self::create_balanced_options(),
        };

        println!("Sending request to: {}", url);
        
        let result = async {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Ollama API returned error status: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                ));
            }

            let generate_response: GenerateResponse = response
                .json()
                .await
                .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

            if let Some(error) = generate_response.error {
                return Err(anyhow!("Ollama returned error: {}", error));
            }

            Ok(generate_response.response)
        }.await;

        match result {
            Ok(response) => {
                receipt.finish(start_instant, response.len(), true, None);
                Ok((response, receipt))
            }
            Err(e) => {
                receipt.finish(start_instant, 0, false, Some(e.to_string()));
                Err(e)
            }
        }
    }


    pub async fn generate_stream_with_timing(&self, model: &str, prompt: &str) -> Result<(Vec<String>, OllamaReceipt)> {
        let (mut receipt, start_instant) = OllamaReceipt::new(
            "StreamGenerate".to_string(),
            model.to_string(),
            prompt.len(),
        );

        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options: Self::create_balanced_options(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Ollama API returned error status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let mut text_chunks = Vec::new();
        
        // For now, use the non-streaming approach since bytes_stream is not available
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response text: {}", e))?;
        
        // Split the response into chunks (simulating streaming)
        let chunks: Vec<&str> = response_text.split('\n').collect();
        for chunk in chunks {
            if !chunk.trim().is_empty() {
                text_chunks.push(chunk.to_string());
            }
        }
        
        let total_chars: usize = text_chunks.iter().map(|s| s.len()).sum();
        receipt.finish(start_instant, total_chars, true, None);
        Ok((text_chunks, receipt))
    }

    /// Analyze portfolio data with Ollama
    pub async fn analyze_portfolio(&self, model: &str, portfolio_data: &str) -> Result<String> {
        let prompt = format!(
            "Analyze this trading portfolio data and provide insights:\n\n{}\n\nPlease provide:\n1. Portfolio Summary\n2. Risk Assessment\n3. Performance Analysis\n4. Trading Recommendations\n5. Market Insights",
            portfolio_data
        );

        self.generate_optimized(model, &prompt).await
    }

    /// Stream portfolio analysis with real-time updates
    pub async fn stream_portfolio_analysis(&self, model: &str, portfolio_data: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Stream a real-time analysis of this trading portfolio:\n\n{}\n\nProvide continuous insights on:\n- Portfolio performance\n- Risk metrics\n- Market conditions\n- Trading opportunities\n- Position recommendations",
            portfolio_data
        );

        let (chunks, _receipt) = self.generate_stream_with_timing(model, &prompt).await?;
        Ok(chunks)
    }
}