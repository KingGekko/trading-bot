use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Order side (buy or sell)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

/// Order type for Alpaca API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "market"),
            OrderType::Limit => write!(f, "limit"),
            OrderType::Stop => write!(f, "stop"),
            OrderType::StopLimit => write!(f, "stop_limit"),
            OrderType::TrailingStop => write!(f, "trailing_stop"),
        }
    }
}

/// Time in force for orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeInForce {
    Day,
    Gtc, // Good Till Canceled
    Ioc, // Immediate or Cancel
    Fok, // Fill or Kill
}

/// Order status from Alpaca API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    DoneForDay,
    Canceled,
    Expired,
    Replaced,
    PendingCancel,
    PendingReplace,
    Accepted,
    PendingNew,
    AcceptedForBidding,
    Stopped,
    Rejected,
    Suspended,
    Calculated,
}

/// Alpaca order request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaOrderRequest {
    pub symbol: String,
    pub qty: Option<String>,
    pub notional: Option<String>,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub trail_price: Option<String>,
    pub trail_percent: Option<String>,
    pub extended_hours: Option<bool>,
    pub client_order_id: Option<String>,
    pub order_class: Option<String>,
    pub take_profit: Option<TakeProfit>,
    pub stop_loss: Option<StopLoss>,
}

/// Take profit order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfit {
    pub limit_price: String,
}

/// Stop loss order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLoss {
    pub stop_price: String,
    pub limit_price: Option<String>,
}

/// Alpaca order response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaOrderResponse {
    pub id: String,
    pub client_order_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub submitted_at: String,
    pub filled_at: Option<String>,
    pub expired_at: Option<String>,
    pub canceled_at: Option<String>,
    pub failed_at: Option<String>,
    pub replaced_at: Option<String>,
    pub replaced_by: Option<String>,
    pub replaces: Option<String>,
    pub asset_id: String,
    pub symbol: String,
    pub asset_class: String,
    pub notional: Option<String>,
    pub qty: Option<String>,
    pub filled_qty: Option<String>,
    pub filled_avg_price: Option<String>,
    pub order_class: String,
    pub order_type: OrderType,
    pub order_type_extended: Option<String>,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub status: OrderStatus,
    pub extended_hours: bool,
    pub legs: Option<Vec<AlpacaOrderResponse>>,
    pub trail_percent: Option<String>,
    pub trail_price: Option<String>,
    pub hwm: Option<String>,
}

/// Internal order execution request
#[derive(Debug, Clone)]
pub struct OrderExecutionRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub order_type: OrderType,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub reason: String, // Why this order is being placed
    pub strategy_confidence: f64,
    pub market_regime: String,
}

/// Order execution result
#[derive(Debug, Clone)]
pub struct OrderExecutionResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub error_message: Option<String>,
    pub alpaca_response: Option<AlpacaOrderResponse>,
    pub execution_time: DateTime<Utc>,
}

/// Liquidation trigger conditions
#[derive(Debug, Clone)]
pub struct LiquidationTrigger {
    pub symbol: String,
    pub trigger_type: LiquidationType,
    pub current_price: f64,
    pub target_price: f64,
    pub profit_percentage: f64,
    pub reason: String,
}

/// Types of liquidation triggers
#[derive(Debug, Clone)]
pub enum LiquidationType {
    ProfitTarget,    // 5% profit target reached
    StopLoss,        // Portfolio protection stop loss
    RiskManagement,  // Risk management liquidation
    StrategySignal,  // Strategy recommendation change
}

/// Portfolio position for liquidation analysis
#[derive(Debug, Clone)]
pub struct PositionForLiquidation {
    pub symbol: String,
    pub quantity: f64,
    pub current_price: f64,
    pub average_cost: f64,
    pub market_value: f64,
    pub unrealized_pl: f64,
    pub unrealized_plpc: f64, // Unrealized P&L percentage
}
