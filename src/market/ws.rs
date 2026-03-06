use futures_util::StreamExt;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::connect_async;

use crate::engine::engine::Engine;

pub async fn start_price_feed(engine: Arc<Mutex<Engine>>) {
    let url = "wss://stream.binance.com:9443/ws/solusdt@trade";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to Binance WS");

    let (_, mut read) = ws_stream.split();
    let mut last_price = 0.0;

    while let Some(message) = read.next().await {
        let msg = message.expect("WS error");

        if msg.is_text() {
            let json: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();

            if let Some(price_str) = json["p"].as_str() {
                let price: f64 = price_str.parse().unwrap();

                let mut engine = engine.lock().unwrap();

                let _ = engine.update_price(price);

                if (price - last_price).abs() > 0.01 {
                    println!("Market price updated: {}", price);
                    last_price = price;
                }
            }
        }
    }
}
