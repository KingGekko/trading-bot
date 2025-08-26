use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_stream::StreamExt;
use crate::ollama::ollama_receipt::OllamaReceipt;

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
}

impl OllamaClient {
    pub fn new(base_url: &str, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .pool_idle_timeout(Duration::from_secs(30))  // Keep connections alive
            .pool_max_idle_per_host(10)                  // Connection pooling
            .tcp_keepalive(Duration::from_secs(60))      // TCP keep-alive
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
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
                    if let Ok(chunk_str) = std::str::from_utf8(&chunk) {
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