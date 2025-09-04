use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc, Datelike};
use chrono_tz::America::New_York;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use log::{info, warn, error};

/// Trading account information from Alpaca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAccount {
    pub id: String,
    pub account_number: String,
    pub status: String,
    pub crypto_status: Option<String>,
    pub currency: String,
    pub cash: String,
    pub portfolio_value: String,
    pub non_marginable_buying_power: String,
    pub accrued_fees: String,
    pub pending_transfer_in: Option<String>, // Made optional
    pub pending_transfer_out: Option<String>, // Made optional
    pub pattern_day_trader: bool,
    pub trade_suspended_by_user: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_blocked: bool,
    pub shorting_enabled: bool,
    pub long_market_value: String,
    pub short_market_value: String,
    pub equity: String,
    pub last_equity: String,
    pub multiplier: String,
    pub daytrade_count: i32,
    pub daytrading_buying_power: String,
    pub regt_buying_power: String,
    pub initial_margin: String,
    pub maintenance_margin: String,
    pub sma: String,
    pub last_maintenance_margin: String,
    pub created_at: String,
    
    // New fields from actual API response
    pub admin_configurations: Option<serde_json::Value>,
    pub user_configurations: Option<serde_json::Value>,
    pub options_approved_level: Option<i32>,
    pub options_trading_level: Option<i32>,
    pub buying_power: Option<String>,
    pub effective_buying_power: Option<String>,
    pub options_buying_power: Option<String>,
    pub bod_dtbp: Option<String>,
    pub position_market_value: Option<String>,
    pub balance_asof: Option<String>,
    pub crypto_tier: Option<i32>,
    pub intraday_adjustments: Option<String>,
    pub pending_reg_taf_fees: Option<String>,
}

/// Market hours information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHours {
    pub is_open: bool,
    pub next_open: Option<DateTime<Utc>>,
    pub next_close: Option<DateTime<Utc>>,
    pub current_time: DateTime<Utc>,
    pub market_timezone: String,
    pub trading_hours: String,
}

/// Trading account manager
pub struct TradingAccountManager {
    api_key: String,
    secret_key: String,
    is_paper_trading: bool,
    data_dir: PathBuf,
}

impl TradingAccountManager {
    /// Create a new trading account manager
    pub fn new(api_key: String, secret_key: String, is_paper_trading: bool, data_dir: PathBuf) -> Self {
        Self {
            api_key,
            secret_key,
            is_paper_trading,
            data_dir,
        }
    }

    /// Check if the market is currently open (New York timezone)
    pub async fn is_market_open(&self) -> Result<bool> {
        let ny_time = Utc::now().with_timezone(&New_York);
        let current_time = ny_time.time();
        
        // Market hours: 9:30 AM - 4:00 PM ET (Monday-Friday)
        let market_open = chrono::NaiveTime::from_hms_opt(9, 30, 0)
            .ok_or_else(|| anyhow!("Invalid market open time"))?;
        let market_close = chrono::NaiveTime::from_hms_opt(16, 0, 0)
            .ok_or_else(|| anyhow!("Invalid market close time"))?;
        
        // Check if it's a weekday
        let is_weekday = ny_time.weekday().num_days_from_monday() < 5;
        
        // Check if current time is within market hours
        let is_within_hours = current_time >= market_open && current_time <= market_close;
        
        let is_open = is_weekday && is_within_hours;
        
        info!("🌍 Market Status Check (NY Timezone):");
        info!("   Current NY Time: {}", ny_time.format("%Y-%m-%d %H:%M:%S %Z"));
        info!("   Is Weekday: {}", is_weekday);
        info!("   Within Hours (9:30 AM - 4:00 PM ET): {}", is_within_hours);
        info!("   Market Open: {}", is_open);
        
        Ok(is_open)
    }

