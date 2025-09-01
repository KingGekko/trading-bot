use anyhow::{anyhow, Result};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;


/// Asset information from Alpaca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub class: String,
    pub exchange: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub tradable: bool,
    pub marginable: bool,
    pub shortable: bool,
    pub easy_to_borrow: bool,
    pub fractionable: bool,
    pub min_order_size: Option<f64>,
    pub min_trade_increment: Option<f64>,
    pub price_increment: Option<f64>,
}

/// Position information from Alpaca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: String,
    pub symbol: String,
    pub exchange: String,
    pub asset_class: String,
    pub avg_entry_price: String,
    pub qty: String,
    pub side: String,
    pub market_value: String,
    pub cost_basis: String,
    pub unrealized_pl: String,
    pub unrealized_plpc: String,
    pub unrealized_intraday_pl: String,
    pub unrealized_intraday_plpc: String,
    pub current_price: String,
    pub lastday_price: String,
    pub change_today: String,
}

/// Asset Universe Manager
pub struct AssetUniverseManager {
    api_key: String,
    secret_key: String,
    base_url: String,
    client: Client,
}

impl AssetUniverseManager {
    /// Create a new asset universe manager
    pub fn new(api_key: String, secret_key: String, base_url: String) -> Self {
        // Convert WebSocket URL to REST API URL
        let rest_url = if base_url.contains("paper") {
            "https://paper-api.alpaca.markets".to_string()
        } else {
            "https://api.alpaca.markets".to_string()
        };
        
        Self {
            api_key,
            secret_key,
            base_url: rest_url,
            client: Client::new(),
        }
    }

    /// Get all tradable assets from Alpaca
    pub async fn get_tradable_assets(&self) -> Result<Vec<Asset>> {
        println!("🔍 Fetching tradable assets from Alpaca...");
        
        let url = format!("{}/v2/assets", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch assets: {}", response.status()));
        }

        let assets: Vec<Asset> = response.json().await?;
        
        // Filter for tradable assets
        let tradable_assets: Vec<Asset> = assets
            .into_iter()
            .filter(|asset| asset.tradable && asset.status == "active")
            .collect();

        println!("✅ Found {} tradable assets", tradable_assets.len());
        Ok(tradable_assets)
    }

    /// Get popular/commonly traded assets (S&P 500, major ETFs, etc.)
    pub async fn get_popular_assets(&self) -> Result<Vec<Asset>> {
        println!("📈 Fetching popular assets for trading universe...");
        
        // Common symbols for a diversified trading universe
        let popular_symbols = vec![
            // Major ETFs
            "SPY", "QQQ", "IWM", "VTI", "VOO", "VEA", "VWO", "BND", "GLD", "SLV",
            // Tech Giants
            "AAPL", "MSFT", "GOOGL", "AMZN", "TSLA", "META", "NVDA", "NFLX", "ADBE", "CRM",
            // Financial
            "JPM", "BAC", "WFC", "GS", "MS", "C", "AXP", "V", "MA", "BLK",
            // Healthcare
            "JNJ", "PFE", "UNH", "ABBV", "MRK", "TMO", "ABT", "DHR", "BMY", "AMGN",
            // Consumer
            "PG", "KO", "PEP", "WMT", "HD", "MCD", "DIS", "NKE", "SBUX", "COST",
            // Energy
            "XOM", "CVX", "COP", "EOG", "SLB", "KMI", "PSX", "VLO", "MPC", "OXY",
            // Industrial
            "BA", "CAT", "GE", "MMM", "HON", "UPS", "FDX", "RTX", "LMT", "NOC",
            // Crypto (if available)
            "BTCUSD", "ETHUSD", "ADAUSD", "DOTUSD", "LINKUSD",
        ];

        let mut popular_assets = Vec::new();
        
        for symbol in popular_symbols {
            if let Ok(asset) = self.get_asset_by_symbol(symbol).await {
                popular_assets.push(asset);
            }
        }

        println!("✅ Found {} popular assets", popular_assets.len());
        Ok(popular_assets)
    }

