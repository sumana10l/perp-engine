use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
pub mod middleware;  
const SECRET: &[u8] = b"your-secret-key-change-in-prod";

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   
    pub exp: usize,   
}

pub fn create_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 3600; 

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )?;
    Ok(data.claims)
}