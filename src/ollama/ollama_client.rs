use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use futures_util::StreamExt;
use tokio::time::timeout;
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::ollama::ollama_receipt::OllamaReceipt;

// Connection pool configuration
const MAX_CONCURRENT_REQUESTS: usize = 10;
const CONNECTION_TIMEOUT: u64 = 15;  // Increased from 5 to 15 seconds
const REQUEST_TIMEOUT: u64 = 30;
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

pub struct OllamaClient {
    client: Client,
    base_url: String,
    semaphore: Arc<Semaphore>,
}

impl OllamaClient {
    pub fn new(base_url: &str, timeout_seconds: u64) -> Self {
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
            options: OllamaClient::create_ultra_fast_options(),
        };
        
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
                println!("⏰ Request timeout after {} seconds", REQUEST_TIMEOUT);
                Err(anyhow!("Request timeout after {} seconds", REQUEST_TIMEOUT))
            }
        }
    }
    
    // Ultra-fast options for maximum performance (simplified for compatibility)
    fn create_ultra_fast_options() -> GenerateOptions {
        GenerateOptions {
            // ULTRA-FAST MODE: Maximum speed (2-4s) - Simplified for compatibility
            num_predict: 100,          // Shorter responses for speed
            temperature: 0.1,          // Very focused responses
            top_k: 10,                 // Minimal sampling space
            top_p: 0.8,                // Aggressive nucleus sampling
            
            // Basic performance optimizations (compatible with all models)
            num_ctx: 1024,             // Smaller context for speed
            num_batch: 16,             // Moderate batch size for compatibility
            num_thread: -1,            // Use all CPU cores
            repeat_penalty: 1.05,      // Minimal repetition penalty
            
            // Disable advanced features for compatibility
            mirostat: 0,               // Disable for speed and compatibility
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

        let mut stream = response.bytes_stream();
        let mut text_chunks = Vec::new();
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        text_chunks.push(text);
                    }
                }
                Err(e) => {
                    log::warn!("Stream chunk error: {}", e);
                    continue;
                }
            }
        }
        
        let total_chars: usize = text_chunks.iter().map(|s| s.len()).sum();
        receipt.finish(start_instant, total_chars, true, None);
        Ok((text_chunks, receipt))
    }
}