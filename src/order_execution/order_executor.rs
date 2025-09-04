use crate::order_execution::order_types::*;
use crate::order_execution::liquidation_manager::LiquidationManager;
use crate::market_data::AssetUniverseManager;
use crate::protobuf::ProtobufStorage;
use reqwest::Client;
use serde_json::Value;
use anyhow::{Result, anyhow};
use chrono::Utc;

pub struct OrderExecutor {
    client: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
    liquidation_manager: LiquidationManager,
    protobuf_storage: ProtobufStorage,
}

impl OrderExecutor {
    pub fn new(
        base_url: String,
        api_key: String,
        secret_key: String,
        profit_target_percentage: f64,
        stop_loss_percentage: f64,
        starting_portfolio_value: f64,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
            secret_key,
            liquidation_manager: LiquidationManager::new(
                profit_target_percentage,
                stop_loss_percentage,
                starting_portfolio_value,
            ),
            protobuf_storage: ProtobufStorage::new("trading_data.pb"),
        }
    }

    /// Execute orders based on strategy recommendations and liquidation triggers
    pub async fn execute_orders_from_analysis(
        &self,
        data_dir: &str,
    ) -> Result<Vec<OrderExecutionResult>> {
        let mut results = Vec::new();

        // Load current portfolio data
        let portfolio_data = self.load_portfolio_data(data_dir).await?;
        
        // Check market hours first
        let market_status = &portfolio_data["trading_account"]["market_status"];
        let is_market_open = market_status["is_open"].as_bool().unwrap_or(false);
        
        if !is_market_open {
            println!("⚠️ MARKET IS CLOSED - Orders will be queued for next market open");
            println!("   Next market open: {}", market_status["next_open"].as_str().unwrap_or("Unknown"));
            println!("   Current time: {}", market_status["current_time"].as_str().unwrap_or("Unknown"));
            
            // Return early with market closed message
            return Ok(vec![OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some("Market is closed - orders will be queued for next market open".to_string()),
                alpaca_response: None,
                execution_time: chrono::Utc::now(),
            }]);
        }
        
        // Check trading permissions
        let trading_permissions = &portfolio_data["trading_account"]["trading_permissions"];
        let can_trade = trading_permissions["can_trade"].as_bool().unwrap_or(false);
        
        if !can_trade {
            println!("⚠️ TRADING NOT ALLOWED - Account trading is disabled");
            println!("   Can trade: {}", can_trade);
            println!("   Can crypto: {}", trading_permissions["can_crypto"].as_bool().unwrap_or(false));
            println!("   Can margin: {}", trading_permissions["can_margin"].as_bool().unwrap_or(false));
            println!("   Can short: {}", trading_permissions["can_short"].as_bool().unwrap_or(false));
            
            return Ok(vec![OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some("Trading is not allowed on this account".to_string()),
                alpaca_response: None,
                execution_time: chrono::Utc::now(),
            }]);
        }
        
        let current_portfolio_value = portfolio_data["trading_account"]["account_info"]["portfolio_value"]
            .as_str()
            .unwrap_or("100000")
            .parse::<f64>()
            .unwrap_or(100000.0);

        // Check for liquidation triggers first
        let liquidation_triggers = self.liquidation_manager
            .analyze_liquidation_needs(data_dir, current_portfolio_value)
            .await?;

        if !liquidation_triggers.is_empty() {
            println!("🚨 LIQUIDATION TRIGGERS DETECTED:");
            println!("{}", self.liquidation_manager.get_liquidation_summary(&liquidation_triggers));
            
            // Execute liquidation orders
            for trigger in liquidation_triggers {
                let result = self.execute_liquidation_order(&trigger, data_dir).await?;
                results.push(result);
            }
        }

        // Load strategy recommendations
        let strategy_recommendations = self.load_strategy_recommendations(data_dir).await?;
        
        // Load market regime analysis
        let market_regime = self.load_market_regime_analysis(data_dir).await?;

        // Execute strategy-based orders
        for recommendation in strategy_recommendations {
            let result = self.execute_strategy_order(&recommendation, &market_regime, data_dir).await?;
            results.push(result);
        }

        // Store execution signals in protobuf
        self.store_execution_signals(&results).await?;

        Ok(results)
    }

    /// Execute a liquidation order
    async fn execute_liquidation_order(
        &self,
        trigger: &LiquidationTrigger,
        data_dir: &str,
    ) -> Result<OrderExecutionResult> {
        println!("🔄 Executing liquidation order for {}: {}", trigger.symbol, trigger.reason);

        // Load current positions to get quantity
        let positions = AssetUniverseManager::load_positions(data_dir).await?;
        let position = positions.iter()
            .find(|p| p.symbol == trigger.symbol)
            .ok_or_else(|| anyhow!("Position not found for symbol: {}", trigger.symbol))?;

        let quantity = position.qty.parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse quantity: {}", e))?;

        if quantity <= 0.0 {
            return Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some("No position to liquidate".to_string()),
                alpaca_response: None,
                execution_time: Utc::now(),
            });
        }

        // Create sell order
        let order_request = AlpacaOrderRequest {
            symbol: trigger.symbol.clone(),
            qty: Some(quantity.to_string()),
            notional: None,
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: Some(false),
            client_order_id: Some(format!("LIQUIDATION_{}_{}", trigger.symbol, Utc::now().timestamp())),
            order_class: None,
            take_profit: None,
            stop_loss: None,
        };

        self.execute_order(order_request, &format!("Liquidation: {}", trigger.reason)).await
    }

    /// Execute a strategy-based order
    async fn execute_strategy_order(
        &self,
        recommendation: &Value,
        market_regime: &Value,
        data_dir: &str,
    ) -> Result<OrderExecutionResult> {
        let symbol = recommendation["symbol"].as_str()
            .ok_or_else(|| anyhow!("Missing symbol in recommendation"))?;
        
        let action = recommendation["action"].as_str()
            .ok_or_else(|| anyhow!("Missing action in recommendation"))?;
        
        let confidence = recommendation["confidence_score"].as_f64()
            .unwrap_or(0.0);

        // Only execute high-confidence recommendations
        if confidence < 0.7 {
            return Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some(format!("Confidence too low: {:.2}", confidence)),
                alpaca_response: None,
                execution_time: Utc::now(),
            });
        }

        // Check market regime compatibility
        let regime = market_regime["market_regime"].as_str().unwrap_or("unknown");
        if !self.is_regime_compatible(action, regime) {
            return Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some(format!("Action '{}' not compatible with regime '{}'", action, regime)),
                alpaca_response: None,
                execution_time: Utc::now(),
            });
        }

        let side = match action {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => return Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some(format!("Unknown action: {}", action)),
                alpaca_response: None,
                execution_time: Utc::now(),
            }),
        };

        // Calculate position size based on confidence and available cash
        let position_size = self.calculate_position_size(symbol, confidence, data_dir).await?;
        
        if position_size <= 0.0 {
            return Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some("Insufficient funds or invalid position size".to_string()),
                alpaca_response: None,
                execution_time: Utc::now(),
            });
        }

        // Create order request
        let order_request = AlpacaOrderRequest {
            symbol: symbol.to_string(),
            qty: Some(position_size.to_string()),
            notional: None,
            side,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: Some(false),
            client_order_id: Some(format!("STRATEGY_{}_{}", symbol, Utc::now().timestamp())),
            order_class: None,
            take_profit: None,
            stop_loss: None,
        };

        let reason = format!("Strategy: {} (confidence: {:.2}, regime: {})", action, confidence, regime);
        self.execute_order(order_request, &reason).await
    }

    /// Execute an order via Alpaca API
    async fn execute_order(
        &self,
        order_request: AlpacaOrderRequest,
        reason: &str,
    ) -> Result<OrderExecutionResult> {
        let url = format!("{}/v2/orders", self.base_url);
        
        println!("📤 Sending order to Alpaca: {} {} {} shares of {}", 
            order_request.side, 
            order_request.qty.as_ref().unwrap_or(&"N/A".to_string()),
            order_request.order_type,
            order_request.symbol
        );

        let response = self.client
            .post(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .json(&order_request)
            .send()
            .await?;

        let execution_time = Utc::now();

        if response.status().is_success() {
            let alpaca_response: AlpacaOrderResponse = response.json().await?;
            
            println!("✅ Order executed successfully: ID {}", alpaca_response.id);
            println!("   Reason: {}", reason);
            
            Ok(OrderExecutionResult {
                success: true,
                order_id: Some(alpaca_response.id.clone()),
                error_message: None,
                alpaca_response: Some(alpaca_response),
                execution_time,
            })
        } else {
            let error_text = response.text().await?;
            let error_message = format!("Alpaca API error: {}", error_text);
            
            println!("❌ Order execution failed: {}", error_message);
            
            Ok(OrderExecutionResult {
                success: false,
                order_id: None,
                error_message: Some(error_message),
                alpaca_response: None,
                execution_time,
            })
        }
    }

    /// Load portfolio data from JSON file
    async fn load_portfolio_data(&self, data_dir: &str) -> Result<Value> {
        let file_path = format!("{}/trading_portfolio.json", data_dir);
        let content = tokio::fs::read_to_string(&file_path).await?;
        let data: Value = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Load strategy recommendations
    async fn load_strategy_recommendations(&self, data_dir: &str) -> Result<Vec<Value>> {
        let file_path = format!("{}/enhanced_strategy_recommendations.json", data_dir);
        let content = tokio::fs::read_to_string(&file_path).await?;
        let data: Value = serde_json::from_str(&content)?;
        
        if let Some(recommendations) = data["recommendations"].as_array() {
            Ok(recommendations.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Load market regime analysis
    async fn load_market_regime_analysis(&self, data_dir: &str) -> Result<Value> {
        let file_path = format!("{}/market_regime_analysis.json", data_dir);
        let content = tokio::fs::read_to_string(&file_path).await?;
        let data: Value = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Check if action is compatible with market regime
    fn is_regime_compatible(&self, action: &str, regime: &str) -> bool {
        match regime {
            "bull_market" => action == "buy",
            "bear_market" => action == "sell",
            "sideways_market" => true, // Both buy and sell can work
            "high_volatility" => action == "sell", // Reduce risk
            "low_volatility" => action == "buy", // Accumulate
            "crisis" => action == "sell", // Defensive
            "recovery" => action == "buy", // Growth
            "consolidation" => true, // Both can work
            "momentum" => action == "buy", // Follow trend
            "mean_reversion" => action == "sell", // Contrarian
            _ => true, // Default to allowing all actions
        }
    }

    /// Calculate position size based on confidence and available funds
    async fn calculate_position_size(&self, symbol: &str, confidence: f64, data_dir: &str) -> Result<f64> {
        let portfolio_data = self.load_portfolio_data(data_dir).await?;
        
        // Fix: Get cash balance correctly from the nested structure
        let cash_balance = if let Some(cash_str) = portfolio_data["trading_account"]["account_info"]["cash"].as_str() {
            cash_str.parse::<f64>().unwrap_or(0.0)
        } else if let Some(cash_num) = portfolio_data["trading_account"]["account_info"]["cash"].as_f64() {
            cash_num
        } else {
            0.0
        };

        // Also check buying_power field as fallback
        let buying_power = if let Some(bp_str) = portfolio_data["trading_account"]["account_info"]["buying_power"].as_str() {
            bp_str.parse::<f64>().unwrap_or(0.0)
        } else if let Some(bp_num) = portfolio_data["trading_account"]["account_info"]["buying_power"].as_f64() {
            bp_num
        } else {
            0.0
        };

        // Use the higher of cash or buying_power
        let available_funds = cash_balance.max(buying_power);
        
        if available_funds <= 0.0 {
            return Ok(0.0);
        }

        // Get current price for the symbol
        let current_price = if let Some(market_data) = portfolio_data["market_data"]["symbols"].as_object() {
            if let Some(symbol_data) = market_data.get(symbol) {
                symbol_data["price"].as_f64().unwrap_or(100.0)
            } else {
                100.0 // Default price if symbol not found
            }
        } else {
            100.0 // Default price
        };

        // Use 10% of available funds, scaled by confidence
        let base_allocation = available_funds * 0.10;
        let confidence_scaled = base_allocation * confidence;
        
        // Calculate shares based on actual current price
        let shares = (confidence_scaled / current_price).floor();
        
        Ok(shares.max(1.0)) // Minimum 1 share
    }

    /// Store execution signals in protobuf storage
    async fn store_execution_signals(&self, results: &[OrderExecutionResult]) -> Result<()> {
        for result in results {
            if result.success {
                if let Some(_order_id) = &result.order_id {
                    self.protobuf_storage.append_execution_signal(
                        "order_executed",
                        "STRATEGY", // Will be more specific in real implementation
                        1.0, // strength
                    )?;
                }
            } else {
                if let Some(_error) = &result.error_message {
                    self.protobuf_storage.append_execution_signal(
                        "order_failed",
                        "STRATEGY",
                        0.0, // strength
                    )?;
                }
            }
        }
        Ok(())
    }
}
