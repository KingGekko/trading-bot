pub mod trading_bot {
    tonic::include_proto!("trading_bot");
}

use trading_bot::*;
use prost::Message;
use std::path::Path;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub struct ProtobufStorage {
    pub file_path: String,
}

impl ProtobufStorage {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn save_trading_data(&self, data: &TradingData) -> Result<()> {
        let encoded = data.encode_to_vec();
        std::fs::write(&self.file_path, encoded)?;
        println!("✅ Protobuf data saved to: {}", self.file_path);
        Ok(())
    }

    pub fn load_trading_data(&self) -> Result<TradingData> {
        if !Path::new(&self.file_path).exists() {
            return Ok(TradingData::default());
        }
        
        let encoded = std::fs::read(&self.file_path)?;
        let data = TradingData::decode(encoded.as_slice())?;
        println!("✅ Protobuf data loaded from: {}", self.file_path);
        Ok(data)
    }

    pub fn create_sample_data() -> TradingData {
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        TradingData {
            api_keys: vec![
                ApiKey {
                    id: 1,
                    provider: "alpaca".to_string(),
                    api_key: "test_api_key_123".to_string(),
                    environment: "paper".to_string(),
                    is_active: true,
                    created_at: Some(timestamp.clone()),
                }
            ],
            assets: vec![
                Asset {
                    id: 1,
                    symbol: "AAPL".to_string(),
                    name: "Apple Inc.".to_string(),
                    asset_class: "stock".to_string(),
                    exchange: "NASDAQ".to_string(),
                    is_tradable: true,
                    last_price: 150.0,
                    created_at: Some(timestamp.clone()),
                }
            ],
            trades: vec![
                Trade {
                    id: 1,
                    asset_symbol: "AAPL".to_string(),
                    side: "buy".to_string(),
                    quantity: 10.0,
                    price: 150.0,
                    total_value: 1500.0,
                    trade_time: Some(timestamp.clone()),
                }
            ],
            ollama_receipts: vec![
                OllamaReceipt {
                    id: 1,
                    model: "llama3.2".to_string(),
                    prompt: "Analyze AAPL stock".to_string(),
                    response: "AAPL shows strong fundamentals".to_string(),
                    success: true,
                    created_at: Some(timestamp.clone()),
                }
            ],
            market_regime_analyses: vec![
                MarketRegimeAnalysis {
                    id: 1,
                    market_regime: "bull_market".to_string(),
                    confidence_level: 0.85,
                    analysis_time: Some(timestamp.clone()),
                }
            ],
            strategy_recommendations: vec![
                StrategyRecommendation {
                    id: 1,
                    asset_symbol: "AAPL".to_string(),
                    action: "buy".to_string(),
                    target_price: 160.0,
                    confidence_score: 0.8,
                    created_at: Some(timestamp.clone()),
                }
            ],
            execution_signals: vec![
                ExecutionSignal {
                    id: 1,
                    signal_type: "buy".to_string(),
                    asset_symbol: "AAPL".to_string(),
                    strength: 0.8,
                    is_triggered: false,
                    created_at: Some(timestamp.clone()),
                }
            ],
            portfolio_snapshots: vec![
                PortfolioSnapshot {
                    id: 1,
                    total_value: 100000.0,
                    cash_balance: 50000.0,
                    portfolio_value: 100000.0,
                    snapshot_time: Some(timestamp.clone()),
                }
            ],
            last_updated: Some(timestamp),
            version: "1.0.0".to_string(),
        }
    }

