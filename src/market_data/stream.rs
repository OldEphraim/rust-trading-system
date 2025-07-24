use super::types::*;
use super::binance::BinanceClient;
use tokio::sync::mpsc;
use tracing::info;

pub struct MarketDataStream {
    event_receiver: mpsc::UnboundedReceiver<MarketDataEvent>,
    _client_handle: tokio::task::JoinHandle<()>,
}

impl MarketDataStream {
    pub async fn new(symbols: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let client = BinanceClient::new(symbols.clone(), event_sender);
        
        let client_handle = tokio::spawn(async move {
            if let Err(e) = client.start().await {
                tracing::error!("Binance client error: {}", e);
            }
        });

        info!("Started market data stream for symbols: {:?}", symbols);

        Ok(Self {
            event_receiver,
            _client_handle: client_handle,
        })
    }

    pub async fn next_event(&mut self) -> Option<MarketDataEvent> {
        self.event_receiver.recv().await
    }
}