use super::types::*;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

pub struct BinanceClient {
    symbols: Vec<String>,
    event_sender: mpsc::UnboundedSender<MarketDataEvent>,
}

impl BinanceClient {
    pub fn new(
        symbols: Vec<String>,
        event_sender: mpsc::UnboundedSender<MarketDataEvent>,
    ) -> Self {
        Self {
            symbols,
            event_sender,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.build_stream_url();
        info!("Connecting to Binance testnet: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text) {
                        error!("Error handling message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    let _ = self.event_sender.send(MarketDataEvent::Error(e.to_string()));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn build_stream_url(&self) -> String {
        // Use Binance testnet WebSocket - free fake money trading!
        if self.symbols.len() == 1 {
            let symbol = self.symbols[0].to_lowercase();
            format!("wss://stream.testnet.binance.vision/ws/{}@ticker", symbol)
        } else {
            let streams: Vec<String> = self
                .symbols
                .iter()
                .map(|s| format!("{}@ticker", s.to_lowercase()))
                .collect();
            
            format!(
                "wss://stream.testnet.binance.vision/stream?streams={}",
                streams.join("/")
            )
        }
    }

    fn handle_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data: Value = serde_json::from_str(text)?;
        
        // Handle different message formats
        if let Some(stream) = data.get("stream").and_then(|s| s.as_str()) {
            // Combined stream format
            if stream.contains("@ticker") {
                let ticker_data = &data["data"];
                self.parse_ticker(ticker_data)?;
            }
        } else if data.get("e").and_then(|e| e.as_str()) == Some("24hrTicker") {
            // Single stream format
            self.parse_ticker(&data)?;
        }

        Ok(())
    }

    fn parse_ticker(&self, ticker_data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let ticker = Ticker {
            symbol: ticker_data["s"].as_str().unwrap_or_default().to_string(),
            price: ticker_data["c"].as_str().unwrap_or("0").parse()?,
            volume: ticker_data["v"].as_str().unwrap_or("0").parse()?,
            timestamp: ticker_data["E"].as_u64().unwrap_or(0),
        };
        
        let _ = self.event_sender.send(MarketDataEvent::Ticker(ticker));
        Ok(())
    }
}

use futures_util::StreamExt;