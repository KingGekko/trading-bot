use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use std::collections::HashMap;
use uuid::Uuid;

// Include the generated protobuf code
pub mod receipt {
    include!(concat!(env!("OUT_DIR"), "/trading_bot.rs"));
}

use receipt::{
    ModelCapabilities, ModelPerformance, OllamaReceipt as ProtoReceipt, ReceiptLog, RequestContext,
    SystemInfo,
};

pub struct ProtobufReceipt {
    pub proto_receipt: ProtoReceipt,
}

impl ProtobufReceipt {
    pub fn new(
        request_type: String,
        model_name: String,
        prompt_length: usize,
        prompt_text: &str,
    ) -> Self {
        let request_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();
        
        // Analyze prompt complexity
        let (prompt_category, prompt_complexity, keywords) = Self::analyze_prompt(prompt_text);
        
        // Get system information
        let system_info = Self::get_system_info();
        
        // Get model capabilities
        let model_capabilities = Self::get_model_capabilities(&model_name);
        
        let proto_receipt = ProtoReceipt {
            request_id,
            request_type,
            model_name: model_name.clone(),
            model_version: "latest".to_string(),
            start_time: Some(Self::chrono_to_timestamp(start_time)),
            end_time: Some(Self::chrono_to_timestamp(start_time)),
            duration_ms: 0,
            duration_ns: 0,
            prompt_length: prompt_length as u32,
            response_length: 0,
            tokens_generated: 0,
            tokens_per_second: 0,
            chars_per_second: 0.0,
            words_per_second: 0.0,
            response_quality_score: 0.0,
            model_performance: Some(Self::get_model_performance(&model_name)),
            model_capabilities: Some(model_capabilities),
            request_context: Some(RequestContext {
                user_id: "default".to_string(),
                session_id: Uuid::new_v4().to_string(),
                prompt_category,
                prompt_complexity,
                prompt_keywords: keywords,
            }),
            success: false,
            error_message: String::new(),
            error_code: String::new(),
            system_info: Some(system_info),
            metadata: HashMap::new(),
        };

        Self { proto_receipt }
    }

    pub fn finish(
        &mut self,
        response_length: usize,
        success: bool,
        error_message: Option<String>,
        response_text: &str,
    ) {
        let end_time = Utc::now();
        let start_time = self.proto_receipt.start_time.as_ref()
            .map(|ts| Self::timestamp_to_chrono(ts))
            .unwrap_or_else(Utc::now);
        
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;
        let duration_ns = duration.num_nanoseconds().unwrap_or(0) as u64;
        
        // Calculate performance metrics
        let chars_per_second = if duration_ms > 0 {
            response_length as f32 / (duration_ms as f32 / 1000.0)
        } else {
            0.0
        };
        
        let words_per_second = if duration_ms > 0 {
            response_text.split_whitespace().count() as f32 / (duration_ms as f32 / 1000.0)
        } else {
            0.0
        };
        
        // Estimate tokens (rough approximation: 1 token ≈ 4 characters)
        let tokens_generated = (response_length / 4) as u32;
        let tokens_per_second = if duration_ms > 0 {
            tokens_generated as f32 / (duration_ms as f32 / 1000.0)
        } else {
            0.0
        };
        
        // Calculate response quality score
        let response_quality_score = Self::calculate_quality_score(response_text, duration_ms);
        
        // Update the protobuf receipt
        self.proto_receipt.end_time = Some(Self::chrono_to_timestamp(end_time));
        self.proto_receipt.duration_ms = duration_ms;
        self.proto_receipt.duration_ns = duration_ns;
        self.proto_receipt.response_length = response_length as u32;
        self.proto_receipt.tokens_generated = tokens_generated;
        self.proto_receipt.tokens_per_second = tokens_per_second as u32;
        self.proto_receipt.chars_per_second = chars_per_second;
        self.proto_receipt.words_per_second = words_per_second;
        self.proto_receipt.response_quality_score = response_quality_score;
        self.proto_receipt.success = success;
        
        if let Some(error) = error_message {
            self.proto_receipt.error_message = error;
            self.proto_receipt.error_code = "ERROR".to_string();
        }
        
        // Add metadata
        self.proto_receipt.metadata.insert(
            "response_time_category".to_string(),
            Self::categorize_response_time(duration_ms),
        );
        self.proto_receipt.metadata.insert(
            "performance_rating".to_string(),
            Self::get_performance_rating(chars_per_second),
        );
    }

