mod caching;
mod cleanup;
mod coingecko;
mod db;
mod postgres;
mod server;

use crate::caching::CachingDB;
use crate::cleanup::CleanupDB;
use crate::postgres::PostgresDB;
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
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };
    log::info!("API key: {}", args.api_key);
    log::info!("PostgreSQL connection URL: {}", args.pg_url);
    let api = coingecko::API::new(args.api_key);
    let db = CleanupDB::new(CachingDB::new(PostgresDB::new(&args.pg_url).await?));
    server::Server::new(api, db, 3000).start().await?;
    Ok(())
}
