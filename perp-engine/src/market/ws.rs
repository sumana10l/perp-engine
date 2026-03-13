use crate::engine::event::EngineEvent;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn start_price_feed(tx: mpsc::Sender<EngineEvent>) {
    let url = "wss://stream.binance.com:9443/ws/solusdt@trade";

    loop {
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (_, mut read) = ws_stream.split();

                while let Some(message) = read.next().await {
                    match message {
                        Ok(msg) => {
                            if let Message::Text(text) = msg {
                                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                    if let Some(price_str) = json["p"].as_str() {
                                        if let Ok(price) = Decimal::from_str(price_str) {
                                            let _ = tx.send(EngineEvent::PriceUpdate(price)).await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(e) => eprintln!("Connection failed: {}. Retrying...", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
