use crate::coingecko::PriceData;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, Row};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait DB: Clone + Send + Sync + 'static {
    fn insert(&self, data: PriceData) -> impl Future<Output = Result<(), String>> + Send;

    fn fetch_all(&self) -> impl Future<Output = Vec<PriceData>> + Send;
}

#[derive(Clone)]
pub struct CachingDB<T: DB> {
    src: T,
    cache: Arc<RwLock<Option<Vec<PriceData>>>>,
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
                    cached_data.insert(0, data);
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
                return cached_data.clone();
            }
            drop(cache_read);
            let data = src.fetch_all().await;
            let mut cache_write = cache.write().await;
            *cache_write = Some(data.clone());
            data
        }
    }
}

#[derive(Clone)]
pub struct PostgresDB {
    pool: Pool<Postgres>,
}

impl PostgresDB {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS btc_prices (
                id SERIAL PRIMARY KEY,
                price DOUBLE PRECISION NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self { pool })
    }
}

impl DB for PostgresDB {
    fn insert(&self, data: PriceData) -> impl Future<Output = Result<(), String>> + Send {
        let pool = self.pool.clone();
        async move {
            let timestamp =
                DateTime::from_timestamp(data.timestamp, 0).unwrap_or_else(|| Utc::now());
            sqlx::query("INSERT INTO btc_prices (price, timestamp) VALUES ($1, $2)")
                .bind(data.price)
                .bind(timestamp)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }

    fn fetch_all(&self) -> impl Future<Output = Vec<PriceData>> + Send {
        let pool = self.pool.clone();
        async move {
            let rows =
                sqlx::query("SELECT price, timestamp FROM btc_prices ORDER BY timestamp ASC")
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
            rows.into_iter()
                .map(|row| {
                    let timestamp_dt: DateTime<Utc> = row.get("timestamp");
                    PriceData {
                        price: row.get::<f64, _>("price"),
                        timestamp: timestamp_dt.timestamp(),
                    }
                })
                .collect()
        }
    }
}
