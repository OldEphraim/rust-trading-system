use rust_trading_system::trading::{TestnetTrader, OrderSide};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use serde_json::json;

#[tokio::test]
async fn test_get_account_info_integration() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create mock response
    let mock_response = json!({
        "balances": [
            {"asset": "BTC", "free": "1.00000000", "locked": "0.00000000"},
            {"asset": "USDT", "free": "10000.00000000", "locked": "0.00000000"}
        ],
        "canTrade": true,
        "canWithdraw": false,
        "canDeposit": true
    });

    // Set up mock endpoint
    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .and(header("X-MBX-APIKEY", "test_api_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&mock_server)
        .await;

    // Create trader with mock server URL
    let trader = TestnetTrader::new(
        "test_api_key".to_string(),
        "test_secret_key".to_string(),
    ).with_base_url(mock_server.uri());

    // Test the API call
    let account_info = trader.get_account_info().await.unwrap();

    assert!(account_info.can_trade);
    assert!(!account_info.can_withdraw);
    assert!(account_info.can_deposit);
    assert_eq!(account_info.balances.len(), 2);
    
    let btc_balance = account_info.balances.iter()
        .find(|b| b.asset == "BTC")
        .unwrap();
    assert_eq!(btc_balance.free, 1.0);
}

#[tokio::test]
async fn test_get_current_price_integration() {
    let mock_server = MockServer::start().await;

    let mock_response = json!({
        "symbol": "BTCUSDT",
        "price": "50000.00"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/price"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&mock_server)
        .await;

    let trader = TestnetTrader::new(
        "test_api_key".to_string(),
        "test_secret_key".to_string(),
    ).with_base_url(mock_server.uri());

    let price = trader.get_current_price("BTCUSDT").await.unwrap();
    assert_eq!(price, 50000.0);
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    // Mock a 400 error response
    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "code": -1022,
            "msg": "Signature for this request is not valid."
        })))
        .mount(&mock_server)
        .await;

    let trader = TestnetTrader::new(
        "test_api_key".to_string(),
        "test_secret_key".to_string(),
    ).with_base_url(mock_server.uri());

    let result = trader.get_account_info().await;
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Signature for this request is not valid"));
}