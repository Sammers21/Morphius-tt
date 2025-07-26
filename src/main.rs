mod coingecko;
mod db;
mod ws;

use crate::db::{CachingDB, PostgresDB};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "--api-key" {
        eprintln!("Usage: {} --api-key <key_here>", args[0]);
        std::process::exit(1);
    }
    let pg_url = "postgresql://morphius_user:morphius_password@localhost:5432/morphius?sslmode=disable";
    let api_key = &args[2];
    let api = coingecko::API::new(api_key.clone());
    let pg = PostgresDB::new(pg_url).await?;
    let db = CachingDB::new(pg);
    let ws = ws::WS::new(api, db);
    ws.start();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
