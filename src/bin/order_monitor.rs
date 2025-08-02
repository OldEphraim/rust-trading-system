use dotenv::dotenv;
use rust_trading_system::trading::TestnetTrader;
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let api_key = env::var("TESTNET_BINANCE_VISION_API_KEY")
        .expect("TESTNET_BINANCE_VISION_API_KEY must be set in .env file");
    let secret_key = env::var("TESTNET_BINANCE_VISION_SECRET_KEY")
        .expect("TESTNET_BINANCE_VISION_SECRET_KEY must be set in .env file");

    let trader = TestnetTrader::new(api_key, secret_key);

    loop {
        println!("📊 Testnet Order Monitor");
        println!("========================");
        println!();

        // Get all open orders
        match trader.get_open_orders(None).await {
            Ok(orders) => {
                if orders.is_empty() {
                    println!("📭 No open orders found");
                } else {
                    println!("📋 Open Orders ({} total):", orders.len());
                    println!();
                    
                    for order in &orders {
                        println!("🔸 Order #{}", order.order_id);
                        println!("   Symbol: {}", order.symbol);
                        println!("   Side: {}", order.side);
                        println!("   Type: {}", order.order_type);
                        println!("   Status: {:?}", order.status);
                        println!("   Quantity: {} (Executed: {})", order.orig_qty, order.executed_qty);
                        println!("   Price: ${}", order.price);
                        println!("   Time: {}", 
                                if let Some(time) = order.transact_time.or(order.time) {
                                    chrono::DateTime::from_timestamp_millis(time as i64)
                                        .unwrap_or_default()
                                        .format("%Y-%m-%d %H:%M:%S UTC")
                                        .to_string()
                                } else {
                                    "Unknown".to_string()
                                });
                        println!();
                    }
                }

                // Interactive menu
                println!("🎛️  What would you like to do?");
                println!("1. Refresh orders");
                println!("2. Cancel an order");
                println!("3. Check account balance");
                println!("4. Get current Bitcoin price");
                println!("5. Exit");
                println!();
                print!("Enter choice (1-5): ");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                match input.trim() {
                    "1" => {
                        println!("🔄 Refreshing...");
                        println!();
                        continue; // Loop back to refresh orders
                    }
                    "2" => {
                        if !orders.is_empty() {
                            println!("Enter order ID to cancel:");
                            let mut order_id_input = String::new();
                            std::io::stdin().read_line(&mut order_id_input)?;
                            
                            if let Ok(order_id) = order_id_input.trim().parse::<u64>() {
                                // Find the symbol for this order
                                if let Some(order) = orders.iter().find(|o| o.order_id == order_id) {
                                    match trader.cancel_order(&order.symbol, order_id).await {
                                        Ok(_) => println!("✅ Order {} canceled successfully!", order_id),
                                        Err(e) => println!("❌ Failed to cancel order: {}", e),
                                    }
                                } else {
                                    println!("❌ Order ID not found in your open orders");
                                }
                            } else {
                                println!("❌ Invalid order ID");
                            }
                        } else {
                            println!("❌ No orders to cancel");
                        }
                        println!("Press Enter to continue...");
                        let mut _dummy = String::new();
                        std::io::stdin().read_line(&mut _dummy)?;
                    }
                    "3" => {
                        match trader.get_account_info().await {
                            Ok(account) => {
                                println!("💳 Main Balances:");
                                let important_assets = ["BTC", "ETH", "USDT", "BNB"];
                                for balance in &account.balances {
                                    if important_assets.contains(&balance.asset.as_str()) && balance.free > 0.0 {
                                        println!("   {} {:.8} (Free: {:.8})", balance.asset, balance.free + balance.locked, balance.free);
                                    }
                                }
                            }
                            Err(e) => println!("❌ Failed to get account info: {}", e),
                        }
                        println!("Press Enter to continue...");
                        let mut _dummy = String::new();
                        std::io::stdin().read_line(&mut _dummy)?;
                    }
                    "4" => {
                        match trader.get_current_price("BTCUSDT").await {
                            Ok(price) => println!("💰 Current BTC/USDT: ${:.2}", price),
                            Err(e) => println!("❌ Failed to get price: {}", e),
                        }
                        println!("Press Enter to continue...");
                        let mut _dummy = String::new();
                        std::io::stdin().read_line(&mut _dummy)?;
                    }
                    "5" => {
                        println!("👋 Goodbye!");
                        return Ok(());
                    }
                    _ => {
                        println!("❌ Invalid choice");
                        println!("Press Enter to continue...");
                        let mut _dummy = String::new();
                        std::io::stdin().read_line(&mut _dummy)?;
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to get orders: {}", e);
                println!("Press Enter to continue...");
                let mut _dummy = String::new();
                std::io::stdin().read_line(&mut _dummy)?;
            }
        }
    }
}