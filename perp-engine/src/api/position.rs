use actix_web::{HttpResponse, web};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::engine::engine::Engine;
use crate::engine::position::PositionType;
#[derive(serde::Deserialize)]
pub struct OpenRequest {
    pub asset: String,
    pub margin: f64,
    pub leverage: f64,
    pub position_type: PositionType,
}

pub async fn open_position(
    data: web::Data<Arc<Mutex<Engine>>>,
    req: web::Json<OpenRequest>,
) -> HttpResponse {
    let mut engine = data.lock().unwrap();

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
    let engine = data.lock().unwrap();

    HttpResponse::Ok().json(&engine.positions)
}

#[derive(serde::Deserialize)]
//Only when you need to parse JSON into a struct
pub struct CloseRequest {
    pub position_id: Uuid,
}

pub async fn close_position(
    data: web::Data<Arc<Mutex<Engine>>>,
    req: web::Json<CloseRequest>,
) -> HttpResponse {
    let mut engine = data.lock().unwrap();

    match engine.close_position(req.position_id) {
        Some(position) => HttpResponse::Ok().json(position),
        None => HttpResponse::NotFound().body("Position not found"),
    }
}
pub async fn get_price(
    data: web::Data<Arc<Mutex<Engine>>>,
) -> HttpResponse {
    let engine = data.lock().unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "price": engine.current_price
    }))
}
pub async fn get_balance(
    data: web::Data<Arc<Mutex<Engine>>>,
) -> HttpResponse {
    let engine = data.lock().unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "balance": engine.balance
    }))
}
pub async fn get_trade_history(
    data: web::Data<Arc<Mutex<Engine>>>,
) -> HttpResponse {
    let engine = data.lock().unwrap();

    HttpResponse::Ok().json(&engine.trade_history)
}