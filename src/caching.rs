use crate::coingecko::PriceData;
use crate::db::DB;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;


/// `CachingDB` is a proxy caching implementation of `DB` using Btree under the hood 
#[derive(Clone)]
pub struct CachingDB<T: DB> {
    src: T,
    cache: Arc<RwLock<Option<BTreeMap<i64, PriceData>>>>,
}

impl<T: DB> CachingDB<T> {
    pub fn new(src: T) -> Self {
        Self {
            src,
            cache: Arc::new(RwLock::new(None)),
        }
    }
}

impl<T: DB> DB for CachingDB<T> {
    fn insert(&self, data: PriceData) -> impl Future<Output = Result<(), String>> + Send {
        let src = self.src.clone();
        let cache = self.cache.clone();
        async move {
            let result = src.insert(data.clone()).await;
            if result.is_ok() {
                let mut cache_guard = cache.write().await;
                if let Some(ref mut cached_data) = *cache_guard {
                    cached_data.insert(data.timestamp, data);
                }
            }
            result
        }
    }

    fn fetch_all(&self) -> impl Future<Output = Vec<PriceData>> + Send {
        let cache = self.cache.clone();
        let src = self.src.clone();
        async move {
            let cache_read = cache.read().await;
            if let Some(ref cached_data) = *cache_read {
                return cached_data.values().cloned().collect();
            }
            drop(cache_read);
            let data = src.fetch_all().await;
            let mut cache_write = cache.write().await;
            let cache_map: BTreeMap<i64, PriceData> = data.iter().map(|d| (d.timestamp, d.clone())).collect();
            *cache_write = Some(cache_map);
            data
        }
    }

    fn delete_by_timestamp(&self, timestamp: i64) -> impl Future<Output = Result<(), String>> + Send {
        let src = self.src.clone();
        let cache = self.cache.clone();
        async move {
            let result = src.delete_by_timestamp(timestamp).await;
            if result.is_ok() {
                let mut cache_guard = cache.write().await;
                if let Some(ref mut cached_data) = *cache_guard {
                    cached_data.remove(&timestamp);
                }
            }
            result
        }
    }
}
