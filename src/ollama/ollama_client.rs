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
const CONNECTION_TIMEOUT: u64 = 5;
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
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    response: String,
    done: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    context: Option<Vec<i32>>,
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
            .http2_prior_knowledge()
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
            Ok(Err(e)) => Err(anyhow!("Request failed: {}", e)),
            Err(_) => Err(anyhow!("Request timeout after {} seconds", REQUEST_TIMEOUT)),
        }
    }
    
    // Ultra-fast options for maximum performance
    fn create_ultra_fast_options() -> GenerateOptions {
        GenerateOptions {
            // ULTRA-FAST MODE: Maximum speed (2-4s)
            num_predict: 100,          // Shorter responses for speed
            temperature: 0.1,          // Very focused responses
            top_k: 10,                 // Minimal sampling space
            top_p: 0.8,                // Aggressive nucleus sampling
            
            // Performance optimizations
            num_ctx: 1024,             // Smaller context for speed
            num_batch: 32,             // Larger batch size
            num_thread: -1,            // Use all CPU cores
            repeat_penalty: 1.05,      // Minimal repetition penalty
            
            // Speed-focused features
            mirostat: 0,               // Disable for speed
            mirostat_eta: 0.0,        
            mirostat_tau: 0.0,
        }
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (response, _timing) = self.generate_with_timing(model, prompt).await?;
        Ok(response)
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

    pub async fn generate_stream_simple(&self, model: &str, prompt: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true, // Enable streaming
            options: Self::create_balanced_options(),
        };

        println!("Sending streaming request to: {}", url);
        
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
        let mut buffer = String::with_capacity(8192); // Buffer for balanced responses (~200 tokens)
        let mut text_chunks = Vec::with_capacity(64); // Expected chunks for balanced mode
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    // Convert bytes to string and add to buffer
                    if let Ok(chunk_str) = std::str::from_utf8(&chunk.as_ref()) {
                        buffer.push_str(chunk_str);
                        
                        // Process complete lines (JSON objects)
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer.drain(..newline_pos + 1);
                            
                            if !line.is_empty() {
                                // Try to parse the JSON line
                                match serde_json::from_str::<StreamResponse>(&line) {
                                    Ok(stream_response) => {
                                        if let Some(error) = stream_response.error {
                                            return Err(anyhow!("Ollama error: {}", error));
                                        }
                                        
                                        if stream_response.done {
                                            // End of stream
                                            return Ok(text_chunks);
                                        }
                                        
                                        if !stream_response.response.is_empty() {
                                            text_chunks.push(stream_response.response);
                                        }
                                    }
                                    Err(_) => {
                                        // Skip malformed JSON lines
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Stream error: {}", e));
                }
            }
        }
        
        Ok(text_chunks)
    }

    pub async fn generate_stream_with_timing(&self, model: &str, prompt: &str) -> Result<(Vec<String>, OllamaReceipt)> {
        let (mut receipt, start_instant) = OllamaReceipt::new(
            "StreamGenerate".to_string(),
            model.to_string(),
            prompt.len(),
        );

        match self.generate_stream_simple(model, prompt).await {
            Ok(chunks) => {
                let total_chars: usize = chunks.iter().map(|s| s.len()).sum();
                receipt.finish(start_instant, total_chars, true, None);
                Ok((chunks, receipt))
            }
            Err(e) => {
                receipt.finish(start_instant, 0, false, Some(e.to_string()));
                Err(e)
            }
        }
    }

    // High-performance streaming with optimized chunk processing
    pub async fn generate_stream_optimized(&self, model: &str, prompt: &str) -> Result<(Vec<String>, OllamaReceipt)> {
        let _permit = self.semaphore.acquire().await.map_err(|e| anyhow!("Semaphore error: {}", e))?;
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options: OllamaClient::create_ultra_fast_options(),
        };
        
        let generate_url = format!("{}/api/generate", self.base_url);
        
        let response = self.client.post(&generate_url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }
        
        let mut chunks = Vec::new();
        let mut stream = response.bytes_stream();
        
        // Process chunks with minimal delay
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        chunks.push(text);
                    }
                }
                Err(e) => {
                    log::warn!("Stream chunk error: {}", e);
                    continue;
                }
            }
        }
        
        // Create receipt for logging
        let (mut receipt, start_instant) = OllamaReceipt::new(
            "StreamOptimized".to_string(),
            model.to_string(),
            prompt.len(),
        );
        
        // Finish the receipt
        receipt.finish(start_instant, chunks.join("").len(), true, None);
        
        Ok((chunks, receipt))
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch models from Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Ollama API returned error status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        #[derive(Deserialize)]
        struct Model {
            name: String,
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<Model>,
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse models response: {}", e))?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}