mod ws;
mod db;
mod btc;

use axum::{
    extract::{ws::{Message, WebSocket}, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router
};
use rand::Rng;
use serde::Serialize;
use std::time::Duration;
use tokio::time::interval;
use tower_http::services::ServeDir;


#[derive(Serialize)]
struct PriceData {
    price: f64,
    timestamp: u64,
}

// Thread-safe function to generate a random price
fn generate_price() -> f64 {
    // Base price around 100k
    let base_price = 100_000.0;
    
    // Create a new random number generator
    let mut rng = rand::thread_rng();
    
    // Generate a random price fluctuation (±10k)
    let fluctuation = rng.gen_range(-10_000.0..10_000.0);
    
    // Return the price
    base_price + fluctuation
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // This closure is 'static and 'Send' because it doesn't capture any non-Send types
    ws.on_upgrade(|socket| async move {
        handle_socket(socket).await
    })
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = interval(Duration::from_secs(1));
    
    // Initial timestamp in seconds since epoch
    let mut current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Simple task that sends price updates every second
    loop {
        // Wait for the next interval tick
        interval.tick().await;
        
        // Generate price (thread-safe function call)
        let price = generate_price();
        
        // Increment timestamp
        current_time += 1;
        
        // Create price data
        let price_data = PriceData {
            price,
            timestamp: current_time,
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&price_data).unwrap();
        
        // Send the price data
        if socket.send(Message::Text(json)).await.is_err() {
            // If sending fails, the client has disconnected
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        // websocket
        .route("/ws", get(ws_handler))
        // serving frontend
        .nest_service("/", ServeDir::new("static"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
