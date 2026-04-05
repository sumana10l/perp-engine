use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::auth::create_token;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(req: web::Json<LoginRequest>) -> HttpResponse {
    if req.username == "admin" && req.password == "secret" {
        match create_token(&req.username) {
            Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "expires_in": 3600
            })),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }))
    }
}