use actix_web::{HttpResponse, web, HttpRequest};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use validator::Validate;

use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;
use crate::engine::multi_user_engine::MultiUserEngine;
use actix_web::HttpMessage;

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

fn get_user_id(req: &HttpRequest) -> Result<String, HttpResponse> {
    req.extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ErrorResponse {
                code: "NO_USER_ID".to_string(),
                message: "User ID not found in request".to_string(),
            })
        })
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn open_position(
    req: HttpRequest,
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
    req_body: web::Json<OpenPositionRequest>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    if let Err(e) = req_body.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Validation failed: {}", e),
        });
    }

    let mut engine = data.write().await;

    let user_account = engine.get_or_create_user(&user_id, 1000.0);

    match user_account.engine.open_position(
        &req_body.asset,
        req_body.margin,
        req_body.leverage,
        req_body.position_type.clone(),
    ) {
        Ok(position_id) => {
            if let Some(position) = user_account.engine.get_position(position_id) {
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

pub async fn get_positions(
    req: HttpRequest,
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let engine = data.read().await;

    match engine.get_user(&user_id) {
        Some(user_account) => {
            let all_positions = user_account.engine.get_all_positions();
            let positions: Vec<Position> = all_positions.iter().map(|&p| p.clone()).collect();
            let total = positions.len();

            HttpResponse::Ok().json(PositionsResponse { positions, total })
        }
        None => HttpResponse::Ok().json(PositionsResponse {
            positions: vec![],
            total: 0,
        }),
    }
}

pub async fn close_position(
    req: HttpRequest,
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
    req_body: web::Json<ClosePositionRequest>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let mut engine = data.write().await;

    match engine.get_user_mut(&user_id) {
        Some(user_account) => {
            match user_account.engine.close_position(req_body.position_id) {
                Ok((pnl, equity_returned)) => HttpResponse::Ok().json(serde_json::json!({
                    "position_id": req_body.position_id,
                    "pnl": pnl,
                    "equity_returned": equity_returned,
                    "closed_at": chrono::Utc::now(),
                })),
                Err(e) => HttpResponse::NotFound().json(ErrorResponse {
                    code: "POSITION_NOT_FOUND".to_string(),
                    message: e,
                }),
            }
        }
        None => HttpResponse::NotFound().json(ErrorResponse {
            code: "USER_NOT_FOUND".to_string(),
            message: "User account not found".to_string(),
        }),
    }
}

pub async fn get_price(
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
) -> HttpResponse {
    let engine = data.read().await;

    if let Some(first_user) = engine.get_all_users().first() {
        if let Some(user_account) = engine.get_user(first_user) {
            let current_price = user_account.engine.current_price;
            let history_len = user_account.engine.price_history.len();
            let mark_price = user_account.engine.mark_price;

            return HttpResponse::Ok().json(serde_json::json!({
                "current_price": current_price,
                "mark_price": mark_price,
                "price_history_length": history_len,
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "current_price": "No price data",
        "mark_price": "No price data",
    }))
}

pub async fn get_funding_rate(
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
) -> HttpResponse {
    let engine = data.read().await;

    if let Some(first_user) = engine.get_all_users().first() {
        if let Some(user_account) = engine.get_user(first_user) {
            return HttpResponse::Ok().json(serde_json::json!({
                "funding_rate": user_account.engine.funding_rate,
                "yearly_apr": user_account.engine.funding_rate * Decimal::from(365) * Decimal::from(24),
                "last_funding_time": user_account.engine.last_funding_time.elapsed().as_secs(),
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"error": "No data"}))
}

pub async fn get_balance(
    req: HttpRequest,
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let engine = data.read().await;

    match engine.get_user(&user_id) {
        Some(user_account) => {
            HttpResponse::Ok().json(serde_json::json!({
                "balance": user_account.engine.balance,
                "total_equity": user_account.engine.get_total_equity(),
                "currency": "USDT",
            }))
        }
        None => HttpResponse::Ok().json(serde_json::json!({
            "balance": 1000.0,
            "total_equity": 1000.0,
            "currency": "USDT",
        })),
    }
}

pub async fn get_trade_history(
    req: HttpRequest,
    data: web::Data<Arc<RwLock<MultiUserEngine>>>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let engine = data.read().await;

    match engine.get_user(&user_id) {
        Some(user_account) => {
            let trades: Vec<Trade> = user_account.engine.trade_history.iter().cloned().collect();
            let total_trades = trades.len();

            HttpResponse::Ok().json(TradesResponse {
                trades,
                total_trades,
            })
        }
        None => HttpResponse::Ok().json(TradesResponse {
            trades: vec![],
            total_trades: 0,
        }),
    }
}