    /// Get market hours information
    pub async fn get_market_hours(&self) -> Result<MarketHours> {
        let ny_time = Utc::now().with_timezone(&New_York);
        let current_time = Utc::now();
        
        // Calculate next market open/close
        let mut next_open = None;
        let mut next_close = None;
        
        let ny_date = ny_time.date_naive();
        let market_open_time = chrono::NaiveDateTime::new(
            ny_date,
            chrono::NaiveTime::from_hms_opt(9, 30, 0)
                .ok_or_else(|| anyhow!("Invalid market open time"))?
        );
        let market_close_time = chrono::NaiveDateTime::new(
            ny_date,
            chrono::NaiveTime::from_hms_opt(16, 0, 0)
                .ok_or_else(|| anyhow!("Invalid market close time"))?
        );
        
        // Convert to UTC
        if let Some(ny_tz) = New_York.from_local_datetime(&market_open_time).earliest() {
            next_open = Some(ny_tz.with_timezone(&Utc));
        }
        if let Some(ny_tz) = New_York.from_local_datetime(&market_close_time).earliest() {
            next_close = Some(ny_tz.with_timezone(&Utc));
        }
        
        let is_open = self.is_market_open().await?;
        
        Ok(MarketHours {
            is_open,
            next_open,
            next_close,
            current_time,
            market_timezone: "America/New_York".to_string(),
            trading_hours: "9:30 AM - 4:00 PM ET (Monday-Friday)".to_string(),
        })
    }

    /// Fetch trading account information from Alpaca
    pub async fn fetch_account_info(&self) -> Result<TradingAccount> {
        let client = reqwest::Client::new();
        
        // Determine API endpoint based on paper/live trading
        let base_url = if self.is_paper_trading {
            "https://paper-api.alpaca.markets"
        } else {
            "https://api.alpaca.markets"
        };
        
        let url = format!("{}/v2/account", base_url);
        
        info!("🔍 Fetching account info from: {}", url);
        
        let response = client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;
        
        let status = response.status();
        info!("📡 Response status: {}", status);
        
        if response.status().is_success() {
            let response_text = response.text().await?;
            info!("📨 Raw response: {}", response_text);
            
            // Try to parse as JSON first to see the structure
            match serde_json::from_str::<serde_json::Value>(&response_text) {
                Ok(json_value) => {
                    info!("✅ Successfully parsed as JSON: {}", serde_json::to_string_pretty(&json_value)?);
                }
                Err(e) => {
                    error!("❌ Failed to parse as JSON: {}", e);
                    error!("📄 Raw response text: {}", response_text);
                }
            }
            
            // Now try to deserialize into TradingAccount
            match serde_json::from_str::<TradingAccount>(&response_text) {
                Ok(account) => {
                    info!("✅ Successfully deserialized trading account information");
                    info!("   Account ID: {}", account.id);
                    info!("   Status: {}", account.status);
                    info!("   Equity: ${}", account.equity);
                    info!("   Cash: ${}", account.cash);
                    info!("   Pattern Day Trader: {}", account.pattern_day_trader);
                    
                    Ok(account)
                }
                Err(e) => {
                    error!("❌ Failed to deserialize TradingAccount: {}", e);
                    error!("📄 Raw response text: {}", response_text);
                    Err(anyhow!("Failed to deserialize account info: {}", e))
                }
            }
        } else {
            let error_text = response.text().await?;
            error!("❌ API request failed: Status {} - {}", status, error_text);
            Err(anyhow!("Failed to fetch account info: Status {} - {}", status, error_text))
        }
    }

