use crate::coingecko::PriceData;
use std::future::Future;

pub trait DB: Clone + Send + Sync + 'static {
    fn insert(&self, data: PriceData) -> impl Future<Output = Result<(), String>> + Send;
    fn fetch_all(&self) -> impl Future<Output = Vec<PriceData>> + Send;
    fn delete_by_timestamp(&self, timestamp: i64) -> impl Future<Output = Result<(), String>> + Send;
}
