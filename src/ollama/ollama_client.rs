use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use tokio::time::timeout;
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::ollama::ollama_receipt::OllamaReceipt;

// Connection pool configuration
const MAX_CONCURRENT_REQUESTS: usize = 3;  // Reduced to prevent overload
const CONNECTION_TIMEOUT: u64 = 60;  // Increased for better reliability
const REQUEST_TIMEOUT: u64 = 180;  // Reduced to prevent long timeouts
const KEEP_ALIVE_DURATION: u64 = 60;  // Reduced for better connection management
const MAX_IDLE_PER_HOST: usize = 5;  // Reduced to prevent memory issues

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
        // Check if Ollama is running first
        if let Err(e) = self.check_ollama_status().await {
            return Err(anyhow!("Ollama server is not running or not accessible: {}", e));
        }
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|e| anyhow!("Semaphore error: {}", e))?;
        
        // Try streaming first, fallback to non-streaming if needed
        match self.generate_with_streaming(model, prompt).await {
            Ok(response) => Ok(response),
            Err(stream_error) => {
                println!("⚠️ Streaming failed, trying non-streaming mode: {}", stream_error);
                self.generate_without_streaming(model, prompt).await
            }
        }
    }
    
    // Check if Ollama server is running
    async fn check_ollama_status(&self) -> Result<()> {
        let status_url = format!("{}/api/tags", self.base_url);
        
        match timeout(Duration::from_secs(10), self.client.get(&status_url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!("Ollama server returned status: {}", response.status()))
                }
            }
            Ok(Err(e)) => Err(anyhow!("Failed to connect to Ollama: {}", e)),
            Err(_) => Err(anyhow!("Timeout connecting to Ollama server")),
        }
    }
    
    // Generate with streaming for better performance and timeout handling
    async fn generate_with_streaming(&self, model: &str, prompt: &str) -> Result<String> {
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true, // Enable streaming for better timeout handling
            options: OllamaClient::create_ultra_fast_options(), // Use faster options for streaming
        };
        
        println!("🧠 Using model: {} (streaming mode)", model);
        println!("📝 Prompt length: {} characters", prompt.len());
        let generate_url = format!("{}/api/generate", self.base_url);
        println!("🔗 Attempting to connect to: {}", generate_url);
        
        // Use timeout for the entire request
        let response_future = self.client.post(&generate_url)
            .json(&request)
            .send();
        
        match timeout(Duration::from_secs(REQUEST_TIMEOUT), response_future).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    // For streaming, we need to read the response as text and parse it
                    let response_text = response.text().await?;
                    let mut full_response = String::new();
                    
                    // Parse streaming response (each line is a JSON object)
                    for line in response_text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        
                        if let Ok(stream_response) = serde_json::from_str::<StreamResponse>(line) {
                            full_response.push_str(&stream_response.response);
                        }
                    }
                    
                    if full_response.is_empty() {
                        Err(anyhow!("Empty response from Ollama streaming"))
                    } else {
                        Ok(full_response)
                    }
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
    
    // Fallback to non-streaming mode
    async fn generate_without_streaming(&self, model: &str, prompt: &str) -> Result<String> {
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaClient::create_default_options(),
        };
        
        println!("🧠 Using model: {} (non-streaming mode)", model);
        let generate_url = format!("{}/api/generate", self.base_url);
        
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
            // PERFORMANCE OPTIMIZED: Faster responses for trading analysis (3-8s)
            num_predict: 150,          // Shorter responses for speed
            temperature: 0.1,          // More focused responses
            top_k: 10,                 // Reduced sampling for speed
            top_p: 0.8,               // Efficient nucleus sampling
            
            // Performance optimizations for speed
            num_ctx: 512,             // Smaller context for speed
            num_batch: 4,             // Smaller batch for faster processing
            num_thread: -1,           // Use all CPU cores
            repeat_penalty: 1.02,     // Minimal repetition penalty
            
            // Disable advanced features for speed
            mirostat: 0,              // Disable for speed
            mirostat_eta: 0.0,        
            mirostat_tau: 0.0,
        }
    }
    
    // Default options (like ollama run)
    fn create_default_options() -> GenerateOptions {
        GenerateOptions {
            // DEFAULT MODE: Like ollama run (compatible with all models)
            num_predict: 256,          // Increased for better responses
            temperature: 0.7,          // Standard temperature
            top_k: 40,                // Standard sampling
            top_p: 0.9,               // Standard nucleus sampling
            
            // Standard settings
            num_ctx: 4096,            // Increased context window
            num_batch: 256,           // Reduced batch size for better compatibility
            num_thread: -1,           // Use all CPU cores
            repeat_penalty: 1.1,      // Standard repetition penalty
            
            // Standard features
            mirostat: 0,              // Disabled by default
            mirostat_eta: 0.0,        
            mirostat_tau: 0.0,
        }
    }

    /// Generate text with custom parameters for consensus engine
    pub async fn generate_with_params(
        &self, 
        model: &str, 
        prompt: &str, 
        temperature: f64, 
        max_tokens: u32
    ) -> Result<String> {
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: GenerateOptions {
                num_predict: max_tokens as i32,
                temperature: temperature as f32,
                top_k: 20,
                top_p: 0.9,
                num_ctx: 2048,
                num_batch: 16,
                num_thread: -1,
                repeat_penalty: 1.1,
                mirostat: 0,
                mirostat_eta: 0.0,
                mirostat_tau: 0.0,
            },
        };
        
        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let generate_response: GenerateResponse = response.json().await?;
            Ok(generate_response.response)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Ollama API error: {}", error_text))
        }
    }

    /// Chat with a model using the conversations endpoint
    pub async fn chat_with_model(
        &self,
        model: &str,
        messages: Vec<crate::ollama::conversation_manager::ConversationMessage>,
        temperature: f32,
        max_tokens: i32,
    ) -> Result<String> {
        let request = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": max_tokens,
                "top_k": 20,
                "top_p": 0.9,
                "repeat_penalty": 1.1
            }
        });
        
        let response = self.client
            .post(&format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let chat_response: serde_json::Value = response.json().await?;
            if let Some(message) = chat_response["message"]["content"].as_str() {
                Ok(message.to_string())
            } else {
                Err(anyhow::anyhow!("Invalid response format from Ollama chat API"))
            }
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Ollama chat API error: {}", error_text))
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