use rust_trading_system::market_data::MarketDataStream;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create market data stream for BTC/USDT
    let mut stream = MarketDataStream::new(vec!["BTCUSDT".to_string()]).await?;

    println!("🚀 Starting Rust Trading System");
    println!("📊 Listening for BTC/USDT price updates...");
    println!("Press Ctrl+C to stop\n");

    // Process market data events
    while let Some(event) = stream.next_event().await {
        match event {
            rust_trading_system::market_data::MarketDataEvent::Ticker(ticker) => {
                println!(
                    "💰 {} | Price: ${:.2} | Volume: {:.2} | Time: {}",
                    ticker.symbol, ticker.price, ticker.volume, ticker.timestamp
                );
            }
            rust_trading_system::market_data::MarketDataEvent::Error(err) => {
                eprintln!("❌ Error: {}", err);
            }
            _ => {}
        }
    }

    Ok(())
}