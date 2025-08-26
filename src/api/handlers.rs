use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};

use super::json_stream::JsonStreamManager;

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
        .with_state(state)
}

/// Request payload for starting file watching
#[derive(serde::Deserialize)]
pub struct StartWatchingRequest {
    pub file_path: String,
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