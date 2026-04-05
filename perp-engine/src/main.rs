use actix_web::{App, HttpServer, middleware, web};
use crate::auth::middleware::JwtMiddleware;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
mod api;
mod engine;
mod market;
mod auth;  


use crate::api::position::close_position;
use crate::api::position::get_balance;
use crate::api::position::get_funding_rate;
use crate::api::position::get_positions;
use crate::api::position::get_price;
use crate::api::position::get_trade_history;
use crate::api::position::health_check;
use crate::api::position::open_position;
use crate::engine::engine::Engine;
use crate::engine::event::EngineEvent;
use crate::market::ws::start_price_feed;
use crate::api::auth::login; 


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting perp-engine v0");

    let engine = Arc::new(RwLock::new(Engine::new(1000.0)));

    let (tx, mut rx) = tokio::sync::mpsc::channel::<EngineEvent>(100);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        match start_price_feed(tx_clone, "solusdt").await {
            Ok(_) => {
                error!("Price feed exited unexpectedly");
            }
            Err(e) => {
                error!("Price feed error: {}", e);
            }
        }
    });

    let engine_clone = engine.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                EngineEvent::PriceUpdate(price) => {
                    let update_result = {
                        let mut engine = engine_clone.write().await;
                        engine.update_price(price)
                    }; 

                    match update_result {
                        Ok(summary) => {
                            info!(
                                price = %summary.new_price,
                                mark_price = %summary.mark_price,
                                positions_affected = summary.positions_affected,
                                liquidation_count = summary.liquidated_positions.len(),
                                "Price updated"
                            );

                            if !summary.liquidated_positions.is_empty() {
                                for (id, pnl) in summary.liquidated_positions {
                                    warn!(
                                        position_id = %id,
                                        pnl = %pnl,
                                        "Position liquidated"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, price = %price, "Price update failed");
                        }
                    }
                }
            }
        }
        error!("Engine event processor channel closed!");
    });

    let engine_for_funding = engine.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;

            let funding_result = {
                let mut engine = engine_for_funding.write().await;
                engine.apply_funding()
            }; 

            match funding_result {
                Ok(summary) => {
                    info!(
                        rate = %summary.rate,
                        total_applied = %summary.total_funding_applied,
                        liquidation_count = summary.liquidated_ids.len(),
                        timestamp = ?summary.timestamp,
                        "Funding applied"
                    );

                    if summary.total_funding_applied.abs() > rust_decimal_macros::dec!(1000) {
                        error!(
                            total_funding = %summary.total_funding_applied,
                            "⚠️ CRITICAL: Massive funding spike detected!"
                        );
                    }

                    if !summary.liquidated_ids.is_empty() {
                        warn!(
                            count = summary.liquidated_ids.len(),
                            "Positions liquidated by funding"
                        );
                        for (id, pnl) in summary.liquidated_ids {
                            warn!(
                                position_id = %id,
                                final_pnl = %pnl,
                                "Funding liquidation"
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Funding application failed");
                }
            }
        }
    });

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    info!("Starting HTTP server on 0.0.0.0:{}", port);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(JwtMiddleware)   
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(engine.clone()))
            .route("/auth/login", web::post().to(login))  
            .route("/health", web::get().to(health_check)) 
            .route("/position/open", web::post().to(open_position))
            .route("/positions", web::get().to(get_positions))
            .route("/position/close", web::post().to(close_position))
            .route("/price", web::get().to(get_price))
            .route("/balance", web::get().to(get_balance))
            .route("/trade-history", web::get().to(get_trade_history))
            .route("/funding-rate", web::get().to(get_funding_rate))
    })
    .bind(("0.0.0.0", port))?
    .run();

    let server_handle = server.handle();

    let shutdown = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Ctrl+C signal error: {}", e);
        }
        info!("Shutdown signal received, stopping server...");
    };

    tokio::select! {
        _ = shutdown => {
            server_handle.stop(true).await;
            info!("Server stopped gracefully");
        }
        result = server => {
            match result {
                Ok(_) => info!("Server exited normally"),
                Err(e) => error!("Server error: {}", e),
            }
        }
    }

    Ok(())
}