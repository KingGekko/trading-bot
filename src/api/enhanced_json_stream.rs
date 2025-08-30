use crate::market_data::types::MarketData;
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{accept_async, WebSocketStream, MaybeTlsStream};
use tokio::net::{TcpListener, TcpStream};
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc;

/// Enhanced JSON streamer that combines WebSocket, file watching, and market data
pub struct EnhancedJsonStreamer {
    data_dir: PathBuf,
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    running: Arc<RwLock<bool>>,
    port: u16,
}

/// Client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe {
        streams: Vec<String>, // "market_data", "file_updates", "ai_analysis"
        symbols: Option<Vec<String>>,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        streams: Vec<String>,
    },
    #[serde(rename = "request_data")]
    RequestData {
        symbol: String,
        data_type: String, // "current", "historical", "analysis"
    },
}

/// Server message types
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "market_data")]
    MarketData {
        symbol: String,
        data: MarketData,
        timestamp: String,
    },
    #[serde(rename = "file_update")]
    FileUpdate {
        filename: String,
        content: Value,
        timestamp: String,
    },
    #[serde(rename = "ai_analysis")]
    AiAnalysis {
        symbol: String,
        analysis: String,
        confidence: f64,
        timestamp: String,
    },
    #[serde(rename = "system")]
    System {
        message: String,
        status: String,
        timestamp: String,
    },
}

impl EnhancedJsonStreamer {
    /// Create a new enhanced JSON streamer
    pub fn new(data_dir: PathBuf, port: u16) -> Result<Self> {
        Ok(Self {
            data_dir,
            market_data: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            port,
        })
    }

    /// Start the enhanced JSON streamer
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Enhanced JSON Streamer on port {}", self.port);
        
        // Create data directory
        fs::create_dir_all(&self.data_dir).await?;
        
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Start all components concurrently
        let mut tasks = Vec::new();
        
        // Task 1: WebSocket server for client connections
        let ws_task = {
            let clients = self.clients.clone();
            let running = self.running.clone();
            let port = self.port;
            tokio::spawn(async move {
                Self::run_websocket_server(clients, running, port).await
            })
        };
        tasks.push(ws_task);
        
        // Task 2: File watching for local changes
        let file_watch_task = {
            let data_dir = self.data_dir.clone();
            let clients = self.clients.clone();
            let running = self.running.clone();
            tokio::spawn(async move {
                Self::run_file_watcher(data_dir, clients, running).await
            })
        };
        tasks.push(file_watch_task);
        
        // Task 3: Market data streaming (if connected to Alpaca)
        let market_data_task = {
            let market_data = self.market_data.clone();
            let clients = self.clients.clone();
            let running = self.running.clone();
            tokio::spawn(async move {
                Self::run_market_data_stream(market_data, clients, running).await
            })
        };
        tasks.push(market_data_task);
        
