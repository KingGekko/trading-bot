use anyhow::{anyhow, Result};
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Alpaca account types as of August 2025
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "premium")]
    Premium,
    #[serde(rename = "enterprise")]
    Enterprise,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Basic => write!(f, "Basic"),
            AccountType::Full => write!(f, "Full"),
            AccountType::Premium => write!(f, "Premium"),
            AccountType::Enterprise => write!(f, "Enterprise"),
        }
    }
}

/// Account status and trading permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    pub account_blocked: bool,
    pub account_closed: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_number: String,
    pub crypto_status: Option<String>,
    pub options_approved_level: Option<i32>,
    pub options_trading_level: Option<i32>,
}

/// Account verification result
#[derive(Debug, Clone)]
pub struct AccountVerification {
    pub account_type: AccountType,
    pub account_status: AccountStatus,
    pub available_features: Vec<String>,
    pub trading_permissions: TradingPermissions,
    pub data_access: DataAccess,
    pub is_verified: bool,
    pub verification_message: String,
}

/// Trading permissions based on account type
#[derive(Debug, Clone)]
pub struct TradingPermissions {
    pub can_trade_stocks: bool,
    pub can_trade_crypto: bool,
    pub can_trade_options: bool,
    pub can_trade_forex: bool,
    pub can_trade_futures: bool,
    pub can_short: bool,
    pub can_margin: bool,
    pub can_day_trade: bool,
    pub can_after_hours: bool,
    pub can_pre_market: bool,
}

/// Data access permissions based on account type
#[derive(Debug, Clone)]
pub struct DataAccess {
    pub market_data_feed: String,
    pub real_time_quotes: bool,
    pub real_time_trades: bool,
    pub real_time_bars: bool,
    pub options_data: bool,
    pub crypto_data: bool,
    pub news_data: bool,
    pub fundamental_data: bool,
    pub historical_data: bool,
}

/// Alpaca account information from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaAccount {
    pub id: String,
    pub account_number: String,
    pub status: String,
    pub crypto_status: Option<String>,
    pub currency: String,
    pub buying_power: String,
    pub regt_buying_power: String,
    pub daytrading_buying_power: String,
    pub non_marginable_buying_power: String,
    pub cash: String,
    pub accrued_fees: String,
    pub pending_transfer_out: Option<String>,
    pub pending_transfer_in: Option<String>,
    pub portfolio_value: String,
    pub pattern_day_trader: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_blocked: bool,
    pub created_at: String,
    pub trade_suspended_by_user: bool,
    pub multiplier: String,
    pub shorting_enabled: bool,
    pub equity: String,
    pub last_equity: String,
    pub long_market_value: String,
    pub short_market_value: String,
    pub initial_margin: String,
    pub maintenance_margin: String,
    pub last_maintenance_margin: String,
    pub sma: String,
    pub daytrade_count: i32,
    pub options_approved_level: Option<i32>,
    pub options_trading_level: Option<i32>,
    pub account_type: Option<String>,
}

/// Account verifier for Alpaca API
pub struct AccountVerifier {
    client: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
}

impl AccountVerifier {
    /// Create a new account verifier
    pub fn new(api_key: String, secret_key: String, is_paper: bool) -> Self {
        let base_url = if is_paper {
            "https://paper-api.alpaca.markets".to_string()
        } else {
            "https://api.alpaca.markets".to_string()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            api_key,
            secret_key,
        }
    }

    /// Verify account and determine available features
    pub async fn verify_account(&self) -> Result<AccountVerification> {
        info!("🔍 Verifying Alpaca account and permissions...");

        // Get account information
        let account = self.get_account_info().await?;
        
        // Determine account type
        let account_type = self.determine_account_type(&account)?;
        
        // Check account status
        let account_status = self.extract_account_status(&account);
        
        // Determine trading permissions
        let trading_permissions = self.determine_trading_permissions(&account, &account_type);
        
        // Determine data access
        let data_access = self.determine_data_access(&account_type);
        
        // Check if account is verified and ready for trading
        let (is_verified, verification_message) = self.check_account_verification(
            &account_status, 
            &trading_permissions
        );

        // Determine available features
        let available_features = self.determine_available_features(
            &account_type, 
            &trading_permissions, 
            &data_access
        );

        let verification = AccountVerification {
            account_type,
            account_status,
            available_features,
            trading_permissions,
            data_access,
            is_verified,
            verification_message,
        };

        info!("✅ Account verification completed");
        info!("📊 Account Type: {}", verification.account_type);
        info!("🔓 Available Features: {}", verification.available_features.join(", "));
        
        Ok(verification)
    }

