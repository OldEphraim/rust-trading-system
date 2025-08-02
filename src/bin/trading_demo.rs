use dotenv::dotenv;
use rust_trading_system::trading::{TestnetTrader, OrderSide};
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get API credentials from environment
    let api_key = env::var("TESTNET_BINANCE_VISION_API_KEY")
        .expect("TESTNET_BINANCE_VISION_API_KEY must be set in .env file");
    let secret_key = env::var("TESTNET_BINANCE_VISION_SECRET_KEY")
        .expect("TESTNET_BINANCE_VISION_SECRET_KEY must be set in .env file");

    // Create trader instance
    let trader = TestnetTrader::new(api_key, secret_key);

    println!("🎮 Welcome to Testnet Trading Demo!");
    println!("💰 All trades use FAKE MONEY - completely risk-free!");
    println!();

    // Check account info and balances
    println!("📊 Getting account information...");
    match trader.get_account_info().await {
        Ok(account) => {
            println!("✅ Account Status: {}", if account.can_trade { "Trading Enabled" } else { "Trading Disabled" });
            println!();
            println!("💳 Your Fake Balances:");
            
            // Show main balances (BTC, ETH, USDT, etc.)
            let important_assets = ["BTC", "ETH", "USDT", "BNB"];
            for balance in &account.balances {
                if important_assets.contains(&balance.asset.as_str()) && balance.free > 0.0 {
                    println!("   {} {:.8} {} (Free: {:.8}, Locked: {:.8})", 
                            if balance.asset == "USDT" { "💵" } else { "🪙" },
                            balance.free + balance.locked,
                            balance.asset,
                            balance.free,
                            balance.locked);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get account info: {}", e);
            return Ok(());
        }
    }

    println!();

    // Get current Bitcoin price
    println!("📈 Getting current Bitcoin price...");
    match trader.get_current_price("BTCUSDT").await {
        Ok(price) => {
            println!("💰 Current BTC/USDT Price: ${:.2}", price);
            
            // Demo: Place a small limit buy order (will likely not fill immediately)
            println!();
            println!("🛒 Demo: Placing a limit BUY order for 0.001 BTC at 10% below market price...");
            let buy_price = price * 0.9; // 10% below market
            let quantity = 0.001; // Small amount for testing
            
            match trader.place_limit_order("BTCUSDT", OrderSide::Buy, quantity, buy_price).await {
                Ok(order) => {
                    println!("✅ Limit order placed successfully!");
                    println!("   Order ID: {}", order.order_id);
                    println!("   Status: {:?}", order.status);
                    println!("   Quantity: {} BTC", order.orig_qty);
                    println!("   Price: ${}", order.price);
                    println!("   Side: {}", order.side);
                    println!();
                    println!("💡 This order will only fill if Bitcoin drops to ${:.2}", buy_price);
                    println!("   (Since it's 10% below market, it probably won't fill immediately)");
                }
                Err(e) => {
                    println!("❌ Failed to place order: {}", e);
                    println!("   This might be due to insufficient balance or API issues.");
                }
            }

            // Alternative demo: Show what a market order would look like (but don't execute)
            println!();
            println!("📝 What a MARKET order would look like:");
            println!("   - Market BUY 0.0001 BTC would execute immediately at ~${:.2}", price);
            println!("   - Market SELL would execute immediately at current bid price");
            println!("   (Not executing to preserve your fake balance)");
        }
        Err(e) => {
            println!("❌ Failed to get current price: {}", e);
        }
    }

    println!();
    println!("🎯 Next Steps:");
    println!("   1. Check your orders on https://testnet.binance.vision/");
    println!("   2. Try different trading strategies");
    println!("   3. Add LLM integration for AI trading decisions!");
    println!();
    println!("🚀 Ready to build an AI trading bot with this foundation!");

    Ok(())
}