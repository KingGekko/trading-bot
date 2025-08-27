use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{timeout, Duration, Instant};

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
    
    match state.json_manager.watch_file(&file_path).await {
        Ok(_) => {
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
            serde_json::to_string(&message).unwrap()
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
                    serde_json::to_string(&message).unwrap()
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
                                serde_json::to_string(&pong).unwrap()
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
        .route("/api/watch/:file_path", get(stop_watching))
        .route("/api/files", get(get_watched_files))
        .route("/api/content/:file_path", get(get_file_content))
        .route("/api/stream/:file_path", get(websocket_handler))
        .route("/api/ollama/process", post(ollama_process_json))
        .route("/api/ollama/process/threaded", post(ollama_process_json_threaded))
        .route("/api/ollama/process/ultra-fast", post(ollama_process_ultra_fast))
        .route("/api/ollama/process/ultra-threaded", post(ollama_process_ultra_threaded))
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

/// Process JSON file content with Ollama using a prompt
pub async fn ollama_process_json(
    State(state): State<ApiState>,
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
    
    // Get the file content
    let file_content = match state.json_manager.get_file_content(&file_path_str).await {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to get content for {}: {}", file_path_str, e);
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
    
    // Create a comprehensive prompt that includes the JSON data
    let enhanced_prompt = format!(
        "Please analyze the following JSON data and respond to this prompt: {}\n\nJSON Data:\n{}",
        payload.prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
    // Process with Ollama
    match ollama_client.generate(&model, &enhanced_prompt).await {
        Ok(response) => {
            Ok(Json(json!({
                "status": "success",
                "file_path": file_path_str,
                "prompt": payload.prompt,
                "model": model,
                "ollama_response": response,
                "json_data_processed": file_content
            })))
        }
        Err(e) => {
            log::error!("Ollama processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Process JSON file content with Ollama using a prompt (Threaded Stream Version)
pub async fn ollama_process_json_threaded(
    State(state): State<ApiState>,
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
    
    // Get the file content
    let file_content = match state.json_manager.get_file_content(&file_path_str).await {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to get content for {}: {}", file_path_str, e);
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
    
    // Create a comprehensive prompt that includes the JSON data
    let enhanced_prompt = format!(
        "Please analyze the following JSON data and respond to this prompt: {}\n\nJSON Data:\n{}",
        payload.prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
    // Process with Ollama using threaded streams with timeout
    let ollama_future = tokio::spawn_blocking(move || {
        // This runs in a separate thread to prevent blocking
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Use the optimized client for maximum performance
            ollama_client.generate_optimized(&model, &enhanced_prompt).await
        })
    });
    
    // Add timeout to prevent hanging
    let timeout_duration = Duration::from_secs(30); // 30 second timeout
    
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
                "timeout_seconds": 30
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
            log::error!("Ollama request timed out after {} seconds", timeout_duration.as_secs());
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Ultra-fast Ollama processing (direct async, no threading overhead)
pub async fn ollama_process_ultra_fast(
    State(state): State<ApiState>,
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
    
    // Get the file content
    let file_content = match state.json_manager.get_file_content(&file_path_str).await {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to get content for {}: {}", file_path_str, e);
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
    let enhanced_prompt = format!(
        "Analyze this JSON data: {}\n\nData: {}",
        payload.prompt,
        serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
    );
    
    let prompt_prep_time = start_time.elapsed() - file_read_time;
    
    // Process with ultra-fast timeout
    let timeout_duration = Duration::from_secs(15); // 15 second timeout for ultra-fast mode
    
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
                "timeout_seconds": 15,
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
            log::error!("Ollama request timed out after {} seconds", timeout_duration.as_secs());
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Ultra-threaded Ollama processing (maximum threading optimization)
pub async fn ollama_process_ultra_threaded(
    State(state): State<ApiState>,
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
    
    // Spawn file content reading in a separate thread
    let file_content_future = tokio::spawn_blocking(move || {
        // This runs in a separate thread for I/O operations
        std::fs::read_to_string(&file_path_str)
    });
    
    // Spawn config loading in parallel
    let config_future = tokio::spawn_blocking(|| {
        Config::from_env()
    });
    
    // Wait for both operations to complete
    let (file_content_result, config_result) = tokio::join!(
        file_content_future,
        config_future
    );
    
    let file_content = match file_content_result? {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file {}: {}", file_path_str, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let config = match config_result? {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let file_read_time = start_time.elapsed();
    
    // Create optimized Ollama client
    let ollama_client = OllamaClient::new(&config.ollama_base_url, config.max_timeout_seconds);
    
    // Use provided model or default from config
    let model = payload.model.unwrap_or_else(|| config.ollama_model.clone());
    
    // Spawn prompt preparation in a separate thread
    let prompt_future = tokio::spawn_blocking(move || {
        // This runs in a separate thread for string processing
        format!(
            "Analyze this JSON data: {}\n\nData: {}",
            payload.prompt,
            serde_json::to_string_pretty(&file_content).unwrap_or_else(|_| format!("{:?}", file_content))
        )
    });
    
    let enhanced_prompt = prompt_future.await?;
    let prompt_prep_time = start_time.elapsed() - file_read_time;
    
    // Spawn Ollama processing in a separate thread with timeout
    let ollama_start = Instant::now();
    let timeout_duration = Duration::from_secs(10); // 10 second timeout for ultra-threaded mode
    
    let ollama_future = tokio::spawn_blocking(move || {
        // This runs in a separate thread for the blocking Ollama call
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            ollama_client.generate_optimized(&model, &enhanced_prompt).await
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
                "timeout_seconds": 10,
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
            log::error!("Ollama request timed out after {} seconds", timeout_duration.as_secs());
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
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
    
    if let Ok(entries) = std::fs::read_dir(current_dir) {
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
    use axum::http::StatusCode;
    use serde_json::json;

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