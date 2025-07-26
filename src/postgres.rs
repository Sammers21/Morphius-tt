use crate::coingecko::PriceData;
use crate::db::DB;
use chrono::{DateTime, Utc};
use log::info;
use sqlx::{Pool, Postgres, Row};
use std::future::Future;

/// The `PostgresDB` is the PostgreSQL implementation of DB
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
                timestamp TIMESTAMPTZ PRIMARY KEY,
                price DOUBLE PRECISION NOT NULL
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
            sqlx::query("INSERT INTO btc_prices (timestamp, price) VALUES ($1, $2) ON CONFLICT (timestamp) DO UPDATE SET price = EXCLUDED.price")
                .bind(timestamp)
                .bind(data.price)
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

    fn delete_by_timestamp(
        &self,
        timestamp: i64,
    ) -> impl Future<Output = Result<(), String>> + Send {
        let pool = self.pool.clone();
        async move {
            let timestamp_dt = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
            let result = sqlx::query("DELETE FROM btc_prices WHERE timestamp = $1")
                .bind(timestamp_dt)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            let rows_affected = result.rows_affected();
            let readable_time = chrono::DateTime::from_timestamp(timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "invalid timestamp".to_string());
            info!(
                "Deleted {} rows with timestamp {}",
                rows_affected, readable_time
            );
            Ok(())
        }
    }
}
