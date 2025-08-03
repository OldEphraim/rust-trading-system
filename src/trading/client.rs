use super::types::*;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{error, info};

type HmacSha256 = Hmac<Sha256>;

/// TestnetTrader is the main struct for interacting with Binance's testnet API
/// It handles authentication, API calls, and order management with fake money
pub struct TestnetTrader {
    api_key: String,      // Your testnet API key
    secret_key: String,   // Your testnet secret key (for signing requests)
    client: Client,       // HTTP client for making requests
    base_url: String,     // Base URL for the API (can be changed for testing)
}

impl TestnetTrader {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
            client: Client::new(),
            base_url: "https://testnet.binance.vision".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo, Box<dyn std::error::Error>> {
        let endpoint = "/api/v3/account";
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        // Build parameters for the API call
        let mut params = HashMap::new();
        params.insert("timestamp".to_string(), timestamp.to_string());
        
        // Create query string and sign it
        let query_string = self.build_query_string(&params);
        let signature = self.sign(&query_string);
        
        let url = format!("{}{}?{}&signature={}", 
                         self.base_url, endpoint, query_string, signature);
        
        let response = self.client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("API Error Response: {}", error_text);
            return Err(format!("API Error: {}", error_text).into());
        }

        let response_text = response.text().await?;
        info!("Account API Response: {}", response_text);
        
        let account_info: AccountInfo = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse account info: {}. Response was: {}", e, response_text))?;
        
        Ok(account_info)
    }

    pub async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
    ) -> Result<OrderResponse, Box<dyn std::error::Error>> {
        let endpoint = "/api/v3/order";
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        });
        params.insert("type".to_string(), "MARKET".to_string());
        params.insert("quantity".to_string(), format!("{:.8}", quantity));
        params.insert("timestamp".to_string(), timestamp.to_string());
        
        let query_string = self.build_query_string(&params);
        let signature = self.sign(&query_string);
        
        let url = format!("{}{}", self.base_url, endpoint);
        let body = format!("{}&signature={}", query_string, signature);
        
        info!("Placing {} order for {} {} on testnet", 
              match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
              quantity, symbol);
        
        let response = self.client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Order placement failed: {}", error_text);
            return Err(format!("Order Error: {}", error_text).into());
        }

        let order_response: OrderResponse = response.json().await?;
        info!("Order placed successfully: ID {}", order_response.order_id);
        Ok(order_response)
    }

    pub async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> Result<OrderResponse, Box<dyn std::error::Error>> {
        let endpoint = "/api/v3/order";
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        });
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("timeInForce".to_string(), "GTC".to_string()); // Good Till Canceled
        params.insert("quantity".to_string(), format!("{:.8}", quantity));
        params.insert("price".to_string(), format!("{:.2}", price));
        params.insert("timestamp".to_string(), timestamp.to_string());
        
        let query_string = self.build_query_string(&params);
        let signature = self.sign(&query_string);
        
        let url = format!("{}{}", self.base_url, endpoint);
        let body = format!("{}&signature={}", query_string, signature);
        
        info!("Placing {} limit order for {} {} at ${} on testnet", 
              match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
              quantity, symbol, price);
        
        let response = self.client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Limit order placement failed: {}", error_text);
            return Err(format!("Order Error: {}", error_text).into());
        }

        let order_response: OrderResponse = response.json().await?;
        info!("Limit order placed successfully: ID {}", order_response.order_id);
        Ok(order_response)
    }

    pub async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<OrderResponse>, Box<dyn std::error::Error>> {
        let endpoint = "/api/v3/openOrders";
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        params.insert("timestamp".to_string(), timestamp.to_string());
        
        let query_string = self.build_query_string(&params);
        let signature = self.sign(&query_string);
        
        let url = format!("{}{}?{}&signature={}", 
                         self.base_url, endpoint, query_string, signature);
        
        let response = self.client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Get orders API Error: {}", error_text);
            return Err(format!("API Error: {}", error_text).into());
        }

        let response_text = response.text().await?;
        info!("Open orders API response: {}", response_text);
        
        let orders: Vec<OrderResponse> = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse open orders: {}. Response was: {}", e, response_text))?;
        
        Ok(orders)
    }

    pub async fn cancel_order(&self, symbol: &str, order_id: u64) -> Result<OrderResponse, Box<dyn std::error::Error>> {
        let endpoint = "/api/v3/order";
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("orderId".to_string(), order_id.to_string());
        params.insert("timestamp".to_string(), timestamp.to_string());
        
        let query_string = self.build_query_string(&params);
        let signature = self.sign(&query_string);
        
        let url = format!("{}{}", self.base_url, endpoint);
        let body = format!("{}&signature={}", query_string, signature);
        
        info!("Canceling order {} for {} on testnet", order_id, symbol);
        
        let response = self.client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Cancel order failed: {}", error_text);
            return Err(format!("Cancel Error: {}", error_text).into());
        }

        let order_response: OrderResponse = response.json().await?;
        info!("Order {} canceled successfully", order_id);
        Ok(order_response)
    }

    pub async fn get_current_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v3/ticker/price?symbol={}", self.base_url, symbol);
        
        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;
        
        if let Some(price_str) = data["price"].as_str() {
            Ok(price_str.parse()?)
        } else {
            Err("Could not parse price".into())
        }
    }

    pub fn build_query_string(&self, params: &std::collections::HashMap<String, String>) -> String {
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|&(k, _)| k);
        
        sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn sign(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}