use anyhow::{anyhow, Result};
use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub bot_name: String,
    pub log_level: String,
    pub max_timeout_seconds: u64,
    pub log_directory: String,
    pub max_prompt_length: usize,
    pub max_response_length: usize,
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

        let ollama_model = env::var("OLLAMA_MODEL")
            .map_err(|_| anyhow!("OLLAMA_MODEL environment variable is required"))?;

        // Optional environment variables with secure defaults
        let bot_name = env::var("BOT_NAME")
            .unwrap_or_else(|_| "TradingBot".to_string());

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

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

        let max_response_length = env::var("MAX_RESPONSE_LENGTH")
            .unwrap_or_else(|_| "32768".to_string())
            .parse::<usize>()
            .map_err(|_| anyhow!("MAX_RESPONSE_LENGTH must be a valid number"))?;

        // Validate and secure the configuration
        Self::validate_config(&ollama_base_url, &ollama_model, &log_level, 
                             max_timeout_seconds, max_prompt_length, max_response_length)?;

        Ok(Config {
            ollama_base_url,
            ollama_model,
            bot_name,
            log_level,
            max_timeout_seconds,
            log_directory,
            max_prompt_length,
            max_response_length,
        })
    }

    fn validate_config(
        ollama_base_url: &str,
        ollama_model: &str,
        log_level: &str,
        max_timeout_seconds: u64,
        max_prompt_length: usize,
        max_response_length: usize,
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
        if ollama_model.is_empty() || ollama_model.len() > 100 {
            return Err(anyhow!("OLLAMA_MODEL must be 1-100 characters"));
        }

        if !ollama_model.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
            return Err(anyhow!("OLLAMA_MODEL contains invalid characters"));
        }

        // Validate log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&log_level.to_lowercase().as_str()) {
            return Err(anyhow!("LOG_LEVEL must be one of: error, warn, info, debug, trace"));
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

        if max_response_length > 10_000_000 {
            return Err(anyhow!("MAX_RESPONSE_LENGTH cannot exceed 10,000,000 characters"));
        }

        Ok(())
    }

    pub fn display(&self) {
        println!("Configuration:");
        println!("  Ollama Base URL: {}", self.ollama_base_url);
        println!("  Ollama Model: {}", self.ollama_model);
        println!("  Bot Name: {}", self.bot_name);
        println!("  Log Level: {}", self.log_level);
        println!("  Max Timeout: {} seconds", self.max_timeout_seconds);
        println!("  Log Directory: {}", self.log_directory);
        println!("  Max Prompt Length: {} characters", self.max_prompt_length);
        println!("  Max Response Length: {} characters", self.max_response_length);
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

    pub fn validate_response_length(&self, response: &str) -> Result<()> {
        if response.len() > self.max_response_length {
            return Err(anyhow!("Response exceeds maximum length of {} characters", self.max_response_length));
        }
        Ok(())
    }
}