    /// Get asset by symbol
    pub async fn get_asset_by_symbol(&self, symbol: &str) -> Result<Asset> {
        let url = format!("{}/v2/assets/{}", self.base_url, symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch asset {}: {}", symbol, response.status()));
        }

        let asset: Asset = response.json().await?;
        Ok(asset)
    }

    /// Get current positions from Alpaca
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        println!("📊 Fetching current positions from Alpaca...");
        
        let url = format!("{}/v2/positions", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch positions: {}", response.status()));
        }

        let positions: Vec<Position> = response.json().await?;
        
        println!("✅ Found {} current positions", positions.len());
        Ok(positions)
    }

    /// Get account portfolio history
    pub async fn get_portfolio_history(&self) -> Result<Value> {
        println!("📈 Fetching portfolio history...");
        
        let url = format!("{}/v2/account/portfolio/history", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch portfolio history: {}", response.status()));
        }

        let history: Value = response.json().await?;
        println!("✅ Portfolio history fetched successfully");
        Ok(history)
    }

    /// Save asset universe to file
    pub async fn save_asset_universe(&self, assets: &[Asset], data_dir: &str) -> Result<()> {
        let file_path = format!("{}/asset_universe.json", data_dir);
        let json_content = serde_json::to_string_pretty(assets)?;
        tokio::fs::write(&file_path, json_content).await?;
        println!("💾 Saved asset universe to: {}", file_path);
        Ok(())
    }

    /// Save positions to file
    pub async fn save_positions(&self, positions: &[Position], data_dir: &str) -> Result<()> {
        let file_path = format!("{}/current_positions.json", data_dir);
        let json_content = serde_json::to_string_pretty(positions)?;
        tokio::fs::write(&file_path, json_content).await?;
        println!("💾 Saved positions to: {}", file_path);
        Ok(())
    }

    /// Load asset universe from consolidated file
    pub async fn load_asset_universe(data_dir: &str) -> Result<Vec<Asset>> {
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        
        if !std::path::Path::new(&portfolio_file).exists() {
            // Fallback to individual file if consolidated file doesn't exist
            let file_path = format!("{}/asset_universe.json", data_dir);
            if !std::path::Path::new(&file_path).exists() {
                return Ok(Vec::new());
            }
            let content = tokio::fs::read_to_string(&file_path).await?;
            let assets: Vec<Asset> = serde_json::from_str(&content)?;
            println!("📂 Loaded {} assets from: {}", assets.len(), file_path);
            return Ok(assets);
        }
        
        let content = tokio::fs::read_to_string(&portfolio_file).await?;
        let data: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(assets_array) = data["asset_universe"].as_array() {
            let assets: Vec<Asset> = serde_json::from_value(serde_json::Value::Array(assets_array.clone()))?;
            println!("📂 Loaded {} assets from: {}", assets.len(), portfolio_file);
            Ok(assets)
        } else {
            Ok(Vec::new())
        }
    }

    /// Load positions from consolidated file
    pub async fn load_positions(data_dir: &str) -> Result<Vec<Position>> {
        let portfolio_file = format!("{}/trading_portfolio.json", data_dir);
        
        if !std::path::Path::new(&portfolio_file).exists() {
            // Fallback to individual file if consolidated file doesn't exist
            let file_path = format!("{}/current_positions.json", data_dir);
            if !std::path::Path::new(&file_path).exists() {
                return Ok(Vec::new());
            }
            let content = tokio::fs::read_to_string(&file_path).await?;
            let positions: Vec<Position> = serde_json::from_str(&content)?;
            println!("📂 Loaded {} positions from: {}", positions.len(), file_path);
            return Ok(positions);
        }
        
        let content = tokio::fs::read_to_string(&portfolio_file).await?;
        let data: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(positions_array) = data["current_positions"].as_array() {
            let positions: Vec<Position> = serde_json::from_value(serde_json::Value::Array(positions_array.clone()))?;
            println!("📂 Loaded {} positions from: {}", positions.len(), portfolio_file);
            Ok(positions)
        } else {
            Ok(Vec::new())
        }
    }
}
