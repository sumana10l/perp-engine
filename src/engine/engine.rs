use std::collections::HashMap;
use uuid::Uuid;

use crate::engine::position::{Position, PositionType};

pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: f64,
    pub balance: f64,
}

impl Engine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            positions: HashMap::new(),
            current_price: 0.0,
            balance: initial_balance,
        }
    }
    pub fn open_position(
        &mut self,
        asset: String,
        margin: f64,
        leverage: f64,
        position_type: PositionType,
    ) -> Position {
        if self.current_price <= 0.0 {
            panic!("Market price not initialized");
        }

        if margin > self.balance {
            panic!("Insufficient balance");
        }

        self.balance -= margin;

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
            position_type,
        };

        self.positions.insert(position.id, position.clone());

        position
    }
    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;

        for position in self.positions.values_mut() {
            let pnl = match position.position_type {
                PositionType::Long => {
                    (self.current_price - position.entry_price) * position.quantity
                }
                PositionType::Short => {
                    (position.entry_price - self.current_price) * position.quantity
                }
            };

            position.pnl = pnl;
        }
    }

    pub fn close_position(&mut self, position_id: Uuid) -> Option<Position> {
        if let Some(position) = self.positions.remove(&position_id) {
            self.balance += position.margin + position.pnl;
            Some(position)
        } else {
            None
        }
    }
}
