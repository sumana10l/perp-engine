use actix_web::{HttpResponse, web};
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::engine::engine::Engine;
use crate::engine::position::PositionType;
use rust_decimal_macros::dec;

#[derive(serde::Deserialize)]
pub struct OpenRequest {
    pub asset: String,
    pub margin: Decimal,
    pub leverage: Decimal,
    pub position_type: PositionType,
}

pub async fn open_position(
    data: web::Data<Arc<Mutex<Engine>>>,
    req: web::Json<OpenRequest>,
) -> HttpResponse {
    let mut engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };

    match engine.open_position(
        req.asset.clone(),
        req.margin,
        req.leverage,
        req.position_type.clone(),
    ) {
        Ok(position) => HttpResponse::Ok().json(position),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

pub async fn get_positions(data: web::Data<Arc<Mutex<Engine>>>) -> HttpResponse {
    let engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };
    HttpResponse::Ok().json(&engine.positions)
}

#[derive(serde::Deserialize)]
pub struct CloseRequest {
    pub position_id: Uuid,
}

pub async fn close_position(
    data: web::Data<Arc<Mutex<Engine>>>,
    req: web::Json<CloseRequest>,
) -> HttpResponse {
    let mut engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };
    match engine.close_position(req.position_id) {
        Some(position) => HttpResponse::Ok().json(position),
        None => HttpResponse::NotFound().body("Position not found"),
    }
}
pub async fn get_price(data: web::Data<Arc<Mutex<Engine>>>) -> HttpResponse {
    let engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };

    let mark_price = if engine.price_history.is_empty() {
        engine.current_price
    } else {
        let sum: Decimal = engine.price_history.iter().sum();
        sum / Decimal::from(engine.price_history.len())
    };

    HttpResponse::Ok().json(serde_json::json!({
        "current_price": engine.current_price,
        "mark_price": mark_price,
        "history_count": engine.price_history.len()
    }))
}
pub async fn get_funding_rate(data: web::Data<Arc<Mutex<Engine>>>) -> HttpResponse {
    let engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "funding_rate": engine.funding_rate,
        "yearly_apr": (engine.funding_rate * dec!(3) * dec!(365)) // Just for trader's perspective
    }))
}
pub async fn get_balance(data: web::Data<Arc<Mutex<Engine>>>) -> HttpResponse {
    let engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };
    HttpResponse::Ok().json(serde_json::json!({
        "balance": engine.balance
    }))
}
pub async fn get_trade_history(data: web::Data<Arc<Mutex<Engine>>>) -> HttpResponse {
    let engine = match data.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Engine lock failed"),
    };
    HttpResponse::Ok().json(&engine.trade_history)
}

