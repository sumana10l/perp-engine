use actix_web::HttpResponse;
use actix_web::web;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use validator::Validate;

use crate::engine::engine::Engine;
use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;

#[derive(Deserialize, Validate)]
pub struct OpenPositionRequest {
    #[validate(length(min = 1, max = 20))]
    pub asset: String,

    #[validate(custom = "validate_positive")]
    pub margin: Decimal,

    #[validate(custom = "validate_leverage")]
    pub leverage: Decimal,

    pub position_type: PositionType,
}

#[derive(Deserialize)]
pub struct ClosePositionRequest {
    pub position_id: Uuid,
}

#[derive(Serialize)]
pub struct PositionsResponse {
    pub positions: Vec<Position>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
    pub total_trades: usize,
}
#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

fn validate_positive(value: &Decimal) -> Result<(), validator::ValidationError> {
    if *value <= Decimal::ZERO {
        return Err(validator::ValidationError::new("must_be_positive"));
    }
    Ok(())
}

fn validate_leverage(value: &Decimal) -> Result<(), validator::ValidationError> {
    if *value < Decimal::ONE || *value > Decimal::from(100) {
        return Err(validator::ValidationError::new("leverage_out_of_range"));
    }
    Ok(())
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn open_position(
    data: web::Data<Arc<RwLock<Engine>>>,
    req: web::Json<OpenPositionRequest>,
) -> HttpResponse {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Validation failed: {}", e),
        });
    }

    let mut engine = data.write().await;

    match engine.open_position(
        &req.asset,
        req.margin,
        req.leverage,
        req.position_type.clone(),
    ) {
        Ok(position_id) => {
            if let Some(position) = engine.get_position(position_id) {
                HttpResponse::Created().json(position.clone())
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "Position created but not found".to_string(),
                })
            }
        }
        Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
            code: "POSITION_OPEN_FAILED".to_string(),
            message: err,
        }),
    }
}

pub async fn get_positions(data: web::Data<Arc<RwLock<Engine>>>) -> HttpResponse {
    let engine = data.read().await;

    let all_positions = engine.get_all_positions();
    let positions: Vec<Position> = all_positions.iter().map(|&p| p.clone()).collect();

    let total = positions.len();

    drop(engine);

    HttpResponse::Ok().json(PositionsResponse { positions, total })
}
pub async fn close_position(
    data: web::Data<Arc<RwLock<Engine>>>,
    req: web::Json<ClosePositionRequest>,
) -> HttpResponse {
    let mut engine = data.write().await;

    match engine.close_position(req.position_id) {
        Ok(pnl) => HttpResponse::Ok().json(serde_json::json!({
            "position_id": req.position_id,
            "pnl": pnl,
            "closed_at": chrono::Utc::now(),
        })),
        Err(e) => HttpResponse::NotFound().json(ErrorResponse {
            code: "POSITION_NOT_FOUND".to_string(),
            message: e,
        }),
    }
}

pub async fn get_price(data: web::Data<Arc<RwLock<Engine>>>) -> HttpResponse {
    let engine = data.read().await;
    let current_price = engine.current_price;
    let history_len = engine.price_history.len();

    let mark_price = if engine.price_history.is_empty() {
        engine.current_price
    } else {
        let sum: Decimal = engine.price_history.iter().sum();
        sum / Decimal::from(engine.price_history.len())
    };

    drop(engine);

    HttpResponse::Ok().json(serde_json::json!({
        "current_price": current_price,
        "mark_price": mark_price,
        "price_history_length": history_len,
    }))
}

pub async fn get_funding_rate(data: web::Data<Arc<RwLock<Engine>>>) -> HttpResponse {
    let engine = data.read().await;

    HttpResponse::Ok().json(serde_json::json!({
        "funding_rate": engine.funding_rate,
        "yearly_apr": engine.funding_rate * Decimal::from(365) * Decimal::from(8),
        "last_funding_time": engine.last_funding_time.elapsed().as_secs(),
    }))
}

pub async fn get_balance(data: web::Data<Arc<RwLock<Engine>>>) -> HttpResponse {
    let engine = data.read().await;

    HttpResponse::Ok().json(serde_json::json!({
        "balance": engine.balance,
        "total_equity": engine.get_total_equity(),
        "currency": "USDT",
    }))
}

pub async fn get_trade_history(data: web::Data<Arc<RwLock<Engine>>>) -> HttpResponse {
    let engine = data.read().await;

    let trades: Vec<Trade> = engine.trade_history.iter().cloned().collect();
    let total_trades = trades.len();

    drop(engine);

    HttpResponse::Ok().json(TradesResponse {
        trades,
        total_trades,
    })
}
