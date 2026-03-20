use crate::engine::event::EngineEvent;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info, warn};

pub async fn start_price_feed(
    tx: mpsc::Sender<EngineEvent>,
    asset: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "wss://stream.binance.com:9443/ws/{}@trade",
        asset.to_lowercase()
    );

    info!("Connecting to price feed: {}", asset);

    let mut connection_failures = 0;
    let max_retries = 5;

    loop {
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("Connected to Binance price feed for {}", asset);
                connection_failures = 0;

                let (_, mut read) = ws_stream.split();

                loop {
                    match read.next().await {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = handle_price_message(&tx, text).await {
                                error!("Error processing price message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            warn!("WebSocket closed by server");
                            break;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            warn!("WebSocket connection ended");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                connection_failures += 1;
                error!(
                    "Connection failed (attempt #{}/{}): {}",
                    connection_failures, max_retries, e
                );

                if connection_failures >= max_retries {
                    return Err(format!("Max connection retries exceeded: {}", e).into());
                }

                let wait_secs = 2_u64.pow(connection_failures as u32).min(60);
                warn!(
                    "Retrying in {} seconds... ({}/{})",
                    wait_secs, connection_failures, max_retries
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
            }
        }
    }
}

async fn handle_price_message(
    tx: &mpsc::Sender<EngineEvent>,
    text: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::from_str::<Value>(&text)?;

    if let Some(price_str) = json["p"].as_str() {
        let price = Decimal::from_str(price_str)?;

        match tx.send(EngineEvent::PriceUpdate(price)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to send price update to engine: {}", e);
                Err(format!("Channel error: {}", e).into())
            }
        }
    } else {
        Err("Missing price field 'p' in message".into())
    }
}
