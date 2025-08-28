use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use serde_json::Value;
use notify::{Watcher, RecursiveMode, RecommendedWatcher};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use anyhow::Result;
use log::{info, warn};

/// Manages JSON file streaming with real-time updates
pub struct JsonStreamManager {
    /// Active file watchers
    watchers: Arc<RwLock<HashMap<String, FileWatcher>>>,
    /// Broadcast channels for each file
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Value>>>>,
}

/// Individual file watcher
struct FileWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl JsonStreamManager {
    /// Create a new JSON stream manager
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching a JSON file for changes
    pub async fn watch_file(&self, file_path: &str) -> Result<broadcast::Receiver<Value>> {
        let path = PathBuf::from(file_path);
        
        log::info!("JsonStreamManager: Attempting to watch file: {}", file_path);
        log::info!("JsonStreamManager: Resolved path: {:?}", path);
        
        if !path.exists() {
            log::error!("JsonStreamManager: File does not exist: {}", file_path);
            return Err(anyhow::anyhow!("File does not exist: {}", file_path));
        }
        
        log::info!("JsonStreamManager: File exists, checking if already watching...");
        
        // Check if already watching
        {
            let channels = self.channels.read().await;
            if let Some(sender) = channels.get(file_path) {
                log::info!("JsonStreamManager: Already watching file: {}", file_path);
                return Ok(sender.subscribe());
            }
        }
        
        log::info!("JsonStreamManager: Creating new broadcast channel...");
        
        // Create new broadcast channel
        let (tx, rx) = broadcast::channel(100);
        
        // Store the channel
        {
            let mut channels = self.channels.write().await;
            channels.insert(file_path.to_string(), tx.clone());
            log::info!("JsonStreamManager: Stored channel for file: {}", file_path);
        }
        
        log::info!("JsonStreamManager: Starting file watcher...");
        
        // Start file watcher
        self.start_file_watcher(file_path.to_string(), path, tx).await?;
        
        log::info!("JsonStreamManager: Successfully started watching file: {}", file_path);
        Ok(rx)
    }

    /// Start watching a specific file
    async fn start_file_watcher(
        &self,
        file_path: String,
        path: PathBuf,
        tx: broadcast::Sender<Value>,
    ) -> Result<()> {
        log::info!("JsonStreamManager: start_file_watcher called for: {}", file_path);
        
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        log::info!("JsonStreamManager: Created notify channel");
        
        // Create file watcher
        log::info!("JsonStreamManager: Creating RecommendedWatcher...");
        let mut watcher = RecommendedWatcher::new(notify_tx, notify::Config::default())?;
        log::info!("JsonStreamManager: Created watcher, starting to watch path: {:?}", path);
        
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        log::info!("JsonStreamManager: Successfully started watching path: {:?}", path);

        // Store watcher
        {
            let mut watchers = self.watchers.write().await;
            watchers.insert(file_path.clone(), FileWatcher {
                _watcher: watcher,
            });
            log::info!("JsonStreamManager: Stored watcher in watchers map");
        }

        // Send initial file content
        log::info!("JsonStreamManager: Reading initial file content...");
        if let Ok(content) = Self::read_json_file(&path).await {
            log::info!("JsonStreamManager: Successfully read initial content, sending...");
            if let Err(e) = tx.send(content) {
                warn!("Failed to send initial content for {}: {}", file_path, e);
            } else {
                log::info!("JsonStreamManager: Successfully sent initial content");
            }
        } else {
            log::warn!("JsonStreamManager: Failed to read initial file content");
        }

        // Spawn background task to handle file changes
        let file_path_clone = file_path.clone();
        let tx_clone = tx.clone();
        let path_clone = path.clone();
        log::info!("JsonStreamManager: Spawning background task for file changes...");
        tokio::spawn(async move {
            Self::handle_file_changes(file_path_clone, path_clone, tx_clone, notify_rx).await;
        });

        info!("Started watching file: {}", file_path);
        Ok(())
    }

    /// Handle file change notifications
    async fn handle_file_changes(
        file_path: String,
        path: PathBuf,
        tx: broadcast::Sender<Value>,
        rx: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    ) {
        for event_result in rx {
            match event_result {
                Ok(event) => {
                    match event {
                        notify::Event {
                            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(_)),
                            paths,
                            ..
                        } => {
                            if paths.contains(&path) {
                                if let Ok(content) = Self::read_json_file(&path).await {
                                    if let Err(e) = tx.send(content) {
                                        warn!("Failed to broadcast update for {}: {}", file_path, e);
                                    } else {
                                        info!("Broadcasted update for file: {}", file_path);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    warn!("File watcher error for {}: {}", file_path, e);
                }
            }
        }
    }

    /// Read and parse JSON file
    async fn read_json_file(path: &PathBuf) -> Result<Value> {
        log::info!("JsonStreamManager: read_json_file called for path: {:?}", path);
        
        let file = File::open(path).await?;
        log::info!("JsonStreamManager: Successfully opened file");
        
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;
        log::info!("JsonStreamManager: Successfully read file content, length: {}", content.len());
        
        let json: Value = serde_json::from_str(&content)?;
        log::info!("JsonStreamManager: Successfully parsed JSON");
        Ok(json)
    }

    /// Stop watching a file
    pub async fn stop_watching(&self, file_path: &str) -> Result<()> {
        let mut watchers = self.watchers.write().await;
        let mut channels = self.channels.write().await;
        
        watchers.remove(file_path);
        channels.remove(file_path);
        
        info!("Stopped watching file: {}", file_path);
        Ok(())
    }

    /// Get list of watched files
    pub async fn get_watched_files(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }

    /// Get current content of a watched file
    pub async fn get_file_content(&self, file_path: &str) -> Result<Value> {
        let path = PathBuf::from(file_path);
        Self::read_json_file(&path).await
    }
}

impl Default for JsonStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use serde_json::json;

    #[tokio::test]
    async fn test_json_stream_manager() {
        let manager = JsonStreamManager::new();
        
        // Create temporary JSON file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        
        // Write initial content
        let initial_content = json!({"test": "data"});
        std::fs::write(file_path, serde_json::to_string(&initial_content).unwrap()).unwrap();
        
        // Start watching
        let mut receiver = manager.watch_file(file_path).await.unwrap();
        
        // Should receive initial content
        let received = receiver.recv().await.unwrap();
        assert_eq!(received, initial_content);
        
        // Stop watching
        manager.stop_watching(file_path).await.unwrap();
    }
} 