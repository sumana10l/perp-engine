use std::collections::HashMap;
use uuid::Uuid;

use crate::engine::position::Position;

pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: f64,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            current_price: 0.0,
        }
    }
    pub fn open_position(
        &mut self,
        asset: String,
        margin: f64,
        leverage: f64,
    ) -> Position {


        if self.current_price <= 0.0 {
            panic!("Market price not initialized");
        }
        let position_size = margin * leverage;
        let quantity = position_size / self.current_price;
    
        let position = Position {
            id: Uuid::new_v4(),
            asset,
            entry_price: self.current_price,
            quantity,
            margin,
            leverage,
            pnl: 0.0,
        };
    
        self.positions.insert(position.id, position.clone());
    
        position
    }
    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
    
        for position in self.positions.values_mut() {
            let pnl = (self.current_price - position.entry_price) * position.quantity;
            position.pnl = pnl;
        }
    }
}