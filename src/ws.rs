use crate::coingecko;
use crate::db::DB;
use tokio::time::{sleep, Duration};

pub(crate) struct WS<T: DB> {
    api: coingecko::API,
    db: T,
}

impl<T: DB> WS<T> {
    pub fn new(api: coingecko::API, db: T) -> Self {
        Self { api, db }
    }

    // start fetching api data every second and inserting it to DB
    pub fn start(&self) {
        let api = self.api.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            loop {
                match api.btc_price().await {
                    Ok(price_data) => {
                        match db.insert(price_data.clone()).await {
                            Ok(_) => println!("Price inserted: ${}", price_data.price),
                            Err(e) => eprintln!("Failed to insert price data: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch BTC price: {}", e);
                    }
                }
                sleep(Duration::from_secs(1)).await;
            }
        });
    }
}
