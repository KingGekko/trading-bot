use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use log::info;

use super::{handlers::create_router, json_stream::JsonStreamManager};
use super::handlers::ApiState;

/// Start the API server for JSON streaming
pub async fn start_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Create JSON stream manager
    let json_manager = Arc::new(JsonStreamManager::new());
    
    // Create API state
    let state = ApiState {
        json_manager: json_manager.clone(),
    };
    
    // Create router
    let app = create_router(state);
    
    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 API Server starting on {}", addr);
    info!("📡 Available endpoints:");
    info!("   GET  /health                    - Health check");
    info!("   POST /api/watch                 - Start watching a JSON file");
    info!("   GET  /api/watch/:file_path     - Stop watching a file");
    info!("   GET  /api/files                - List watched files");
    info!("   GET  /api/content/:file_path   - Get file content");
    info!("   GET  /api/stream/:file_path    - WebSocket stream for real-time updates");
    info!("   POST /api/ollama/process       - Process JSON file with Ollama AI");
    info!("   POST /api/ollama/process/threaded - Process JSON file with Ollama AI (threaded, non-blocking)");
    info!("   POST /api/ollama/process/ultra-fast - Process JSON file with Ollama AI (maximum speed, direct async)");
    info!("   POST /api/ollama/process/ultra-threaded - Process JSON file with Ollama AI (maximum threading, parallel operations)");
    info!("   GET  /api/available-files      - List available JSON files in directory");
    
    // Start server
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Start API server with default configuration
pub async fn start_default_api_server() -> Result<(), Box<dyn std::error::Error>> {
    start_api_server(8080).await
}

/// Start API server with custom port
pub async fn start_api_server_with_port(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    start_api_server(port).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_api_server_startup() {
        // This test verifies the server can start (but doesn't actually bind to port)
        let json_manager = Arc::new(JsonStreamManager::new());
        let state = ApiState {
            json_manager: json_manager.clone(),
        };
        
        let app = create_router(state);
        assert!(app.into_make_service().is_service());
    }
} 