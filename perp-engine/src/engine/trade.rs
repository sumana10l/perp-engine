use serde::{Serialize, Deserialize};
use crate::engine::position::PositionType;

#[derive(Serialize, Deserialize, Clone)]
pub struct Trade {
    pub entry: f64,
    pub exit: f64,
    pub pnl: f64,
    pub position_type: PositionType,
}