    /// Get account information from Alpaca API
    async fn get_account_info(&self) -> Result<AlpacaAccount> {
        let url = format!("{}/v2/account", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get account info: {} - {}", status, error_text));
        }

        let account: AlpacaAccount = response.json().await?;
        Ok(account)
    }

    /// Determine account type based on account information
    fn determine_account_type(&self, account: &AlpacaAccount) -> Result<AccountType> {
        info!("🔍 Determining account type...");
        info!("   Options approved level: {:?}", account.options_approved_level);
        info!("   Options trading level: {:?}", account.options_trading_level);
        info!("   Crypto status: {:?}", account.crypto_status);
        info!("   Shorting enabled: {}", account.shorting_enabled);
        info!("   Pattern day trader: {}", account.pattern_day_trader);
        
        // Check account_type field first (if available)
        if let Some(account_type_str) = &account.account_type {
            info!("   Account type from API: {}", account_type_str);
            match account_type_str.to_lowercase().as_str() {
                "basic" => return Ok(AccountType::Basic),
                "full" => return Ok(AccountType::Full),
                "premium" => return Ok(AccountType::Premium),
                "enterprise" => return Ok(AccountType::Enterprise),
                _ => {}
            }
        }

        // Fallback: determine by features and permissions
        if account.options_approved_level.is_some() && account.options_trading_level.is_some() {
            info!("   ✅ Account has options approval - classifying as Full");
            // If you have options approval, you're at least a Full account
            // Pattern day trader status is separate from account tier
            Ok(AccountType::Full)
        } else if account.crypto_status.is_some() {
            info!("   ✅ Account has crypto access - classifying as Basic");
            Ok(AccountType::Basic)
        } else {
            // Default to Basic if we can't determine
            warn!("⚠️ Could not determine account type, defaulting to Basic");
            Ok(AccountType::Basic)
        }
    }

    /// Extract account status from account information
    fn extract_account_status(&self, account: &AlpacaAccount) -> AccountStatus {
        AccountStatus {
            account_blocked: account.account_blocked,
            account_closed: false, // Not directly available in API response
            trading_blocked: account.trading_blocked,
            transfers_blocked: account.transfers_blocked,
            account_number: account.account_number.clone(),
            crypto_status: account.crypto_status.clone(),
            options_approved_level: account.options_approved_level,
            options_trading_level: account.options_trading_level,
        }
    }

    /// Determine trading permissions based on account type and status
    fn determine_trading_permissions(&self, account: &AlpacaAccount, account_type: &AccountType) -> TradingPermissions {
        match account_type {
            AccountType::Basic => TradingPermissions {
                can_trade_stocks: true,
                can_trade_crypto: account.crypto_status.is_some(),
                can_trade_options: account.options_approved_level.is_some(), // Enable if approved
                can_trade_forex: false,
                can_trade_futures: false,
                can_short: account.shorting_enabled, // Enable if available
                can_margin: account.multiplier.parse::<i32>().unwrap_or(1) > 1, // Enable if multiplier > 1
                can_day_trade: account.pattern_day_trader,
                can_after_hours: false,
                can_pre_market: false,
            },
            AccountType::Full => TradingPermissions {
                can_trade_stocks: true,
                can_trade_crypto: account.crypto_status.is_some(),
                can_trade_options: account.options_approved_level.is_some(),
                can_trade_forex: false,
                can_trade_futures: false,
                can_short: account.shorting_enabled,
                can_margin: true,
                can_day_trade: account.pattern_day_trader,
                can_after_hours: true,
                can_pre_market: true,
            },
            AccountType::Premium => TradingPermissions {
                can_trade_stocks: true,
                can_trade_crypto: account.crypto_status.is_some(),
                can_trade_options: account.options_approved_level.is_some(),
                can_trade_forex: true,
                can_trade_futures: false,
                can_short: account.shorting_enabled,
                can_margin: true,
                can_day_trade: account.pattern_day_trader,
                can_after_hours: true,
                can_pre_market: true,
            },
            AccountType::Enterprise => TradingPermissions {
                can_trade_stocks: true,
                can_trade_crypto: account.crypto_status.is_some(),
                can_trade_options: account.options_approved_level.is_some(),
                can_trade_forex: true,
                can_trade_futures: true,
                can_short: account.shorting_enabled,
                can_margin: true,
                can_day_trade: account.pattern_day_trader,
                can_after_hours: true,
                can_pre_market: true,
            },
        }
    }

