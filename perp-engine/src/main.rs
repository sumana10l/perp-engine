use actix_web::{App, HttpServer, web};
use std::sync::{Arc, Mutex};
mod api;
mod engine;
mod market;
use crate::api::position::close_position;
use crate::api::position::get_balance;
use crate::api::position::get_positions;
use crate::api::position::get_price;
use crate::api::position::get_trade_history;
use crate::api::position::open_position;
use crate::engine::engine::Engine;
use crate::engine::event::EngineEvent;
use crate::market::ws::start_price_feed;
use actix_cors::Cors;
use tokio::sync::mpsc;
#[actix_web::main]

async fn main() -> std::io::Result<()> {
    let engine = Arc::new(Mutex::new(Engine::new(1000.0)));
    // move transfers ownership of captured variables into the closure.
    let (tx, mut rx) = mpsc::channel::<EngineEvent>(100);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        start_price_feed(tx_clone).await;
    });
    let engine_clone = engine.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut engine = engine_clone.lock().unwrap();

            match event {
                EngineEvent::PriceUpdate(price) => {
                    let _ = engine.update_price(price);
                }
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(engine.clone()))
            .route("/position/open", web::post().to(open_position))
            .route("/positions", web::get().to(get_positions))
            .route("/position/close", web::post().to(close_position))
            .route("/price", web::get().to(get_price))
            .route("/balance", web::get().to(get_balance))
            .route("/trade-history", web::get().to(get_trade_history))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
