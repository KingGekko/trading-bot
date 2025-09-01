use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{timeout, Duration, Instant};
use tokio::task::spawn_blocking;

use futures_util::{SinkExt, StreamExt};

use super::json_stream::JsonStreamManager;
use crate::ollama::OllamaClient;
use crate::ollama::Config;

/// API state shared across handlers
#[derive(Clone)]
pub struct ApiState {
    pub json_manager: Arc<JsonStreamManager>,
}

/// Start watching a JSON file
pub async fn start_watching(
    State(state): State<ApiState>,
    Json(payload): Json<StartWatchingRequest>,
) -> Result<Json<Value>, StatusCode> {
    let file_path = payload.file_path;
    
    log::info!("Starting to watch file: {}", file_path);
    
    // Check if file exists
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        log::error!("File does not exist: {}", file_path);
        return Err(StatusCode::NOT_FOUND);
    }
    
    log::info!("File exists, attempting to start watch...");
    
    match state.json_manager.watch_file(&file_path).await {
        Ok(_) => {
            log::info!("Successfully started watching: {}", file_path);
            Ok(Json(json!({
                "status": "success",
                "message": format!("Started watching file: {}", file_path),
                "file_path": file_path
            })))
        }
        Err(e) => {
            log::error!("Failed to start watching {}: {}", file_path, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Stop watching a JSON file
pub async fn stop_watching(
    State(state): State<ApiState>,
    Path(file_path): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.json_manager.stop_watching(&file_path).await {
        Ok(_) => {
            Ok(Json(json!({
                "status": "success",
                "message": format!("Stopped watching file: {}", file_path),
                "file_path": file_path
            })))
        }
        Err(e) => {
            log::error!("Failed to stop watching {}: {}", file_path, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get list of watched files
pub async fn get_watched_files(
    State(state): State<ApiState>,
) -> Json<Value> {
    let files = state.json_manager.get_watched_files().await;
    Json(json!({
        "status": "success",
        "watched_files": files
    }))
}

/// Get current content of a watched file
pub async fn get_file_content(
    State(state): State<ApiState>,
    Path(file_path): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.json_manager.get_file_content(&file_path).await {
        Ok(content) => {
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path,
                "content": content
            })))
        }
        Err(e) => {
            log::error!("Failed to get content for {}: {}", file_path, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// WebSocket handler for real-time JSON streaming
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    Path(file_path): Path<String>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, file_path))
}

/// Handle WebSocket connection for JSON streaming
async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: ApiState,
    file_path: String,
) {
    let (mut sender, mut receiver) = socket.split();
    
    // Start watching the file
    let mut file_receiver = match state.json_manager.watch_file(&file_path).await {
        Ok(receiver) => receiver,
        Err(e) => {
            log::error!("Failed to start watching {}: {}", file_path, e);
            return;
        }
    };
    
    // Send initial content
    if let Ok(content) = state.json_manager.get_file_content(&file_path).await {
        let message = json!({
            "type": "initial",
            "file_path": file_path,
            "content": content
        });
        
        if let Err(e) = sender.send(axum::extract::ws::Message::Text(
            serde_json::to_string(&message).unwrap().into()
        )).await {
            log::error!("Failed to send initial content: {}", e);
            return;
        }
    }
    
    // Handle incoming messages and file updates
    loop {
        tokio::select! {
            // Handle file updates
            Ok(update) = file_receiver.recv() => {
                let message = json!({
                    "type": "update",
                    "file_path": file_path,
                    "content": update,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                if let Err(e) = sender.send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&message).unwrap().into()
                )).await {
                    log::error!("Failed to send update: {}", e);
                    break;
                }
            }
            
            // Handle WebSocket messages
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        log::info!("WebSocket closed for file: {}", file_path);
                        break;
                    }
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        // Handle client messages (e.g., ping/pong)
                        if text == "ping" {
                            let pong = json!({
                                "type": "pong",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            
                            if let Err(e) = sender.send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&pong).unwrap().into()
                            )).await {
                                log::error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Stop watching when WebSocket closes
    if let Err(e) = state.json_manager.stop_watching(&file_path).await {
        log::error!("Failed to stop watching {}: {}", file_path, e);
    }
}

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "trading-bot-api"
    }))
}

