use proptest::prelude::*;
use rust_trading_system::trading::{Balance, TestnetTrader};
use std::collections::HashMap;

proptest! {
    #[test]
    fn test_balance_parsing_never_panics(
        asset in "[A-Z]{3,5}",
        free in 0.0..1000000.0f64,
        locked in 0.0..1000000.0f64
    ) {
        let json = format!(
            r#"{{"asset":"{}","free":"{:.8}","locked":"{:.8}"}}"#,
            asset, free, locked
        );
        
        // This should never panic, regardless of input
        let result: Result<Balance, _> = serde_json::from_str(&json);
        if let Ok(balance) = result {
            prop_assert_eq!(balance.asset, asset);
            prop_assert!((balance.free - free).abs() < 0.00000001);
            prop_assert!((balance.locked - locked).abs() < 0.00000001);
        }
    }

    #[test]
    fn test_query_string_building_is_deterministic(
        symbol in "[A-Z]{6,8}",
        side in "(BUY|SELL)",
        quantity in 0.001..100.0f64,
        timestamp in 1640000000000u64..1700000000000u64
    ) {
        let trader = TestnetTrader::new(
            "test_key".to_string(),
            "test_secret".to_string(),
        );

        let mut params1 = HashMap::new();
        params1.insert("symbol".to_string(), symbol.clone());
        params1.insert("side".to_string(), side.clone());
        params1.insert("quantity".to_string(), format!("{:.8}", quantity));
        params1.insert("timestamp".to_string(), timestamp.to_string());

        let mut params2 = HashMap::new();
        params2.insert("timestamp".to_string(), timestamp.to_string());
        params2.insert("quantity".to_string(), format!("{:.8}", quantity));
        params2.insert("side".to_string(), side.clone());
        params2.insert("symbol".to_string(), symbol.clone());

        let query1 = trader.build_query_string(&params1);
        let query2 = trader.build_query_string(&params2);

        // Order of insertion shouldn't matter - output should be the same
        prop_assert_eq!(query1, query2);
        
        // Query string should always be sorted
        let parts: Vec<&str> = query1.split('&').collect();
        let mut sorted_parts = parts.clone();
        sorted_parts.sort();
        prop_assert_eq!(parts, sorted_parts);
    }
}