    pub fn display_detailed_data(&self) -> Result<()> {
        let data = self.load_trading_data()?;
        
        println!("\n📊 DETAILED PROTOBUF DATA VIEW");
        println!("{}", "=".repeat(60));
        
        // API Keys
        println!("\n🔑 API KEYS ({}):", data.api_keys.len());
        for key in &data.api_keys {
            println!("   • ID: {} | Provider: {} | Environment: {} | Active: {}", 
                key.id, key.provider, key.environment, key.is_active);
            if let Some(ts) = &key.created_at {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Assets
        println!("\n📈 ASSETS ({}):", data.assets.len());
        for asset in &data.assets {
            println!("   • {} ({}) | Class: {} | Exchange: {} | Price: ${:.2} | Tradable: {}", 
                asset.symbol, asset.name, asset.asset_class, asset.exchange, asset.last_price, asset.is_tradable);
            if let Some(ts) = &asset.created_at {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Trades
        println!("\n💼 TRADES ({}):", data.trades.len());
        for trade in &data.trades {
            println!("   • {} {} {} shares @ ${:.2} | Total: ${:.2}", 
                trade.side.to_uppercase(), trade.quantity, trade.asset_symbol, trade.price, trade.total_value);
            if let Some(ts) = &trade.trade_time {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Time: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Ollama Receipts
        println!("\n🤖 OLLAMA RECEIPTS ({}):", data.ollama_receipts.len());
        for receipt in &data.ollama_receipts {
            println!("   • Model: {} | Success: {}", receipt.model, receipt.success);
            println!("     Prompt: {}", receipt.prompt);
            println!("     Response: {}", receipt.response);
            if let Some(ts) = &receipt.created_at {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Market Regime Analyses
        println!("\n📊 MARKET REGIME ANALYSES ({}):", data.market_regime_analyses.len());
        for analysis in &data.market_regime_analyses {
            println!("   • Regime: {} | Confidence: {:.1}%", 
                analysis.market_regime, analysis.confidence_level * 100.0);
            if let Some(ts) = &analysis.analysis_time {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Time: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Strategy Recommendations
        println!("\n🎯 STRATEGY RECOMMENDATIONS ({}):", data.strategy_recommendations.len());
        for rec in &data.strategy_recommendations {
            println!("   • {} {} | Target: ${:.2} | Confidence: {:.1}%", 
                rec.action.to_uppercase(), rec.asset_symbol, rec.target_price, rec.confidence_score * 100.0);
            if let Some(ts) = &rec.created_at {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Execution Signals
        println!("\n🚦 EXECUTION SIGNALS ({}):", data.execution_signals.len());
        for signal in &data.execution_signals {
            println!("   • {} {} | Strength: {:.1}% | Triggered: {}", 
                signal.signal_type.to_uppercase(), signal.asset_symbol, signal.strength * 100.0, signal.is_triggered);
            if let Some(ts) = &signal.created_at {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Created: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Portfolio Snapshots
        println!("\n💰 PORTFOLIO SNAPSHOTS ({}):", data.portfolio_snapshots.len());
        for snapshot in &data.portfolio_snapshots {
            println!("   • Total Value: ${:.2} | Cash: ${:.2} | Portfolio: ${:.2}", 
                snapshot.total_value, snapshot.cash_balance, snapshot.portfolio_value);
            if let Some(ts) = &snapshot.snapshot_time {
                let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
                println!("     Time: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
        
        // Metadata
        println!("\n📋 METADATA:");
        println!("   • Version: {}", data.version);
        if let Some(ts) = &data.last_updated {
            let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
            println!("   • Last Updated: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        println!("\n🎉 DETAILED DATA VIEW COMPLETE!");
        Ok(())
    }

    pub fn export_to_json(&self, json_file_path: &str) -> Result<()> {
        let data = self.load_trading_data()?;
        
        // Convert protobuf data to JSON-serializable format
        let json_data = serde_json::json!({
            "api_keys": data.api_keys.iter().map(|key| {
                serde_json::json!({
                    "id": key.id,
                    "provider": key.provider,
                    "api_key": key.api_key,
                    "environment": key.environment,
                    "is_active": key.is_active,
                    "created_at": if let Some(ts) = &key.created_at {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "assets": data.assets.iter().map(|asset| {
                serde_json::json!({
                    "id": asset.id,
                    "symbol": asset.symbol,
                    "name": asset.name,
                    "asset_class": asset.asset_class,
                    "exchange": asset.exchange,
                    "is_tradable": asset.is_tradable,
                    "last_price": asset.last_price,
                    "created_at": if let Some(ts) = &asset.created_at {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "trades": data.trades.iter().map(|trade| {
                serde_json::json!({
                    "id": trade.id,
                    "asset_symbol": trade.asset_symbol,
                    "side": trade.side,
                    "quantity": trade.quantity,
                    "price": trade.price,
                    "total_value": trade.total_value,
                    "trade_time": if let Some(ts) = &trade.trade_time {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "ollama_receipts": data.ollama_receipts.iter().map(|receipt| {
                serde_json::json!({
                    "id": receipt.id,
                    "model": receipt.model,
                    "prompt": receipt.prompt,
                    "response": receipt.response,
                    "success": receipt.success,
                    "created_at": if let Some(ts) = &receipt.created_at {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "market_regime_analyses": data.market_regime_analyses.iter().map(|analysis| {
                serde_json::json!({
                    "id": analysis.id,
                    "market_regime": analysis.market_regime,
                    "confidence_level": analysis.confidence_level,
                    "analysis_time": if let Some(ts) = &analysis.analysis_time {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "strategy_recommendations": data.strategy_recommendations.iter().map(|rec| {
                serde_json::json!({
                    "id": rec.id,
                    "asset_symbol": rec.asset_symbol,
                    "action": rec.action,
                    "target_price": rec.target_price,
                    "confidence_score": rec.confidence_score,
                    "created_at": if let Some(ts) = &rec.created_at {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "execution_signals": data.execution_signals.iter().map(|signal| {
                serde_json::json!({
                    "id": signal.id,
                    "signal_type": signal.signal_type,
                    "asset_symbol": signal.asset_symbol,
                    "strength": signal.strength,
                    "is_triggered": signal.is_triggered,
                    "created_at": if let Some(ts) = &signal.created_at {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "portfolio_snapshots": data.portfolio_snapshots.iter().map(|snapshot| {
                serde_json::json!({
                    "id": snapshot.id,
                    "total_value": snapshot.total_value,
                    "cash_balance": snapshot.cash_balance,
                    "portfolio_value": snapshot.portfolio_value,
                    "snapshot_time": if let Some(ts) = &snapshot.snapshot_time {
                        DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string()
                    } else {
                        "N/A".to_string()
                    }
                })
            }).collect::<Vec<_>>(),
            "version": data.version,
            "last_updated": if let Some(ts) = &data.last_updated {
                DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string()
            } else {
                "N/A".to_string()
            }
        });
        
        // Write to JSON file
        std::fs::write(json_file_path, serde_json::to_string_pretty(&json_data)?)?;
        println!("✅ Protobuf data exported to JSON: {}", json_file_path);
        Ok(())
    }

    // Append new API key
    pub fn append_api_key(&self, provider: &str, api_key: &str, environment: &str) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.api_keys.len() as u32 + 1;
        let new_api_key = ApiKey {
            id: new_id,
            provider: provider.to_string(),
            api_key: api_key.to_string(),
            environment: environment.to_string(),
            is_active: true,
            created_at: Some(timestamp.clone()),
        };

        data.api_keys.push(new_api_key);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new API key (ID: {})", new_id);
        Ok(())
    }

    // Append new asset
    pub fn append_asset(&self, symbol: &str, name: &str, asset_class: &str, exchange: &str, last_price: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.assets.len() as u32 + 1;
        let new_asset = Asset {
            id: new_id,
            symbol: symbol.to_string(),
            name: name.to_string(),
            asset_class: asset_class.to_string(),
            exchange: exchange.to_string(),
            is_tradable: true,
            last_price,
            created_at: Some(timestamp.clone()),
        };

        data.assets.push(new_asset);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new asset: {} (ID: {})", symbol, new_id);
        Ok(())
    }

    // Append new trade
    pub fn append_trade(&self, asset_symbol: &str, side: &str, quantity: f64, price: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.trades.len() as u32 + 1;
        let total_value = quantity * price;
        let new_trade = Trade {
            id: new_id,
            asset_symbol: asset_symbol.to_string(),
            side: side.to_string(),
            quantity,
            price,
            total_value,
            trade_time: Some(timestamp.clone()),
        };

        data.trades.push(new_trade);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new trade: {} {} {} @ ${:.2} (ID: {})", 
            side.to_uppercase(), quantity, asset_symbol, price, new_id);
        Ok(())
    }

    // Append new Ollama receipt
    pub fn append_ollama_receipt(&self, model: &str, prompt: &str, response: &str, success: bool) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.ollama_receipts.len() as u32 + 1;
        let new_receipt = OllamaReceipt {
            id: new_id,
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            success,
            created_at: Some(timestamp.clone()),
        };

        data.ollama_receipts.push(new_receipt);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new Ollama receipt (ID: {}) - Success: {}", new_id, success);
        Ok(())
    }

    // Append new market regime analysis
    pub fn append_market_regime(&self, market_regime: &str, confidence_level: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.market_regime_analyses.len() as u32 + 1;
        let new_analysis = MarketRegimeAnalysis {
            id: new_id,
            market_regime: market_regime.to_string(),
            confidence_level,
            analysis_time: Some(timestamp.clone()),
        };

        data.market_regime_analyses.push(new_analysis);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new market regime: {} (ID: {}) - Confidence: {:.1}%", 
            market_regime, new_id, confidence_level * 100.0);
        Ok(())
    }

    // Append new strategy recommendation
    pub fn append_strategy_recommendation(&self, asset_symbol: &str, action: &str, target_price: f64, confidence_score: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.strategy_recommendations.len() as u32 + 1;
        let new_recommendation = StrategyRecommendation {
            id: new_id,
            asset_symbol: asset_symbol.to_string(),
            action: action.to_string(),
            target_price,
            confidence_score,
            created_at: Some(timestamp.clone()),
        };

        data.strategy_recommendations.push(new_recommendation);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new strategy recommendation: {} {} @ ${:.2} (ID: {})", 
            action.to_uppercase(), asset_symbol, target_price, new_id);
        Ok(())
    }

    // Append new execution signal
    pub fn append_execution_signal(&self, signal_type: &str, asset_symbol: &str, strength: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.execution_signals.len() as u32 + 1;
        let new_signal = ExecutionSignal {
            id: new_id,
            signal_type: signal_type.to_string(),
            asset_symbol: asset_symbol.to_string(),
            strength,
            is_triggered: false,
            created_at: Some(timestamp.clone()),
        };

        data.execution_signals.push(new_signal);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new execution signal: {} {} (ID: {}) - Strength: {:.1}%", 
            signal_type.to_uppercase(), asset_symbol, new_id, strength * 100.0);
        Ok(())
    }

    // Append new portfolio snapshot
    pub fn append_portfolio_snapshot(&self, total_value: f64, cash_balance: f64, portfolio_value: f64) -> Result<()> {
        let mut data = self.load_trading_data()?;
        let now = Utc::now();
        let timestamp = prost_types::Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        };

        let new_id = data.portfolio_snapshots.len() as u32 + 1;
        let new_snapshot = PortfolioSnapshot {
            id: new_id,
            total_value,
            cash_balance,
            portfolio_value,
            snapshot_time: Some(timestamp.clone()),
        };

        data.portfolio_snapshots.push(new_snapshot);
        data.last_updated = Some(timestamp);
        self.save_trading_data(&data)?;
        println!("✅ Appended new portfolio snapshot (ID: {}) - Total: ${:.2}", new_id, total_value);
        Ok(())
    }

    // Get statistics about stored data
    pub fn get_statistics(&self) -> Result<()> {
        let data = self.load_trading_data()?;
        
        println!("\n📊 PROTOBUF STORAGE STATISTICS");
        println!("{}", "=".repeat(50));
        println!("📁 File: {}", self.file_path);
        println!("📈 Total Records: {}", 
            data.api_keys.len() + data.assets.len() + data.trades.len() + 
            data.ollama_receipts.len() + data.market_regime_analyses.len() + 
            data.strategy_recommendations.len() + data.execution_signals.len() + 
            data.portfolio_snapshots.len());
        println!("🔑 API Keys: {}", data.api_keys.len());
        println!("📈 Assets: {}", data.assets.len());
        println!("💼 Trades: {}", data.trades.len());
        println!("🤖 Ollama Receipts: {}", data.ollama_receipts.len());
        println!("📊 Market Regime Analyses: {}", data.market_regime_analyses.len());
        println!("🎯 Strategy Recommendations: {}", data.strategy_recommendations.len());
        println!("🚦 Execution Signals: {}", data.execution_signals.len());
        println!("💰 Portfolio Snapshots: {}", data.portfolio_snapshots.len());
        println!("📋 Version: {}", data.version);
        
        if let Some(ts) = &data.last_updated {
            let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
            println!("🕒 Last Updated: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        Ok(())
    }
}
