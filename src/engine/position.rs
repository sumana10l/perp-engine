use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PositionType {
    Long,
    Short,
}
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Position {
    pub id: Uuid,
    pub asset: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub margin: f64,
    pub leverage: f64,
    pub pnl: f64,
    pub position_type: PositionType,
}