    /// Determine data access based on account type
    fn determine_data_access(&self, account_type: &AccountType) -> DataAccess {
        match account_type {
            AccountType::Basic => DataAccess {
                market_data_feed: "test".to_string(), // Use test feed for paper trading
                real_time_quotes: true,
                real_time_trades: true,
                real_time_bars: true,
                options_data: false,
                crypto_data: true,
                news_data: true, // Enable news data for paper trading
                fundamental_data: false,
                historical_data: true,
            },
            AccountType::Full => DataAccess {
                market_data_feed: "sip".to_string(),
                real_time_quotes: true,
                real_time_trades: true,
                real_time_bars: true,
                options_data: true,
                crypto_data: true,
                news_data: true,
                fundamental_data: true,
                historical_data: true,
            },
            AccountType::Premium => DataAccess {
                market_data_feed: "sip".to_string(),
                real_time_quotes: true,
                real_time_trades: true,
                real_time_bars: true,
                options_data: true,
                crypto_data: true,
                news_data: true,
                fundamental_data: true,
                historical_data: true,
            },
            AccountType::Enterprise => DataAccess {
                market_data_feed: "sip".to_string(),
                real_time_quotes: true,
                real_time_trades: true,
                real_time_bars: true,
                options_data: true,
                crypto_data: true,
                news_data: true,
                fundamental_data: true,
                historical_data: true,
            },
        }
    }

    /// Check if account is verified and ready for trading
    fn check_account_verification(
        &self, 
        account_status: &AccountStatus, 
        _trading_permissions: &TradingPermissions
    ) -> (bool, String) {
        let mut issues = Vec::new();

        if account_status.account_blocked {
            issues.push("Account is blocked".to_string());
        }
        if account_status.trading_blocked {
            issues.push("Trading is blocked".to_string());
        }
        if account_status.transfers_blocked {
            issues.push("Transfers are blocked".to_string());
        }

        if issues.is_empty() {
            (true, "Account verified and ready for trading".to_string())
        } else {
            (false, format!("Account verification failed: {}", issues.join(", ")))
        }
    }

    /// Determine available features based on account type and permissions
    fn determine_available_features(
        &self,
        _account_type: &AccountType,
        trading_permissions: &TradingPermissions,
        data_access: &DataAccess,
    ) -> Vec<String> {
        let mut features = vec!["Stocks".to_string()];

        if trading_permissions.can_trade_crypto {
            features.push("Crypto".to_string());
        }
        if trading_permissions.can_trade_options {
            features.push("Options".to_string());
        }
        if trading_permissions.can_trade_forex {
            features.push("Forex".to_string());
        }
        if trading_permissions.can_trade_futures {
            features.push("Futures".to_string());
        }
        if trading_permissions.can_short {
            features.push("Short Selling".to_string());
        }
        if trading_permissions.can_margin {
            features.push("Margin Trading".to_string());
        }
        if trading_permissions.can_day_trade {
            features.push("Day Trading".to_string());
        }
        if data_access.news_data {
            features.push("News Data".to_string());
        }
        if data_access.fundamental_data {
            features.push("Fundamental Data".to_string());
        }

        features
    }

    /// Check if a specific feature is available
    pub fn is_feature_available(&self, verification: &AccountVerification, feature: &str) -> bool {
        verification.available_features.iter().any(|f| f.to_lowercase() == feature.to_lowercase())
    }

    /// Get recommended market data feed based on account type
    pub fn get_recommended_feed(&self, verification: &AccountVerification) -> String {
        verification.data_access.market_data_feed.clone()
    }

