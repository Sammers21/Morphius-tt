use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub(crate) struct API {
    key: String,
    client: reqwest::Client,
}

impl API {
    pub fn new(key: String) -> Self {
        Self { 
            key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn btc_price(&self) -> Result<PriceData, Box<dyn std::error::Error + Send + Sync>> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let response = self.client
            .get(url)
            .header("x-cg-demo-api-key", &self.key)
            .send()
            .await?;
        let api_response: CoinGeckoResponse = response.json().await?;
        let price = api_response.bitcoin.usd;
        Ok(PriceData { price, timestamp: timestamp as i64 })
    }
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct PriceData {
    pub(crate) price: f64,
    pub(crate) timestamp: i64,
}

#[derive(Deserialize)]
struct CoinGeckoResponse {
    bitcoin: BitcoinPrice,
}

#[derive(Deserialize)]
struct BitcoinPrice {
    usd: f64,
}