/// Create the API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/watch", post(start_watching))
        .route("/api/watch/{file_path}", get(stop_watching))
        .route("/api/files", get(get_watched_files))
        .route("/api/content/{file_path}", get(get_file_content))
        .route("/api/stream/{file_path}", get(websocket_handler))
        .route("/api/ollama/process", post(ollama_process_json))
        .route("/api/ollama/process/threaded", post(ollama_process_json_threaded))
        .route("/api/ollama/process/ultra-fast", post(ollama_process_ultra_fast))
        .route("/api/ollama/process/ultra-threaded", post(ollama_process_ultra_threaded))
        .route("/api/ollama/conversation", post(multi_model_conversation))
        .route("/api/available-files", get(list_available_files))
        .with_state(state)
}

/// Request payload for starting file watching
#[derive(serde::Deserialize)]
pub struct StartWatchingRequest {
    pub file_path: String,
}

/// Request payload for Ollama to process JSON file with prompt
#[derive(serde::Deserialize)]
pub struct OllamaProcessRequest {
    pub file_path: String,
    pub prompt: String,
    pub model: Option<String>,
}

/// Process JSON file with Ollama AI (default: ultra-fast threading)
pub async fn ollama_process_json(
    State(_state): State<ApiState>,
    Json(payload): Json<OllamaProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = Instant::now();
    
    // Normalize the file path
    let file_path = if payload.file_path.starts_with("./") {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path[2..])
    } else if payload.file_path.starts_with("/") {
        std::path::PathBuf::from(&payload.file_path)
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path)
    };
    
    let file_path_str = file_path.to_string_lossy().to_string();
    let file_path_str_clone = file_path_str.clone(); // Clone for closure
    
    // Get file content and config in parallel using ultra-fast threading
    let (file_content_result, config_result) = tokio::join!(
        spawn_blocking(move || std::fs::read_to_string(&file_path_str_clone)),
        spawn_blocking(|| Config::from_env())
    );
    
    let file_content = match file_content_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let config = match config_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let file_read_time = start_time.elapsed();
    
    // Create optimized Ollama client
    log::info!("🔧 Creating Ollama client with base_url: {} and timeout: {} seconds", config.ollama_base_url, config.max_timeout_seconds);
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Use provided model or default from config
    let received_model = payload.model.clone(); // Store original value
    let model = received_model.as_ref().unwrap_or(&config.ollama_model).clone();
    log::info!("🧠 API received model: {:?}, using model: {}", received_model, model);
    let model_clone = model.clone(); // Clone for closure
    let payload_prompt = payload.prompt.clone(); // Clone for closure
    let file_content_clone = file_content.clone(); // Clone for closure
    
    // Create optimized prompt in separate thread
    let prompt_future = spawn_blocking(move || {
        // Use custom prompt or default Elite trading analyst prompt
        let base_prompt = if payload_prompt.trim().is_empty() {
            "You are an Elite quantitative trading analyst specializing in algorithmic trading and portfolio optimization. 

ANALYZE THE FOLLOWING TRADING PORTFOLIO DATA AND PROVIDE SPECIFIC TRADING RECOMMENDATIONS:

1. PORTFOLIO ANALYSIS: Evaluate current positions, cash allocation, and risk metrics
2. MARKET OPPORTUNITIES: Identify specific buy/sell opportunities based on current market data
3. RISK ASSESSMENT: Analyze portfolio risk levels and suggest risk management strategies
4. TRADING ACTIONS: Provide specific recommendations with:
   - Buy/Sell/Hold decisions
   - Target entry/exit prices
   - Position sizes (as % of portfolio)
   - Stop loss levels
   - Time horizon for each trade

5. PORTFOLIO OPTIMIZATION: Suggest portfolio rebalancing based on Modern Portfolio Theory

FOCUS ON ACTIONABLE TRADING DECISIONS, NOT GENERAL MARKET COMMENTARY."
        } else {
            &payload_prompt
        };
        
        format!(
            "{}\n\nData: {}",
            base_prompt,
            serde_json::to_string_pretty(&file_content_clone).unwrap_or_else(|_| format!("{:?}", file_content_clone))
        )
    });
    
    let enhanced_prompt: String = prompt_future.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let prompt_prep_time = start_time.elapsed() - file_read_time;
    
    // Process with Ollama using ultra-fast threading
    let ollama_start = Instant::now();
    let timeout_duration = Duration::from_secs(config.max_timeout_seconds); // Use configurable timeout
    log::info!("🕒 Using timeout duration: {} seconds (from config.max_timeout_seconds: {})", timeout_duration.as_secs(), config.max_timeout_seconds);
    
    let ollama_future = spawn_blocking(move || {
        // This runs in a separate thread for the blocking Ollama call
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            ollama_client.generate_optimized(&model_clone, &enhanced_prompt).await
        })
    });
    
    match timeout(timeout_duration, ollama_future).await {
        Ok(Ok(Ok(response))) => {
            let ollama_time = ollama_start.elapsed();
            let total_time = start_time.elapsed();
            
            // Validate response quality
            let response_trimmed = response.trim();
            if response_trimmed.len() < 100 {
                log::warn!("⚠️  Ollama response seems too short ({} chars). Consider using a larger model.", response_trimmed.len());
            }
            
            if response_trimmed.contains("curl") || response_trimmed.contains("API") {
                log::warn!("⚠️  Ollama response appears to be generic API advice rather than trading analysis.");
            }
            
            // Log performance metrics
            log::info!("Ultra-fast threading (default) completed - File: {}ms, Prompt: {}ms, Ollama: {}ms, Total: {}ms", 
                file_read_time.as_millis(), 
                prompt_prep_time.as_millis(), 
                ollama_time.as_millis(), 
                total_time.as_millis()
            );
            
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path_str,
                "prompt": payload.prompt,
                "model": model,
                "ollama_response": response,
                "json_data_processed": file_content,
                "processing_method": "ultra_fast_threading_default",
                "timeout_seconds": config.max_timeout_seconds,
                "performance_mode": "maximum_speed_threading",
                "threading_strategy": "parallel_file_config_prompt_ollama",
                "performance_metrics": {
                    "file_read_ms": file_read_time.as_millis(),
                    "prompt_prep_ms": prompt_prep_time.as_millis(),
                    "ollama_processing_ms": ollama_time.as_millis(),
                    "total_time_ms": total_time.as_millis(),
                    "threading_overhead_ms": (total_time - file_read_time - prompt_prep_time - ollama_time).as_millis()
                }
            })))
        }
        Ok(Ok(Err(e))) => {
            log::error!("Ollama processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(Err(e)) => {
            log::error!("Threading error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            log::error!("Ollama request timed out after {} seconds (configured timeout: {}s). Consider increasing MAX_TIMEOUT_SECONDS in config.env or checking Ollama server performance.", timeout_duration.as_secs(), config.max_timeout_seconds);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Process JSON file content with Ollama using a prompt (Threaded Stream Version)
pub async fn ollama_process_json_threaded(
    State(_state): State<ApiState>,
    Json(payload): Json<OllamaProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Normalize the file path - handle both relative and absolute paths
    let file_path = if payload.file_path.starts_with("./") {
        // Convert relative path to absolute path
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path[2..])
    } else if payload.file_path.starts_with("/") {
        // Absolute path
        std::path::PathBuf::from(&payload.file_path)
    } else {
        // Relative path from current directory
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path)
    };
    
    // Convert back to string for the API response
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Get the file content directly (ultra-fast mode doesn't use state)
    let file_content = match std::fs::read_to_string(&file_path_str) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    // Create Ollama client
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load Ollama config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Use provided model or default from config
    let model = payload.model.unwrap_or_else(|| config.ollama_model.clone());
    let model_clone = model.clone(); // Clone for the closure
    
    // Create a comprehensive prompt that includes the JSON data
    let enhanced_prompt = format!(
        "You are an Elite quantitative trading analyst. 

ANALYZE THE FOLLOWING TRADING PORTFOLIO DATA AND PROVIDE SPECIFIC TRADING RECOMMENDATIONS:

PROMPT: {}

PORTFOLIO DATA:
{}

REQUIRED OUTPUT FORMAT:
1. PORTFOLIO SUMMARY: Current positions, cash, risk metrics
2. MARKET ANALYSIS: Current market conditions and opportunities
3. TRADING RECOMMENDATIONS: Specific buy/sell actions with prices and quantities
4. RISK MANAGEMENT: Stop losses and position sizing
5. PORTFOLIO OPTIMIZATION: Rebalancing suggestions

FOCUS ON ACTIONABLE TRADING DECISIONS.",
        payload.prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
            // Process with Ollama using threaded streams with timeout
        let ollama_future = spawn_blocking(move || {
        // This runs in a separate thread to prevent blocking
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Use the optimized client for maximum performance
            ollama_client.generate_optimized(&model_clone, &enhanced_prompt).await
        })
    });
    
    // Add timeout to prevent hanging
    let timeout_duration = Duration::from_secs(config.max_timeout_seconds); // Use configurable timeout
    
    match timeout(timeout_duration, ollama_future).await {
        Ok(Ok(Ok(response))) => {
            // Success: model processed the request
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path_str,
                "prompt": payload.prompt,
                "model": model,
                "ollama_response": response,
                "json_data_processed": file_content,
                "processing_method": "threaded_stream",
                "timeout_seconds": config.max_timeout_seconds
            })))
        }
        Ok(Ok(Err(e))) => {
            // Ollama error
            log::error!("Ollama processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(Err(join_error)) => {
            // Thread join error
            log::error!("Thread execution failed: {}", join_error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            // Timeout error
            log::error!("Ollama request timed out after {} seconds (configured timeout: {}s). Consider increasing MAX_TIMEOUT_SECONDS in config.env or checking Ollama server performance.", timeout_duration.as_secs(), config.max_timeout_seconds);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Ultra-fast Ollama processing (direct async, no threading overhead)
pub async fn ollama_process_ultra_fast(
    State(_state): State<ApiState>,
    Json(payload): Json<OllamaProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = Instant::now();
    
    // Normalize the file path
    let file_path = if payload.file_path.starts_with("./") {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path[2..])
    } else if payload.file_path.starts_with("/") {
        std::path::PathBuf::from(&payload.file_path)
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path)
    };
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Get the file content directly (ultra-fast mode doesn't use state)
    let file_content = match std::fs::read_to_string(&file_path_str) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let file_read_time = start_time.elapsed();
    
    // Create optimized Ollama client
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load Ollama config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Use provided model or default from config
    let model = payload.model.unwrap_or_else(|| config.ollama_model.clone());
    
    // Create optimized prompt
    let base_prompt = if payload.prompt.trim().is_empty() {
        "You are an Elite quantitative trading analyst specializing in algorithmic trading and portfolio optimization. 

ANALYZE THE FOLLOWING TRADING PORTFOLIO DATA AND PROVIDE SPECIFIC TRADING RECOMMENDATIONS:

1. PORTFOLIO ANALYSIS: Evaluate current positions, cash allocation, and risk metrics
2. MARKET OPPORTUNITIES: Identify specific buy/sell opportunities based on current market data
3. RISK ASSESSMENT: Analyze portfolio risk levels and suggest risk management strategies
4. TRADING ACTIONS: Provide specific recommendations with:
   - Buy/Sell/Hold decisions
   - Target entry/exit prices
   - Position sizes (as % of portfolio)
   - Stop loss levels
   - Time horizon for each trade

5. PORTFOLIO OPTIMIZATION: Suggest portfolio rebalancing based on Modern Portfolio Theory

FOCUS ON ACTIONABLE TRADING DECISIONS, NOT GENERAL MARKET COMMENTARY."
    } else {
        &payload.prompt
    };
    
    let enhanced_prompt = format!(
        "{}\n\nData: {}",
        base_prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
    let prompt_prep_time = start_time.elapsed() - file_read_time;
    
    // Process with ultra-fast timeout
    let timeout_duration = Duration::from_secs(config.max_timeout_seconds); // Use configurable timeout
    
    let ollama_start = Instant::now();
    match timeout(timeout_duration, ollama_client.generate_optimized(&model, &enhanced_prompt)).await {
        Ok(Ok(response)) => {
            let ollama_time = ollama_start.elapsed();
            let total_time = start_time.elapsed();
            
            // Log performance metrics
            log::info!("Ultra-fast processing completed - File: {}ms, Prompt: {}ms, Ollama: {}ms, Total: {}ms", 
                file_read_time.as_millis(), 
                prompt_prep_time.as_millis(), 
                ollama_time.as_millis(), 
                total_time.as_millis()
            );
            
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path_str,
                "prompt": payload.prompt,
                "model": model,
                "ollama_response": response,
                "json_data_processed": file_content,
                "processing_method": "ultra_fast_direct",
                "timeout_seconds": config.max_timeout_seconds,
                "performance_mode": "maximum_speed",
                "performance_metrics": {
                    "file_read_ms": file_read_time.as_millis(),
                    "prompt_prep_ms": prompt_prep_time.as_millis(),
                    "ollama_processing_ms": ollama_time.as_millis(),
                    "total_time_ms": total_time.as_millis()
                }
            })))
        }
        Ok(Err(e)) => {
            log::error!("Ollama processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            log::error!("Ollama request timed out after {} seconds (configured timeout: {}s). Consider increasing MAX_TIMEOUT_SECONDS in config.env or checking Ollama server performance.", timeout_duration.as_secs(), config.max_timeout_seconds);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Ultra-threaded Ollama processing (maximum threading optimization)
pub async fn ollama_process_ultra_threaded(
    State(_state): State<ApiState>,
    Json(payload): Json<OllamaProcessRequest>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = Instant::now();
    
    // Spawn file reading in a separate thread (I/O bound)
    let file_path = if payload.file_path.starts_with("./") {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path[2..])
    } else if payload.file_path.starts_with("/") {
        std::path::PathBuf::from(&payload.file_path)
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path)
    };
    
    let file_path_str = file_path.to_string_lossy().to_string();
    let file_path_str_clone = file_path_str.clone(); // Clone for closure
    
    // Spawn file content reading in a separate thread
    let file_content_future = spawn_blocking(move || {
        // This runs in a separate thread for I/O operations
        std::fs::read_to_string(&file_path_str_clone)
    });
    
    // Spawn config loading in parallel
    let config_future = spawn_blocking(|| {
        Config::from_env()
    });
    
    // Wait for both operations to complete
    let (file_content_result, config_result): (Result<Result<String, std::io::Error>, _>, Result<Result<Config, _>, _>) = tokio::join!(
        file_content_future,
        config_future
    );
    
    let file_content = match file_content_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let config = match config_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let file_read_time = start_time.elapsed();
    
    // Create optimized Ollama client
    log::info!("🔧 Creating Ollama client with base_url: {} and timeout: {} seconds", config.ollama_base_url, config.max_timeout_seconds);
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Use provided model or default from config
    let model = payload.model.unwrap_or_else(|| config.ollama_model.clone());
    let model_clone = model.clone(); // Clone for closure
    let payload_prompt = payload.prompt.clone(); // Clone for closure
    let file_content_clone = file_content.clone(); // Clone for closure
    
    // Spawn prompt preparation in a separate thread
    let prompt_future = spawn_blocking(move || {
        // This runs in a separate thread for string processing
        let base_prompt = if payload_prompt.trim().is_empty() {
            "You are an Elite quantitative trading analyst specializing in algorithmic trading and portfolio optimization. 

ANALYZE THE FOLLOWING TRADING PORTFOLIO DATA AND PROVIDE SPECIFIC TRADING RECOMMENDATIONS:

1. PORTFOLIO ANALYSIS: Evaluate current positions, cash allocation, and risk metrics
2. MARKET OPPORTUNITIES: Identify specific buy/sell opportunities based on current market data
3. RISK ASSESSMENT: Analyze portfolio risk levels and suggest risk management strategies
4. TRADING ACTIONS: Provide specific recommendations with:
   - Buy/Sell/Hold decisions
   - Target entry/exit prices
   - Position sizes (as % of portfolio)
   - Stop loss levels
   - Time horizon for each trade

5. PORTFOLIO OPTIMIZATION: Suggest portfolio rebalancing based on Modern Portfolio Theory

FOCUS ON ACTIONABLE TRADING DECISIONS, NOT GENERAL MARKET COMMENTARY."
        } else {
            &payload_prompt
        };
        
        format!(
            "{}\n\nData: {}",
            base_prompt,
            serde_json::to_string_pretty(&file_content_clone).unwrap_or_else(|_| format!("{:?}", file_content_clone))
        )
    });
    
    let enhanced_prompt = prompt_future.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let prompt_prep_time = start_time.elapsed() - file_read_time;
    
    // Spawn Ollama processing in a separate thread with timeout
    let ollama_start = Instant::now();
    let timeout_duration = Duration::from_secs(config.max_timeout_seconds); // Use configurable timeout
    
    let ollama_future = spawn_blocking(move || {
        // This runs in a separate thread for the blocking Ollama call
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            ollama_client.generate_optimized(&model_clone, &enhanced_prompt).await
        })
    });
    
    match timeout(timeout_duration, ollama_future).await {
        Ok(Ok(Ok(response))) => {
            let ollama_time = ollama_start.elapsed();
            let total_time = start_time.elapsed();
            
            // Log performance metrics
            log::info!("Ultra-threaded processing completed - File: {}ms, Prompt: {}ms, Ollama: {}ms, Total: {}ms", 
                file_read_time.as_millis(), 
                prompt_prep_time.as_millis(), 
                ollama_time.as_millis(), 
                total_time.as_millis()
            );
            
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path_str,
                "prompt": payload.prompt,
                "model": model,
                "ollama_response": response,
                "json_data_processed": file_content,
                "processing_method": "ultra_threaded_optimized",
                "timeout_seconds": config.max_timeout_seconds,
                "performance_mode": "maximum_threading",
                "threading_strategy": "parallel_file_config_prompt_ollama",
                "performance_metrics": {
                    "file_read_ms": file_read_time.as_millis(),
                    "prompt_prep_ms": prompt_prep_time.as_millis(),
                    "ollama_processing_ms": ollama_time.as_millis(),
                    "total_time_ms": total_time.as_millis(),
                    "threading_overhead_ms": (total_time - file_read_time - prompt_prep_time - ollama_time).as_millis()
                }
            })))
        }
        Ok(Ok(Err(e))) => {
            log::error!("Ollama processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(Err(e)) => {
            log::error!("Threading error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            log::error!("Ollama request timed out after {} seconds (configured timeout: {}s). Consider increasing MAX_TIMEOUT_SECONDS in config.env or checking Ollama server performance.", timeout_duration.as_secs(), config.max_timeout_seconds);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Multi-model conversation request
#[derive(Debug, serde::Deserialize)]
pub struct MultiModelConversationRequest {
    pub file_path: String,
    pub initial_prompt: String,
    pub models: Vec<String>,
    pub conversation_rounds: Option<u8>,
    pub conversation_type: Option<String>, // "debate", "collaboration", "review"
}

/// Multi-model conversation response
#[derive(Debug, Clone, Serialize)]
pub struct ModelResponse {
    pub model: String,
    pub response: String,
    pub round: u8,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Multi-model conversation handler
pub async fn multi_model_conversation(
    State(_state): State<ApiState>,
    Json(payload): Json<MultiModelConversationRequest>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = Instant::now();
    let conversation_rounds = payload.conversation_rounds.unwrap_or(3);
    let conversation_type = payload.conversation_type.as_deref().unwrap_or("collaboration");
    
    // Normalize file path
    let file_path = if payload.file_path.starts_with("./") {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path[2..])
    } else if payload.file_path.starts_with("/") {
        std::path::PathBuf::from(&payload.file_path)
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        current_dir.join(&payload.file_path)
    };
    
    let file_path_str = file_path.to_string_lossy().to_string();
    let file_path_str_clone = file_path_str.clone(); // Clone for closure
    
    // Get file content and config in parallel
    let (file_content_result, config_result): (Result<Result<String, std::io::Error>, _>, Result<Result<Config, _>, _>) = tokio::join!(
        spawn_blocking(move || std::fs::read_to_string(&file_path_str_clone)),
        spawn_blocking(|| Config::from_env())
    );
    
    let file_content = match file_content_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let config = match config_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let file_read_time = start_time.elapsed();
    
    // Create Ollama client
    let _ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Initialize conversation context
    let mut conversation_history = Vec::new();
    let base_prompt = if payload.initial_prompt.trim().is_empty() {
        "You are an Elite quantitative trading analyst. Analyze the following trading data to transcend in profit multiplication:"
    } else {
        &payload.initial_prompt
    };
    
    let mut current_context = format!(
        "{}\n\nData: {}\n\n",
        base_prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
    // Clone values for closures
    let config_ollama_base_url = config.ollama_base_url.clone();
    let config_max_timeout_seconds = config.max_timeout_seconds;
    let payload_models = payload.models.clone();
    let payload_initial_prompt = payload.initial_prompt.clone();
    
    // Start multi-model conversation
    for round in 1..=conversation_rounds {
        log::info!("Starting conversation round {} with {} models", round, payload_models.len());
        
        // Process each model in parallel for this round
        let mut round_responses = Vec::new();
        let mut model_futures = Vec::new();
        
        for (model_index, model_name) in payload_models.iter().enumerate() {
            let model_name = model_name.clone();
            let model_name_for_push = model_name.clone(); // Clone for pushing to vector
            let current_context = current_context.clone();
            let conversation_type = conversation_type.to_string();
            let round = round;
            let _model_index = model_index; // Prefix with underscore to suppress warning
            
            // Clone config values for this iteration
            let config_ollama_base_url_clone = config_ollama_base_url.clone();
            let config_max_timeout_seconds_clone = config_max_timeout_seconds;
            
            // Create model-specific prompt based on conversation type
            let model_prompt = match conversation_type.as_str() {
                "debate" => format!(
                    "You are participating in a debate round {}. Previous context: {}\n\nTake a position and respond to the previous statements. Be concise but persuasive.",
                    round, current_context
                ),
                "collaboration" => format!(
                    "You are collaborating with other AI models in round {}. Previous context: {}\n\nBuild upon the previous responses and add your insights. Work together to provide comprehensive analysis.",
                    round, current_context
                ),
                "review" => format!(
                    "You are reviewing responses in round {}. Previous context: {}\n\nReview the previous responses and provide feedback, corrections, or additional insights.",
                    round, current_context
                ),
                _ => format!(
                    "You are participating in round {} of a multi-model conversation. Previous context: {}\n\nProvide your analysis and insights.",
                    round, current_context
                )
            };
            
            // Spawn model response generation in separate thread
            let future = spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // Create a new client instance for this thread
                    let client = OllamaClient::new(&config_ollama_base_url_clone, config_max_timeout_seconds_clone);
                    client.generate_optimized(&model_name, &model_prompt).await
                })
            });
            
            model_futures.push((model_name_for_push, future, _model_index));
        }
        
        // Wait for all models to respond in this round
        for (model_name, future, _model_index) in model_futures {
            match timeout(Duration::from_secs(config.max_timeout_seconds), future).await {
                Ok(Ok(Ok(response))) => {
                    let model_response = ModelResponse {
                        model: model_name.clone(),
                        response: response.clone(),
                        round,
                        timestamp: chrono::Utc::now(),
                    };
                    
                    round_responses.push(model_response.clone());
                    conversation_history.push(model_response);
                    
                    // Update context for next round
                    current_context.push_str(&format!(
                        "\n\n--- Round {} - {} ---\n{}\n",
                        round, model_name, response
                    ));
                    
                    log::info!("Model {} completed round {} in {}ms", model_name, round, start_time.elapsed().as_millis());
                }
                Ok(Ok(Err(e))) => {
                    log::error!("Model {} failed in round {}: {}", model_name, round, e);
                    // Continue with other models
                }
                Ok(Err(e)) => {
                    log::error!("Threading error for model {} in round {}: {}", model_name, round, e);
                }
                Err(_) => {
                    log::error!("Model {} timed out in round {}", model_name, round);
                }
            }
        }
        
        // Add round separator
        current_context.push_str(&format!("\n\n=== End of Round {} ===\n", round));
        
        log::info!("Completed round {} with {} responses", round, round_responses.len());
    }
    
    let total_time = start_time.elapsed();
    
    // Generate conversation summary
    let summary_prompt = format!(
        "Summarize this multi-model conversation about the trading data. \
         Models involved: {}. Conversation type: {}. \
         Key insights from all rounds:\n\n{}",
        payload_models.join(", "),
        conversation_type,
        current_context
    );
    
    // Clone config values for summary generation
    let config_ollama_base_url_clone = config_ollama_base_url.clone();
    let config_max_timeout_seconds_clone = config_max_timeout_seconds;
    let payload_models_clone = payload_models.clone();
    
    let summary_future = spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = OllamaClient::new(&config_ollama_base_url_clone, config_max_timeout_seconds_clone);
            client.generate_optimized(&payload_models_clone[0], &summary_prompt).await
        })
    });
    
    let summary = match timeout(Duration::from_secs(config.max_timeout_seconds), summary_future).await {
        Ok(Ok(Ok(summary))) => summary,
        _ => "Failed to generate summary".to_string(),
    };
    
    Ok(Json(json!({
        "status": "success",
        "file_path": file_path_str,
        "initial_prompt": payload_initial_prompt,
        "models": payload_models,
        "conversation_type": conversation_type,
        "conversation_rounds": conversation_rounds,
        "conversation_history": conversation_history,
        "summary": summary,
        "processing_method": "multi_model_conversation",
        "performance_metrics": {
            "file_read_ms": file_read_time.as_millis(),
            "total_conversation_ms": total_time.as_millis(),
            "models_per_round": payload.models.len(),
            "total_responses": conversation_history.len(),
            "average_response_time_ms": total_time.as_millis() / conversation_history.len() as u128
        }
    })))
}

/// Get list of available JSON files in current directory
pub async fn list_available_files() -> Json<Value> {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            return Json(json!({
                "status": "error",
                "message": "Failed to get current directory"
            }));
        }
    };
    
    let mut json_files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        json_files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    Json(json!({
        "status": "success",
        "current_directory": current_dir.to_string_lossy(),
        "available_json_files": json_files,
        "total_files": json_files.len()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let body = response.0;
        
        assert_eq!(body["status"], "healthy");
        assert!(body["timestamp"].is_string());
        assert_eq!(body["service"], "trading-bot-api");
    }

    #[tokio::test]
    async fn test_start_watching_request() {
        let request = StartWatchingRequest {
            file_path: "/test/file.json".to_string(),
        };
        
        assert_eq!(request.file_path, "/test/file.json");
    }
} 