use std::env;
use rust_trading_system::trading::{TestnetTrader, OrderSide};

// These tests require actual API keys and will hit the real testnet
// Only run if INTEGRATION_TESTS environment variable is set
fn should_run_integration_tests() -> bool {
    env::var("INTEGRATION_TESTS").is_ok()
}

#[tokio::test]
async fn test_real_testnet_connection() {
    if !should_run_integration_tests() {
        return; // Skip if not explicitly enabled
    }

    let api_key = env::var("TESTNET_BINANCE_VISION_API_KEY")
        .expect("TESTNET_BINANCE_VISION_API_KEY required for integration tests");
    let secret_key = env::var("TESTNET_BINANCE_VISION_SECRET_KEY")
        .expect("TESTNET_BINANCE_VISION_SECRET_KEY required for integration tests");

    let trader = TestnetTrader::new(api_key, secret_key);

    // Test getting account info
    let account_info = trader.get_account_info().await.unwrap();
    assert!(account_info.can_trade);

    // Test getting current price
    let price = trader.get_current_price("BTCUSDT").await.unwrap();
    assert!(price > 0.0);

    // Test getting open orders (should work even if empty)
    let orders = trader.get_open_orders(None).await.unwrap();
    println!("Current open orders: {}", orders.len());
}

#[tokio::test]
async fn test_order_lifecycle() {
    if !should_run_integration_tests() {
        return;
    }

    let api_key = env::var("TESTNET_BINANCE_VISION_API_KEY").unwrap();
    let secret_key = env::var("TESTNET_BINANCE_VISION_SECRET_KEY").unwrap();
    let trader = TestnetTrader::new(api_key, secret_key);

    // Get current price and place order 50% below market (won't fill)
    let current_price = trader.get_current_price("BTCUSDT").await.unwrap();
    let order_price = current_price * 0.5;

    // Place order
    let order = trader.place_limit_order("BTCUSDT", OrderSide::Buy, 0.001, order_price)
        .await
        .unwrap();

    println!("Placed order: {:?}", order.order_id);

    // Verify order appears in open orders
    let open_orders = trader.get_open_orders(Some("BTCUSDT")).await.unwrap();
    let our_order = open_orders.iter().find(|o| o.order_id == order.order_id);
    assert!(our_order.is_some(), "Order should appear in open orders");

    // Cancel the order
    let cancelled = trader.cancel_order("BTCUSDT", order.order_id).await.unwrap();
    assert_eq!(cancelled.order_id, order.order_id);

    println!("Order lifecycle test completed successfully");
}