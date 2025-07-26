mod coingecko;
mod db;
mod server;
mod caching;
mod postgres;
mod cleanup;

use crate::caching::CachingDB;
use crate::postgres::PostgresDB;
use crate::cleanup::CleanupDB;
use clap::Parser;

#[derive(Parser)]
#[command(name = "morphius")]
#[command(about = "A cryptocurrency tracking application")]
struct Args {
    #[arg(long, help = "API key for CoinGecko")]
    api_key: String,
    #[arg(long, help = "PostgreSQL connection URL")]
    pg_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    let args = Args::parse();
    let api = coingecko::API::new(args.api_key);
    let db = CleanupDB::new(CachingDB::new(PostgresDB::new(&args.pg_url).await?));
    server::Server::new(api, db, 3000).start().await?;
    Ok(())
}
