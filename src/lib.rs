pub mod market_data;  // Real-time price data and WebSocket connections
pub mod portfolio;    // Portfolio management (currently just stubs)
pub mod orders;       // Order management (currently just stubs)
pub mod strategies;   // Trading strategies (currently just stubs)
pub mod trading;      // Main trading client and types

// Unit tests - these run when you do `cargo test --lib`
// The #[cfg(test)] attribute means this code only compiles during testing
#[cfg(test)]
mod tests {
    // Import the types we need for testing
    use crate::trading::{Balance, OrderSide, TestnetTrader};
    use std::collections::HashMap;

    /// Tests for trading data types (Balance, OrderSide, etc.)
    /// These verify that our structs serialize/deserialize correctly
    mod trading_types_tests {
        use super::*;

        /// Test that Balance struct can be created from JSON (deserialization)
        /// This is crucial because Binance API returns JSON with string numbers
        #[test]
        fn test_balance_deserialization() {
            // This is what Binance API returns for a balance
            let json = r#"{"asset":"BTC","free":"1.50000000","locked":"0.25000000"}"#;
            
            // Our custom deserializer should convert string numbers to f64
            let balance: Balance = serde_json::from_str(json).unwrap();
            
            assert_eq!(balance.asset, "BTC");
            assert_eq!(balance.free, 1.5);      // "1.50000000" -> 1.5
            assert_eq!(balance.locked, 0.25);   // "0.25000000" -> 0.25
        }

        /// Test that OrderSide enum serializes to the correct string format
        /// Binance expects "BUY" and "SELL", not "Buy" and "Sell"
        #[test]
        fn test_order_side_serialization() {
            assert_eq!(serde_json::to_string(&OrderSide::Buy).unwrap(), r#""BUY""#);
            assert_eq!(serde_json::to_string(&OrderSide::Sell).unwrap(), r#""SELL""#);
        }

        /// Test that all OrderStatus variants can be created
        /// This ensures we haven't broken the enum definition
        #[test]
        fn test_order_status_variants() {
            use crate::trading::OrderStatus;
            
            // Test that we can create all variants without errors
            let _new = OrderStatus::New;
            let _filled = OrderStatus::Filled;
            let _canceled = OrderStatus::Canceled;
            let _rejected = OrderStatus::Rejected;
            let _expired = OrderStatus::Expired;
            let _partially_filled = OrderStatus::PartiallyFilled;
        }
    }

    /// Tests for cryptographic signature functionality
    /// These are critical for API security - wrong signatures = rejected requests
    mod signature_tests {
        use super::*;

        /// Helper function to create a trader for testing
        /// Uses dummy credentials since we're only testing crypto functions
        fn create_test_trader() -> TestnetTrader {
            TestnetTrader::new(
                "test_api_key".to_string(),
                "test_secret_key".to_string(),
            )
        }

        /// Test that query string building produces sorted, deterministic output
        /// Binance requires parameters in alphabetical order for valid signatures
        #[test]
        fn test_query_string_building() {
            let trader = create_test_trader();
            let mut params = HashMap::new();
            params.insert("symbol".to_string(), "BTCUSDT".to_string());
            params.insert("side".to_string(), "BUY".to_string());
            params.insert("timestamp".to_string(), "1640995200000".to_string());

            let query_string = trader.build_query_string(&params);
            
            // Should be sorted alphabetically by key: side, symbol, timestamp
            assert_eq!(query_string, "side=BUY&symbol=BTCUSDT&timestamp=1640995200000");
        }

        /// Test that HMAC signature generation is consistent
        /// Same input should always produce same signature
        #[test]
        fn test_signature_consistency() {
            let trader = create_test_trader();
            let query1 = "symbol=BTCUSDT&side=BUY&timestamp=1640995200000";
            let query2 = "symbol=BTCUSDT&side=BUY&timestamp=1640995200000";

            let sig1 = trader.sign(query1);
            let sig2 = trader.sign(query2);

            assert_eq!(sig1, sig2, "Same query should produce same signature");
            assert_eq!(sig1.len(), 64, "HMAC-SHA256 should produce 64-character hex string");
        }

        /// Test that different queries produce different signatures
        /// This verifies the signature actually depends on input content
        #[test]
        fn test_signature_different_for_different_queries() {
            let trader = create_test_trader();
            let query1 = "symbol=BTCUSDT&side=BUY&timestamp=1640995200000";
            let query2 = "symbol=BTCUSDT&side=SELL&timestamp=1640995200000";

            let sig1 = trader.sign(query1);
            let sig2 = trader.sign(query2);

            assert_ne!(sig1, sig2, "Different queries should produce different signatures");
        }

        /// Test that signatures are valid hexadecimal strings
        /// HMAC-SHA256 should produce 32 bytes = 64 hex characters
        #[test]
        fn test_signature_is_hex() {
            let trader = create_test_trader();
            let query = "test=123&time=456";
            let signature = trader.sign(query);
            
            // Should be valid hex (only 0-9, a-f characters)
            assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
            assert_eq!(signature.len(), 64);
        }
    }

    /// Tests for market data functionality
    /// These verify our real-time data structures work correctly
    mod market_data_tests {
        use crate::market_data::{Ticker, MarketDataEvent, TradeSide};

        /// Test basic Ticker struct creation and field access
        #[test]
        fn test_ticker_creation() {
            let ticker = Ticker {
                symbol: "BTCUSDT".to_string(),
                price: 50000.0,
                volume: 1000.0,
                timestamp: 1640995200000,
            };

            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.price, 50000.0);
            assert_eq!(ticker.volume, 1000.0);
        }

        /// Test that MarketDataEvent enum pattern matching works
        /// This verifies we can extract data from event streams
        #[test]
        fn test_market_data_event_matching() {
            let ticker = Ticker {
                symbol: "BTCUSDT".to_string(),
                price: 50000.0,
                volume: 1000.0,
                timestamp: 1640995200000,
            };

            let event = MarketDataEvent::Ticker(ticker.clone());

            // Test pattern matching to extract ticker data
            match event {
                MarketDataEvent::Ticker(t) => {
                    assert_eq!(t.symbol, "BTCUSDT");
                    assert_eq!(t.price, 50000.0);
                }
                _ => panic!("Expected Ticker event"),
            }
        }

        /// Test TradeSide enum debug formatting
        /// Ensures debug output is readable for logging
        #[test]
        fn test_trade_side_enum() {
            assert_eq!(format!("{:?}", TradeSide::Buy), "Buy");
            assert_eq!(format!("{:?}", TradeSide::Sell), "Sell");
        }
    }
}