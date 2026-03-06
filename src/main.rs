use actix_web::{App, HttpServer, web};
use std::sync::{Arc, Mutex};
mod api;
mod engine;
mod market;
use crate::api::position::close_position;
use crate::api::position::get_positions;
use crate::api::position::open_position;
use crate::engine::engine::Engine;
use crate::market::ws::start_price_feed;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    let engine = Arc::new(Mutex::new(Engine::new(1000.0)));
    // move transfers ownership of captured variables into the closure.
    let engine_clone = engine.clone();

    tokio::spawn(async move {
        start_price_feed(engine_clone).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(engine.clone()))
            .route("/position/open", web::post().to(open_position))
            .route("/positions", web::get().to(get_positions))
            .route("/position/close", web::post().to(close_position))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