        // Task 4: AI analysis processing
        let ai_task = {
            let market_data = self.market_data.clone();
            let clients = self.clients.clone();
            let running = self.running.clone();
            tokio::spawn(async move {
                Self::run_ai_analysis(market_data, clients, running).await
            })
        };
        tasks.push(ai_task);
        
        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        
        // Check results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => info!("✅ Task {} completed successfully", i),
                Ok(Err(e)) => error!("❌ Task {} failed: {}", i, e),
                Err(e) => error!("❌ Task {} panicked: {}", i, e),
            }
        }
        
        Ok(())
    }

    /// Stop the enhanced JSON streamer
    pub async fn stop(&self) {
        info!("🛑 Stopping Enhanced JSON Streamer...");
        
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Run WebSocket server for client connections
    async fn run_websocket_server(
        clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        running: Arc<RwLock<bool>>,
        port: u16,
    ) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        info!("🌐 WebSocket server listening on {}", addr);
        
        while {
            let running = running.read().await;
            *running
        } {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("🔌 New client connection from {}", addr);
                    
                    // Handle client connection
                    let clients = clients.clone();
                    let running = running.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client_connection(stream, clients, running).await {
                            error!("❌ Client connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ Accept error: {}", e);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok(())
    }

    /// Handle individual client WebSocket connection
    async fn handle_client_connection(
        stream: TcpStream,
        clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let client_id = uuid::Uuid::new_v4().to_string();
        
        // Add client to active connections
        {
            let mut clients = clients.write().await;
            clients.insert(client_id.clone(), ws_stream);
        }
        
        info!("✅ Client {} connected", client_id);
        
        // Send welcome message
        let welcome_msg = ServerMessage::System {
            message: "Connected to Enhanced JSON Streamer".to_string(),
            status: "connected".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Process client messages
        while {
            let running = running.read().await;
            *running
        } {
            // Handle client messages here
            sleep(Duration::from_millis(100)).await;
        }
        
        // Remove client when disconnected
        {
            let mut clients = clients.write().await;
            clients.remove(&client_id);
        }
        
        info!("🔌 Client {} disconnected", client_id);
        Ok(())
    }

    /// Run file watcher for local file changes
    async fn run_file_watcher(
        data_dir: PathBuf,
        clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        info!("📁 Starting file watcher for {}", data_dir.display());
        
        let (tx, rx) = mpsc::channel();
        let mut watcher = watcher(tx, notify::RecursiveMode::NonRecursive)?;
        
        // Watch the data directory
        watcher.watch(&data_dir, RecursiveMode::NonRecursive)?;
        
        // Process file change events
        while {
            let running = running.read().await;
            *running
        } {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        notify::DebouncedEvent::Write(path) => {
                            if let Some(filename) = path.file_name() {
                                if let Some(filename_str) = filename.to_str() {
                                    info!("📝 File updated: {}", filename_str);
                                    
                                    // Read file content
                                    if let Ok(content) = fs::read_to_string(&path).await {
                                        if let Ok(json_content) = serde_json::from_str::<Value>(&content) {
                                            // Notify all clients
                                            let file_msg = ServerMessage::FileUpdate {
                                                filename: filename_str.to_string(),
                                                content: json_content,
                                                timestamp: chrono::Utc::now().to_rfc3339(),
                                            };
                                            
                                            Self::broadcast_to_clients(&clients, &file_msg).await;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // No events, continue
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("❌ File watcher channel disconnected");
                    break;
                }
            }
        }
        
        Ok(())
    }

    /// Run market data streaming (simulated for now)
    async fn run_market_data_stream(
        market_data: Arc<RwLock<HashMap<String, MarketData>>>,
        clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        info!("📊 Starting market data stream");
        
        let symbols = vec!["AAPL", "SPY", "BTC/USD", "ETH/USD"];
        let mut counter = 0;
        
        while {
            let running = running.read().await;
            *running
        } {
            // Simulate market data updates
            for symbol in &symbols {
                let mock_data = MarketData {
                    timestamp: chrono::Utc::now(),
                    symbol: symbol.to_string(),
                    price: 100.0 + (counter as f64 * 0.01),
                    volume: 1000.0 + (counter as f64 * 10.0),
                    high: Some(105.0),
                    low: Some(95.0),
                    open: Some(100.0),
                    source: "enhanced_streamer".to_string(),
                    exchange: "simulated".to_string(),
                    change_24h: Some(2.5),
                    change_percent: Some(2.5),
                    market_cap: Some(1000000000.0),
                    circulating_supply: Some(1000000.0),
                    options_data: None,
                    news_data: None,
                };
                
                // Update market data cache
                {
                    let mut data = market_data.write().await;
                    data.insert(symbol.to_string(), mock_data.clone());
                }
                
                // Send to clients
                let market_msg = ServerMessage::MarketData {
                    symbol: symbol.to_string(),
                    data: mock_data,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                
                Self::broadcast_to_clients(&clients, &market_msg).await;
            }
            
            counter += 1;
            sleep(Duration::from_millis(5000)).await; // Update every 5 seconds
        }
        
        Ok(())
    }

    /// Run AI analysis processing
    async fn run_ai_analysis(
        market_data: Arc<RwLock<HashMap<String, MarketData>>>,
        clients: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        info!("🤖 Starting AI analysis stream");
        
        while {
            let running = running.read().await;
            *running
        } {
            // Get current market data
            let data = market_data.read().await;
            
            for (symbol, market_data) in data.iter() {
                // Simple AI analysis (in real implementation, this would call Ollama)
                let analysis = if market_data.change_percent.unwrap_or(0.0) > 0.0 {
                    "Bullish trend detected - price increasing"
                } else {
                    "Bearish trend detected - price decreasing"
                };
                
                let confidence = (market_data.change_percent.unwrap_or(0.0).abs() / 10.0).min(1.0);
                
                let ai_msg = ServerMessage::AiAnalysis {
                    symbol: symbol.clone(),
                    analysis: analysis.to_string(),
                    confidence,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                
                Self::broadcast_to_clients(&clients, &ai_msg).await;
            }
            
            sleep(Duration::from_millis(10000)).await; // Analysis every 10 seconds
        }
        
        Ok(())
    }

    /// Broadcast message to all connected clients
    async fn broadcast_to_clients(
        clients: &Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        message: &ServerMessage,
    ) {
        let clients = clients.read().await;
        let message_json = serde_json::to_string(message).unwrap_or_default();
        
        for (client_id, client) in clients.iter() {
            if let Err(e) = client.send(tokio_tungstenite::tungstenite::Message::Text(message_json.clone())).await {
                warn!("⚠️ Failed to send message to client {}: {}", client_id, e);
            }
        }
    }

    /// Get current market data for a symbol
    pub async fn get_market_data(&self, symbol: &str) -> Option<MarketData> {
        let data = self.market_data.read().await;
        data.get(symbol).cloned()
    }

    /// Get all market data
    pub async fn get_all_market_data(&self) -> HashMap<String, MarketData> {
        let data = self.market_data.read().await;
        data.clone()
    }

    /// Get connected client count
    pub async fn get_client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
}