    /// Save account information to JSON file
    pub async fn save_account_info(&self, account: &TradingAccount, market_hours: &MarketHours) -> Result<()> {
        let filename = "trading_account.json";
        let file_path = self.data_dir.join(filename);
        
        // Create account info section
        let account_info = json!({
            "id": account.id,
            "account_number": account.account_number,
            "status": account.status,
            "crypto_status": account.crypto_status,
            "currency": account.currency,
            "cash": account.cash,
            "portfolio_value": account.portfolio_value,
            "equity": account.equity,
            "last_equity": account.last_equity,
            "non_marginable_buying_power": account.non_marginable_buying_power,
            "daytrading_buying_power": account.daytrading_buying_power,
            "regt_buying_power": account.regt_buying_power,
            "initial_margin": account.initial_margin,
            "maintenance_margin": account.maintenance_margin,
            "multiplier": account.multiplier,
            "daytrade_count": account.daytrade_count,
            "pattern_day_trader": account.pattern_day_trader,
            "shorting_enabled": account.shorting_enabled,
            "long_market_value": account.long_market_value,
            "short_market_value": account.short_market_value,
            "accrued_fees": account.accrued_fees,
            "pending_transfer_in": account.pending_transfer_in.as_deref().unwrap_or("0"),
            "pending_transfer_out": account.pending_transfer_out.as_deref().unwrap_or("0"),
            "trade_suspended_by_user": account.trade_suspended_by_user,
            "trading_blocked": account.trading_blocked,
            "transfers_blocked": account.transfers_blocked,
            "account_blocked": account.account_blocked,
            "created_at": account.created_at
        });
        
        // Create market status section
        let market_status = json!({
            "is_open": market_hours.is_open,
            "current_time": market_hours.current_time.to_rfc3339(),
            "market_timezone": market_hours.market_timezone,
            "trading_hours": market_hours.trading_hours,
            "next_open": market_hours.next_open.map(|dt| dt.to_rfc3339()),
            "next_close": market_hours.next_close.map(|dt| dt.to_rfc3339())
        });
        
        // Create trading permissions section
        let trading_permissions = json!({
            "can_trade": !account.trading_blocked && !account.account_blocked && market_hours.is_open,
            "can_short": account.shorting_enabled,
            "can_margin": account.multiplier != "1",
            "can_crypto": account.crypto_status.as_deref() == Some("ACTIVE"),
            "pattern_day_trader": account.pattern_day_trader
        });
        
        // Create risk management section
        let risk_management = json!({
            "buying_power": account.non_marginable_buying_power,
            "daytrading_buying_power": account.daytrading_buying_power,
            "margin_requirements": {
                "initial": account.initial_margin,
                "maintenance": account.maintenance_margin
            },
            "daytrade_count": account.daytrade_count,
            "max_daytrades": if account.pattern_day_trader { 3 } else { 0 }
        });
        
        // Create main account data structure
        let account_data = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "data_source": if self.is_paper_trading { "alpaca_paper_trading" } else { "alpaca_live_trading" },
            "account_info": account_info,
            "market_status": market_status,
            "trading_permissions": trading_permissions,
            "risk_management": risk_management
        });
        
        // Write to file
        let json_content = serde_json::to_string_pretty(&account_data)?;
        fs::write(file_path, json_content).await?;
        
        info!("📝 Saved trading account information to {}", filename);
        Ok(())
    }

    /// Start continuous account monitoring
    pub async fn start_account_monitoring(&self) -> Result<()> {
        info!("🚀 Starting trading account monitoring...");
        
        // Create data directory if it doesn't exist
        fs::create_dir_all(&self.data_dir).await?;
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Update every minute
        
        // Do initial fetch immediately
        info!("📊 Performing initial account fetch...");
        match self.get_market_hours().await {
            Ok(market_hours) => {
                match self.fetch_account_info().await {
                    Ok(account) => {
                        if let Err(e) = self.save_account_info(&account, &market_hours).await {
                            error!("❌ Failed to save initial account info: {}", e);
                        } else {
                            info!("✅ Initial account info saved successfully");
                        }
                        
                        let market_status = if market_hours.is_open { "🟢 OPEN" } else { "🔴 CLOSED" };
                        info!("📊 Market Status: {} | Initial fetch complete", market_status);
                    }
                    Err(e) => {
                        error!("❌ Failed to fetch initial account info: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to get initial market hours: {}", e);
                return Err(e);
            }
        }
        
        info!("🔄 Starting continuous monitoring loop...");
        
        loop {
            interval.tick().await;
            
            // Check market status
            match self.get_market_hours().await {
                Ok(market_hours) => {
                    // Fetch account information
                    match self.fetch_account_info().await {
                        Ok(account) => {
                            // Save to JSON file
                            if let Err(e) = self.save_account_info(&account, &market_hours).await {
                                error!("❌ Failed to save account info: {}", e);
                            } else {
                                let market_status = if market_hours.is_open { "🟢 OPEN" } else { "🔴 CLOSED" };
                                info!("📊 Market Status: {} | Account updated successfully", market_status);
                            }
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to fetch account info: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to get market hours: {}", e);
                }
            }
        }
    }

    /// Get current account status summary
    pub async fn get_account_summary(&self) -> Result<Value> {
        let market_hours = self.get_market_hours().await?;
        let account = self.fetch_account_info().await?;
        
        let summary = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "market_status": market_hours.is_open,
            "account_status": account.status,
            "equity": account.equity,
            "cash": account.cash,
            "buying_power": account.non_marginable_buying_power,
            "pattern_day_trader": account.pattern_day_trader,
            "can_trade": !account.trading_blocked && !account.account_blocked && market_hours.is_open
        });
        
        Ok(summary)
    }
}
