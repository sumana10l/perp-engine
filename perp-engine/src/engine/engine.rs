use std::collections::HashMap;
use uuid::Uuid;

use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;
pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: f64,
    pub balance: f64,
    pub trade_history: Vec<Trade>,
}

impl Engine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            positions: HashMap::new(),
            current_price: 0.0,
            balance: initial_balance,
            trade_history: Vec::new(),
        }
    }
    pub fn open_position(
        &mut self,
        asset: String,
        margin: f64,
        leverage: f64,
        position_type: PositionType,
    ) -> Result<Position, String> {
        if margin <= 0.0 {
            return Err("Margin must be positive".into());
        }

        if leverage <= 0.0 {
            return Err("Leverage must be positive".into());
        }

        if self.current_price <= 0.0 {
            return Err("Market price not initialized".into());
        }

        if margin > self.balance {
            return Err("Insufficient balance".into());
        }

        self.balance -= margin;

        let position_size = margin * leverage;
        let quantity = position_size / self.current_price;
        let entry_price = self.current_price;

        let liquidation_price = if position_type == PositionType::Long {
            entry_price * (1.0 - 1.0 / leverage)
        } else {
            entry_price * (1.0 + 1.0 / leverage)
        };
        let position = Position {
            id: Uuid::new_v4(),
            asset,
            entry_price: self.current_price,
            quantity,
            margin,
            leverage,
            pnl: 0.0,
            position_type,
            liquidation_price,
        };

        self.positions.insert(position.id, position.clone());

        Ok(position)
    }
    pub fn update_price(&mut self, new_price: f64) -> Result<(), String> {
        if new_price <= 0.0 {
            return Err("Price must be positive".into());
        }
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
        Ok(())
    }

    pub fn close_position(&mut self, position_id: Uuid) -> Option<Position> {
        if let Some(position) = self.positions.remove(&position_id) {
    
            let trade = Trade {
                entry: position.entry_price,
                exit: self.current_price,
                pnl: position.pnl,
                position_type: match position.position_type {
                    PositionType::Long => "Long".to_string(),
                    PositionType::Short => "Short".to_string(),
                },
            };
    
            self.trade_history.push(trade);
    
            self.balance += position.margin + position.pnl;
    
            Some(position)
        } else {
            None
        }
    }
}
