use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaReceipt {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: u128,
    pub request_type: String,
    pub model: String,
    pub prompt_length: usize,
    pub response_length: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

impl OllamaReceipt {
    pub fn new(request_type: String, model: String, prompt_length: usize) -> (Self, Instant) {
        let start_instant = Instant::now();
        let start_time = Utc::now();
        
        (
            Self {
                start_time,
                end_time: start_time, // Will be updated when finished
                duration_ms: 0,
                request_type,
                model,
                prompt_length,
                response_length: 0,
                success: false,
                error_message: None,
            },
            start_instant,
        )
    }

    pub fn finish(&mut self, start_instant: Instant, response_length: usize, success: bool, error_message: Option<String>) {
        self.end_time = Utc::now();
        self.duration_ms = start_instant.elapsed().as_millis();
        self.response_length = response_length;
        self.success = success;
        self.error_message = error_message;
    }

    pub fn save_to_log(&self, log_directory: &str) -> Result<(), std::io::Error> {
        // Ensure log directory exists
        std::fs::create_dir_all(log_directory)?;
        
        let log_file = if self.success {
            format!("{}/success_receipts.jsonl", log_directory)
        } else {
            format!("{}/failure_receipts.jsonl", log_directory)
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        // Pretty-printed JSON is the built-in standard format for all receipts
        // This ensures human-readable logs while maintaining programmatic parsing capability
        let json_pretty = serde_json::to_string_pretty(self)?;
        writeln!(file, "{}", json_pretty)?;
        
        Ok(())
    }

    pub fn load_receipts_from_file(file_path: &str) -> Result<Vec<OllamaReceipt>, Box<dyn std::error::Error>> {
        use std::fs;

        let content = fs::read_to_string(file_path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut receipts = Vec::new();
        
        // Use a simple regex approach to extract JSON objects
        let mut current_json = String::new();
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for ch in content.chars() {
            if escape_next {
                escape_next = false;
                current_json.push(ch);
                continue;
            }
            
            if ch == '\\' {
                escape_next = true;
                current_json.push(ch);
                continue;
            }
            
            if ch == '"' && !escape_next {
                in_string = !in_string;
            }
            
            if !in_string {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
            
            current_json.push(ch);
            
            // When we have a complete JSON object
            if brace_count == 0 && current_json.trim().starts_with('{') {
                match serde_json::from_str::<OllamaReceipt>(&current_json.trim()) {
                    Ok(receipt) => receipts.push(receipt),
                    Err(e) => {
                        log::warn!("Failed to parse receipt: {}", e);
                    }
                }
                current_json.clear();
            }
        }

        Ok(receipts)
    }

    pub fn display_receipt_summary(&self, index: Option<usize>) {
        let prefix = if let Some(i) = index {
            format!("Receipt #{}: ", i + 1)
        } else {
            String::new()
        };

        let status_icon = if self.success { "✅" } else { "❌" };
        let duration_sec = self.duration_ms as f64 / 1000.0;
        
        println!("{}{}[{}] {} -> {} in {:.2}s ({} chars → {} chars)", 
            prefix,
            status_icon,
            self.start_time.format("%H:%M:%S"),
            self.request_type,
            self.model,
            duration_sec,
            self.prompt_length,
            self.response_length
        );

        if let Some(error) = &self.error_message {
            println!("    Error: {}", error);
        }
    }

    pub fn log_summary(&self, log_directory: &str) {
        let status = if self.success { "SUCCESS" } else { "FAILED" };
        let error_info = self.error_message.as_ref()
            .map(|e| format!(" - Error: {}", e))
            .unwrap_or_default();

        log::info!(
            "[{}] {} request to {} completed in {}ms | Prompt: {} chars | Response: {} chars{}",
            status,
            self.request_type,
            self.model,
            self.duration_ms,
            self.prompt_length,
            self.response_length,
            error_info
        );

        // Save to log file
        if let Err(e) = self.save_to_log(log_directory) {
            log::error!("Failed to save receipt to log file: {}", e);
        }
    }

    pub fn log_detailed(&self) {
        println!("\n=== OLLAMA RECEIPT DETAILS ===");
        println!("Start Time: {}", self.start_time.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("End Time: {}", self.end_time.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Duration: {}ms ({:.2}s)", self.duration_ms, self.duration_ms as f64 / 1000.0);
        println!("Request Type: {}", self.request_type);
        println!("Model: {}", self.model);
        println!("Prompt Length: {} characters", self.prompt_length);
        println!("Response Length: {} characters", self.response_length);
        println!("Status: {}", if self.success { "SUCCESS" } else { "FAILED" });
        
        if let Some(error) = &self.error_message {
            println!("Error: {}", error);
        }

        // Performance analysis
        println!("\n--- PERFORMANCE ANALYSIS ---");
        if self.success {
            let chars_per_ms = self.response_length as f64 / self.duration_ms as f64;
            let chars_per_second = chars_per_ms * 1000.0;
            println!("Generation Speed: {:.2} chars/second", chars_per_second);
            
            if self.duration_ms > 30000 {
                println!("⚠️  SLOW: Response took over 30 seconds");
            } else if self.duration_ms > 10000 {
                println!("⚡ MODERATE: Response took 10-30 seconds");
            } else {
                println!("🚀 FAST: Response completed quickly");
            }
        }

        // Log file information
        let log_file = if self.success { "success_receipts.jsonl" } else { "failure_receipts.jsonl" };
        println!("Receipt logged to: ollama_logs/{}", log_file);
        println!("==============================\n");
    }
}