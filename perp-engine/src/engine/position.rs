use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionType {
    Long,
    Short,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub id: Uuid,
    pub asset: String,
    pub entry_price: Decimal, 
    pub quantity: Decimal,    
    pub margin: Decimal,      
    pub leverage: Decimal,    
    pub pnl: Decimal,         
    pub position_type: PositionType,
    pub liquidation_price: Decimal, 
}
