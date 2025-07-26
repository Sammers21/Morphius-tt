use crate::coingecko::PriceData;
use crate::db::DB;
use chrono::{DateTime, Utc, Duration};
use std::collections::BTreeMap;
use std::future::Future;

/// `CleanupDB` is a proxy that performs data cleanup on every insert
/// - Last minute: per second data
/// - Last hour: per minute data (except last minute)
/// - Last day: per hour data (except last hour)
/// - Older: per day data
#[derive(Clone)]
pub struct CleanupDB<T: DB> {
    src: T,
}

impl<T: DB> CleanupDB<T> {
    pub fn new(src: T) -> Self {
        Self { src }
    }
    fn to_datetime(timestamp: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
    }
    fn aggregate_by_interval(
        data: Vec<PriceData>,
        format_key: fn(&DateTime<Utc>) -> String,
    ) -> Vec<PriceData> {
        let mut interval_map: BTreeMap<String, PriceData> = BTreeMap::new();
        for item in data {
            let data_time = Self::to_datetime(item.timestamp);
            let key = format_key(&data_time);
            if !interval_map.contains_key(&key) || data_time > Self::to_datetime(interval_map[&key].timestamp) {
                interval_map.insert(key, item);
            }
        }
        interval_map.into_values().collect()
    }
    fn partition_by_age(data: Vec<PriceData>, cutoff: DateTime<Utc>) -> (Vec<PriceData>, Vec<PriceData>) {
        data.into_iter().partition(|item| Self::to_datetime(item.timestamp) >= cutoff)
    }
    async fn cleanup_old_data(&self) -> Result<(), String> {
        let now = Utc::now();
        let all_data = self.src.fetch_all().await;
        let mut to_keep = Vec::new();
        let (recent_data, rest) = Self::partition_by_age(all_data, now - Duration::minutes(1));
        to_keep.extend(recent_data);
        let (hour_data, rest) = Self::partition_by_age(rest, now - Duration::hours(1));
        to_keep.extend(Self::aggregate_by_interval(hour_data, |dt| dt.format("%Y-%m-%d %H:%M").to_string()));
        let (day_data, older_data) = Self::partition_by_age(rest, now - Duration::days(1));
        to_keep.extend(Self::aggregate_by_interval(day_data, |dt| dt.format("%Y-%m-%d %H").to_string()));
        to_keep.extend(Self::aggregate_by_interval(older_data, |dt| dt.format("%Y-%m-%d").to_string()));
        let current_data = self.src.fetch_all().await;
        for existing in current_data {
            if !to_keep.iter().any(|keep| keep.timestamp == existing.timestamp) {
                self.src.delete_by_timestamp(existing.timestamp).await?;
            }
        }
        Ok(())
    }
}

impl<T: DB> DB for CleanupDB<T> {
    fn insert(&self, data: PriceData) -> impl Future<Output = Result<(), String>> + Send {
        let src = self.src.clone();
        async move {
            let result = src.insert(data).await;
            if result.is_ok() {
                let cleanup_result = self.cleanup_old_data().await;
                if let Err(e) = cleanup_result {
                    log::warn!("Cleanup failed: {}", e);
                }
            }
            result
        }
    }
    fn fetch_all(&self) -> impl Future<Output = Vec<PriceData>> + Send {
        let src = self.src.clone();
        async move {
            src.fetch_all().await
        }
    }
    fn delete_by_timestamp(&self, timestamp: i64) -> impl Future<Output = Result<(), String>> + Send {
        let src = self.src.clone();
        async move {
            src.delete_by_timestamp(timestamp).await
        }
    }
}
