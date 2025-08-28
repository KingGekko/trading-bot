use anyhow::{anyhow, Result};
use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub max_timeout_seconds: u64,
    pub log_directory: String,
    pub max_prompt_length: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Try to load from config.env file first
        if let Err(_) = dotenv::from_filename("config.env") {
            // Fallback to .env if config.env doesn't exist
            dotenv::dotenv().ok();
        }

        // Required environment variables - no defaults for security
        let ollama_base_url = env::var("OLLAMA_BASE_URL")
            .map_err(|_| anyhow!("OLLAMA_BASE_URL environment variable is required"))?;

        // Model selection - try to auto-detect, fallback to config, then default
        let ollama_model = env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "auto".to_string());

        let max_timeout_seconds = env::var("MAX_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| anyhow!("MAX_TIMEOUT_SECONDS must be a valid number"))?;

        let log_directory = env::var("LOG_DIRECTORY")
            .unwrap_or_else(|_| "ollama_logs".to_string());

        let max_prompt_length = env::var("MAX_PROMPT_LENGTH")
            .unwrap_or_else(|_| "8192".to_string())
            .parse::<usize>()
            .map_err(|_| anyhow!("MAX_PROMPT_LENGTH must be a valid number"))?;

        // Validate and secure the configuration
        Self::validate_config(&ollama_base_url, &ollama_model, 
                             max_timeout_seconds, max_prompt_length)?;

        Ok(Config {
            ollama_base_url,
            ollama_model,
            max_timeout_seconds,
            log_directory,
            max_prompt_length,
        })
    }

    /// Automatically detect and select the best available Ollama model
    pub async fn auto_detect_model(&mut self) -> Result<()> {
        // If model is not set to auto, keep the current value
        if self.ollama_model != "auto" {
            return Ok(());
        }

        log::info!("🔍 Auto-detecting best available Ollama model...");
        
        // Create a temporary client to check available models
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", self.ollama_base_url);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                                let available_models: Vec<String> = models
                                    .iter()
                                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                                    .map(|s| s.to_string())
                                    .collect();

                                if !available_models.is_empty() {
                                    let best_model = self.select_best_model(&available_models);
                                    log::info!("✅ Auto-selected model: {} (from {} available models)", 
                                              best_model, available_models.len());
                                    self.ollama_model = best_model;
                                    return Ok(());
                                }
                            }
                        }
                        Err(e) => log::warn!("Failed to parse models response: {}", e),
                    }
                }
            }
            Err(e) => log::warn!("Failed to fetch models from Ollama: {}", e),
        }

        // Fallback to default model if auto-detection fails
        log::warn!("⚠️  Auto-detection failed, using fallback model: tinyllama");
        self.ollama_model = "tinyllama".to_string();
        Ok(())
    }

    /// Select the best model from available models based on performance characteristics
    fn select_best_model(&self, available_models: &[String]) -> String {
        // Priority order for model selection (fastest to slowest)
        let model_priorities = [
            // Ultra-fast models (3-5 seconds)
            ("phi", 100),
            ("qwen2.5:0.5b", 95),
            ("gemma2:2b", 90),
            ("tinyllama", 85),
            ("llama2:7b", 80),
            ("llama2:13b", 75),
            ("llama2:70b", 70),
            // Add more models as needed
        ];

        // Find the highest priority available model
        for (model_name, _priority) in model_priorities.iter() {
            if available_models.iter().any(|m| m.contains(model_name)) {
                // Return the full model name as it appears in the list
                return available_models
                    .iter()
                    .find(|m| m.contains(model_name))
                    .unwrap()
                    .clone();
            }
        }

        // If no priority model found, return the first available model
        available_models.first().unwrap_or(&"tinyllama".to_string()).clone()
    }

    /// Get model information and performance characteristics
    pub fn get_model_info(&self) -> String {
        match self.ollama_model.as_str() {
            model if model.contains("phi") => "Ultra-fast (3-5s) - Basic analysis".to_string(),
            model if model.contains("qwen2.5:0.5b") => "Very fast (5-8s) - Good analysis".to_string(),
            model if model.contains("gemma2:2b") => "Fast (6-10s) - Balanced performance".to_string(),
            model if model.contains("tinyllama") => "Fast (8-12s) - Good analysis".to_string(),
            model if model.contains("llama2:7b") => "Moderate (10-20s) - High quality".to_string(),
            model if model.contains("llama2:13b") => "Slower (15-30s) - Excellent quality".to_string(),
            model if model.contains("llama2:70b") => "Slow (30-60s) - Best quality".to_string(),
            _ => "Unknown performance characteristics".to_string(),
        }
    }

    fn validate_config(
        ollama_base_url: &str,
        ollama_model: &str,
        max_timeout_seconds: u64,
        max_prompt_length: usize,
    ) -> Result<()> {
        // Validate URL format and security
        let url = Url::parse(ollama_base_url)
            .map_err(|_| anyhow!("OLLAMA_BASE_URL must be a valid URL"))?;

        // Security checks for URL
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(anyhow!("OLLAMA_BASE_URL must use http or https protocol"));
        }

        // Prevent common security issues
        if url.host_str().is_none() {
            return Err(anyhow!("OLLAMA_BASE_URL must have a valid host"));
        }

        // Validate model name (prevent injection attacks)
        if ollama_model != "auto" && (ollama_model.is_empty() || ollama_model.len() > 100) {
            return Err(anyhow!("OLLAMA_MODEL must be 1-100 characters or 'auto'"));
        }

        if ollama_model != "auto" && !ollama_model.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
            return Err(anyhow!("OLLAMA_MODEL contains invalid characters"));
        }

        // Validate limits to prevent resource exhaustion
        if max_timeout_seconds > 3600 {
            return Err(anyhow!("MAX_TIMEOUT_SECONDS cannot exceed 3600 (1 hour)"));
        }

        if max_timeout_seconds < 1 {
            return Err(anyhow!("MAX_TIMEOUT_SECONDS must be at least 1 second"));
        }

        if max_prompt_length > 1_000_000 {
            return Err(anyhow!("MAX_PROMPT_LENGTH cannot exceed 1,000,000 characters"));
        }

        Ok(())
    }



    pub fn sanitize_input(&self, input: &str) -> Result<String> {
        if input.len() > self.max_prompt_length {
            return Err(anyhow!("Input exceeds maximum length of {} characters", self.max_prompt_length));
        }

        // Remove potentially dangerous characters/sequences
        let sanitized = input
            .replace('\0', "") // Remove null bytes
            .replace('\x1b', "") // Remove escape sequences
            .trim()
            .to_string();

        if sanitized.is_empty() {
            return Err(anyhow!("Input cannot be empty after sanitization"));
        }

        Ok(sanitized)
    }


}