use crate::coingecko::{self, PriceData};
use crate::db::DB;
use axum::extract::ws::{Message, WebSocket};
use axum::http::{HeaderValue, header};
use axum::{Router, extract::WebSocketUpgrade, response::Response, serve};
use serde_json;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use log::{error, info, warn};

pub(crate) struct Server<T: DB> {
    api: coingecko::API,
    db: T,
    port: u32,
    price_sender: broadcast::Sender<PriceData>,
}

impl<T: DB> Server<T> {
    pub fn new(api: coingecko::API, db: T, port: u32) -> Self {
        let (price_sender, _) = broadcast::channel(100);
        Self {
            api,
            db,
            port,
            price_sender,
        }
    }
    async fn websocket_handler(
        ws: WebSocketUpgrade,
        db: T,
        price_sender: broadcast::Sender<PriceData>,
    ) -> Response {
        ws.on_upgrade(move |socket| handle_socket(socket, db, price_sender))
    }
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let api = self.api.clone();
        let db = self.db.clone();
        let price_sender = self.price_sender.clone();
        let db_for_ws = self.db.clone();
        let price_sender_for_ws = self.price_sender.clone();
        tokio::spawn(async move {
            loop {
                match api.btc_price().await {
                    Ok(price_data) => match db.insert(price_data.clone()).await {
                        Ok(_) => {
                            info!("NEW BTC Price inserted to DB: ${}", price_data.price);
                            let _ = price_sender.send(price_data);
                        }
                        Err(e) => error!("Failed to insert price data: {}", e),
                    },
                    Err(e) => {
                        error!("Failed to fetch BTC price: {}", e);
                    }
                }
                sleep(Duration::from_secs(1)).await;
            }
        });
        let app = Router::new()
            .route(
                "/ws",
                axum::routing::get(move |ws| {
                    Self::websocket_handler(ws, db_for_ws, price_sender_for_ws)
                }),
            )
            .nest_service("/", ServeDir::new("static"))
            .layer(SetResponseHeaderLayer::overriding(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::PRAGMA,
                HeaderValue::from_static("no-cache"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::EXPIRES,
                HeaderValue::from_static("0"),
            ));
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&bind_addr).await?;
        info!("Server running on http://localhost:{}", self.port);
        serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_socket<T: DB>(
    mut socket: WebSocket,
    db: T,
    price_sender: broadcast::Sender<PriceData>,
) {
    let data = db.fetch_all().await;
    for price_data in data {
        match serde_json::to_string(&price_data) {
            Ok(json_data) => {
                if let Err(e) = socket.send(Message::Text(json_data)).await {
                    error!("Failed to send historical data to websocket: {}", e);
                    return;
                }
            }
            Err(e) => {
                error!("Failed to serialize historical data: {}", e);
                return;
            }
        }
    }
    let mut price_receiver = price_sender.subscribe();
    loop {
        match price_receiver.recv().await {
            Ok(price_data) => match serde_json::to_string(&price_data) {
                Ok(json_data) => {
                    if let Err(e) = socket.send(Message::Text(json_data)).await {
                        error!("Failed to send real-time data to websocket: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize real-time data: {}", e);
                    break;
                }
            },
            Err(broadcast::error::RecvError::Closed) => {
                warn!("Price broadcast channel closed");
                break;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                warn!("WebSocket client lagged behind, skipping messages");
                continue;
            }
        }
    }
}
