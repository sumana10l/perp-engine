use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Trade {
    pub entry: f64,
    pub exit: f64,
    pub pnl: f64,
    pub position_type: String,
}