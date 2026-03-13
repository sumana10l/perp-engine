use serde::{Serialize, Deserialize};
use crate::engine::position::PositionType;
use rust_decimal::Decimal; 

#[derive(Serialize, Deserialize, Clone)]
pub struct Trade {
    pub entry: Decimal,         
    pub exit: Decimal,          
    pub pnl: Decimal,           
    pub position_type: PositionType,
}