    /// Validate stream types against account permissions
    pub fn validate_stream_types(
        &self, 
        verification: &AccountVerification, 
        requested_streams: &[String]
    ) -> Result<Vec<String>> {
        println!("🔍 Validating stream types...");
        println!("   Requested streams: {:?}", requested_streams);
        println!("   Account type: {:?}", verification.account_type);
        println!("   Trading permissions: can_trade_crypto={}, can_trade_options={}, can_short={}, can_margin={}", 
               verification.trading_permissions.can_trade_crypto,
               verification.trading_permissions.can_trade_options,
               verification.trading_permissions.can_short,
               verification.trading_permissions.can_margin);
        println!("   Data access: crypto_data={}, options_data={}, news_data={}", 
               verification.data_access.crypto_data,
               verification.data_access.options_data,
               verification.data_access.news_data);
        
        info!("🔍 Validating stream types...");
        info!("   Requested streams: {:?}", requested_streams);
        info!("   Account type: {:?}", verification.account_type);
        info!("   Trading permissions: can_trade_crypto={}, can_trade_options={}, can_short={}, can_margin={}", 
               verification.trading_permissions.can_trade_crypto,
               verification.trading_permissions.can_trade_options,
               verification.trading_permissions.can_short,
               verification.trading_permissions.can_margin);
        info!("   Data access: crypto_data={}, options_data={}, news_data={}", 
               verification.data_access.crypto_data,
               verification.data_access.options_data,
               verification.data_access.news_data);
        
        let mut valid_streams = Vec::new();
        let mut invalid_streams = Vec::new();

        for stream in requested_streams {
            match stream.to_lowercase().as_str() {
                "stocks" => {
                    println!("   ✅ Stocks stream: Always available");
                    info!("   ✅ Stocks stream: Always available");
                    valid_streams.push(stream.clone());
                }
                "crypto" => {
                    if verification.trading_permissions.can_trade_crypto {
                        println!("   ✅ Crypto stream: Available");
                        info!("   ✅ Crypto stream: Available");
                        valid_streams.push(stream.clone());
                    } else {
                        println!("   ❌ Crypto stream: Not available (requires crypto-enabled account)");
                        info!("   ❌ Crypto stream: Not available (requires crypto-enabled account)");
                        invalid_streams.push(format!("{} (requires crypto-enabled account)", stream));
                    }
                }
                "options" => {
                    if verification.trading_permissions.can_trade_options {
                        println!("   ✅ Options stream: Available");
                        info!("   ✅ Options stream: Available");
                        valid_streams.push(stream.clone());
                    } else {
                        println!("   ❌ Options stream: Not available (requires options-enabled account)");
                        info!("   ❌ Options stream: Not available (requires options-enabled account)");
                        invalid_streams.push(format!("{} (requires options-enabled account)", stream));
                    }
                }
                "news" => {
                    if verification.data_access.news_data {
                        println!("   ✅ News stream: Available");
                        info!("   ✅ News stream: Available");
                        valid_streams.push(stream.clone());
                    } else {
                        println!("   ❌ News stream: Not available (requires news data access)");
                        info!("   ❌ News stream: Not available (requires news data access)");
                        invalid_streams.push(format!("{} (requires news data access)", stream));
                    }
                }
                "trade_updates" => {
                    println!("   ✅ Trade updates stream: Always available");
                    info!("   ✅ Trade updates stream: Always available");
                    valid_streams.push(stream.clone());
                }
                "account_updates" => {
                    println!("   ✅ Account updates stream: Always available");
                    info!("   ✅ Account updates stream: Always available");
                    valid_streams.push(stream.clone());
                }
                "order_updates" => {
                    println!("   ✅ Order updates stream: Always available");
                    info!("   ✅ Order updates stream: Always available");
                    valid_streams.push(stream.clone());
                }
                _ => {
                    warn!("⚠️ Unknown stream type: {}", stream);
                    invalid_streams.push(format!("{} (unknown stream type)", stream));
                }
            }
        }

        if !invalid_streams.is_empty() {
            warn!("⚠️ Some requested streams are not available: {}", invalid_streams.join(", "));
        }

        println!("   Final valid streams: {:?}", valid_streams);
        info!("   Final valid streams: {:?}", valid_streams);

        if valid_streams.is_empty() {
            return Err(anyhow!("No valid streams available for this account type"));
        }

        Ok(valid_streams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_display() {
        assert_eq!(AccountType::Basic.to_string(), "Basic");
        assert_eq!(AccountType::Full.to_string(), "Full");
        assert_eq!(AccountType::Premium.to_string(), "Premium");
        assert_eq!(AccountType::Enterprise.to_string(), "Enterprise");
    }

    #[test]
    fn test_trading_permissions_basic() {
        let permissions = TradingPermissions {
            can_trade_stocks: true,
            can_trade_crypto: true,
            can_trade_options: false,
            can_trade_forex: false,
            can_trade_futures: false,
            can_short: false,
            can_margin: false,
            can_day_trade: false,
            can_after_hours: false,
            can_pre_market: false,
        };

        assert!(permissions.can_trade_stocks);
        assert!(permissions.can_trade_crypto);
        assert!(!permissions.can_trade_options);
        assert!(!permissions.can_short);
    }

    #[test]
    fn test_data_access_basic() {
        let data_access = DataAccess {
            market_data_feed: "iex".to_string(),
            real_time_quotes: true,
            real_time_trades: true,
            real_time_bars: true,
            options_data: false,
            crypto_data: true,
            news_data: false,
            fundamental_data: false,
            historical_data: true,
        };

        assert_eq!(data_access.market_data_feed, "iex");
        assert!(data_access.real_time_quotes);
        assert!(!data_access.options_data);
        assert!(data_access.crypto_data);
    }
}