    pub fn save_to_protobuf_file(&self, file_path: &str) -> Result<(), std::io::Error> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        
        // Serialize to protobuf binary format
        let bytes = prost::Message::encode_to_vec(&self.proto_receipt);
        file.write_all(&bytes)?;
        file.write_all(b"\n")?; // Add newline separator
        
        Ok(())
    }

    pub fn save_batch_log(&self, receipts: &[ProtobufReceipt], file_path: &str) -> Result<(), std::io::Error> {
        let proto_receipts: Vec<ProtoReceipt> = receipts
            .iter()
            .map(|r| r.proto_receipt.clone())
            .collect();
        
        let receipt_log = ReceiptLog {
            receipts: proto_receipts,
            log_created: Some(Self::chrono_to_timestamp(Utc::now())),
            log_version: "1.0.0".to_string(),
            total_receipts: receipts.len() as u32,
            successful_requests: receipts.iter().filter(|r| r.proto_receipt.success).count() as u32,
            failed_requests: receipts.iter().filter(|r| !r.proto_receipt.success).count() as u32,
        };
        
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)?;
        
        let bytes = prost::Message::encode_to_vec(&receipt_log);
        file.write_all(&bytes)?;
        
        Ok(())
    }

    // Helper methods
    fn chrono_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }

    fn timestamp_to_chrono(ts: &Timestamp) -> DateTime<Utc> {
        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
            .unwrap_or_else(Utc::now)
    }

    fn analyze_prompt(prompt: &str) -> (String, String, Vec<String>) {
        let word_count = prompt.split_whitespace().count();
        let char_count = prompt.len();
        
        let complexity = if word_count < 10 && char_count < 100 {
            "simple".to_string()
        } else if word_count < 50 && char_count < 500 {
            "moderate".to_string()
        } else {
            "complex".to_string()
        };
        
        let category = if prompt.to_lowercase().contains("trading") || prompt.to_lowercase().contains("market") {
            "trading".to_string()
        } else if prompt.to_lowercase().contains("analysis") || prompt.to_lowercase().contains("explain") {
            "analysis".to_string()
        } else if prompt.to_lowercase().contains("code") || prompt.to_lowercase().contains("programming") {
            "programming".to_string()
        } else {
            "general".to_string()
        };
        
        // Extract keywords (simple approach)
        let keywords: Vec<String> = prompt
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .take(5)
            .map(|s| s.to_lowercase())
            .collect();
        
        (category, complexity, keywords)
    }

    fn get_system_info() -> SystemInfo {
        SystemInfo {
            os_type: std::env::consts::OS.to_string(),
            os_version: "unknown".to_string(),
            ollama_version: "unknown".to_string(),
            trading_bot_version: env!("CARGO_PKG_VERSION").to_string(),
            available_memory_mb: 0, // Would need system-specific code to get this
            cpu_cores: num_cpus::get() as u32,
            gpu_type: "unknown".to_string(),
            gpu_memory_gb: 0.0,
        }
    }

    fn get_model_capabilities(model_name: &str) -> ModelCapabilities {
        let (max_context, max_output, family) = match model_name {
            name if name.contains("phi") => (2048, 2048, "phi"),
            name if name.contains("qwen2.5:0.5b") => (4096, 4096, "qwen"),
            name if name.contains("gemma2:2b") => (8192, 8192, "gemma"),
            name if name.contains("tinyllama") => (2048, 2048, "llama"),
            name if name.contains("llama2:7b") => (4096, 4096, "llama"),
            name if name.contains("llama2:13b") => (4096, 4096, "llama"),
            name if name.contains("llama2:70b") => (4096, 4096, "llama"),
            _ => (2048, 2048, "unknown"),
        };
        
        ModelCapabilities {
            max_context_length: max_context,
            max_output_length: max_output,
            supported_tasks: vec!["text-generation".to_string(), "chat".to_string()],
            supports_streaming: true,
            supports_function_calling: false,
            model_family: family.to_string(),
        }
    }

    fn get_model_performance(model_name: &str) -> ModelPerformance {
        match model_name {
            name if name.contains("phi") => ModelPerformance {
                speed_category: "ultra-fast".to_string(),
                expected_response_time_ms: 3000,
                speed_score: 1.0,
                performance_tier: "basic".to_string(),
            },
            name if name.contains("qwen2.5:0.5b") => ModelPerformance {
                speed_category: "very-fast".to_string(),
                expected_response_time_ms: 5000,
                speed_score: 0.95,
                performance_tier: "good".to_string(),
            },
            name if name.contains("gemma2:2b") => ModelPerformance {
                speed_category: "fast".to_string(),
                expected_response_time_ms: 8000,
                speed_score: 0.90,
                performance_tier: "good".to_string(),
            },
            name if name.contains("tinyllama") => ModelPerformance {
                speed_category: "fast".to_string(),
                expected_response_time_ms: 10000,
                speed_score: 0.85,
                performance_tier: "good".to_string(),
            },
            name if name.contains("llama2:7b") => ModelPerformance {
                speed_category: "moderate".to_string(),
                expected_response_time_ms: 15000,
                speed_score: 0.80,
                performance_tier: "excellent".to_string(),
            },
            name if name.contains("llama2:13b") => ModelPerformance {
                speed_category: "slower".to_string(),
                expected_response_time_ms: 25000,
                speed_score: 0.75,
                performance_tier: "excellent".to_string(),
            },
            name if name.contains("llama2:70b") => ModelPerformance {
                speed_category: "slow".to_string(),
                expected_response_time_ms: 45000,
                speed_score: 0.70,
                performance_tier: "best".to_string(),
            },
            _ => ModelPerformance {
                speed_category: "unknown".to_string(),
                expected_response_time_ms: 10000,
                speed_score: 0.80,
                performance_tier: "unknown".to_string(),
            },
        }
    }

    fn calculate_quality_score(response: &str, duration_ms: u64) -> f32 {
        let mut score: f32 = 1.0;
        
        // Penalize very short responses
        if response.len() < 10 {
            score -= 0.3;
        }
        
        // Penalize very long response times
        if duration_ms > 30000 {
            score -= 0.2;
        } else if duration_ms > 10000 {
            score -= 0.1;
        }
        
        // Bonus for well-structured responses
        if response.contains('\n') && response.len() > 50 {
            score += 0.1;
        }
        
        score.max(0.0).min(1.0)
    }

    fn categorize_response_time(duration_ms: u64) -> String {
        match duration_ms {
            0..=5000 => "ultra-fast".to_string(),
            5001..=10000 => "fast".to_string(),
            10001..=20000 => "moderate".to_string(),
            20001..=30000 => "slow".to_string(),
            _ => "very-slow".to_string(),
        }
    }

    fn get_performance_rating(chars_per_second: f32) -> String {
        match chars_per_second {
            cps if cps >= 100.0 => "excellent".to_string(),
            cps if cps >= 50.0 => "good".to_string(),
            cps if cps >= 25.0 => "average".to_string(),
            cps if cps >= 10.0 => "below-average".to_string(),
            _ => "poor".to_string(),
        }
    }
} 