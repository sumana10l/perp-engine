use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: Decimal,
    pub balance: Decimal,
    pub trade_history: Vec<Trade>,
}

impl Engine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            positions: HashMap::new(),
            current_price: dec!(0.0),
            balance: Decimal::from_f64(initial_balance).unwrap_or(dec!(0.0)),
            trade_history: Vec::new(),
        }
    }

    pub fn open_position(
        &mut self,
        asset: String,
        margin: Decimal,
        leverage: Decimal,
        position_type: PositionType,
    ) -> Result<Position, String> {
        if margin <= dec!(0.0) {
            return Err("Margin must be positive".into());
        }

        if leverage <= dec!(0.0) {
            return Err("Leverage must be positive".into());
        }

        if self.current_price <= dec!(0.0) {
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
            entry_price * (dec!(1.0) - (dec!(1.0) / leverage))
        } else {
            entry_price * (dec!(1.0) + (dec!(1.0) / leverage))
        };

        let position = Position {
            id: Uuid::new_v4(),
            asset,
            entry_price: self.current_price,
            quantity,
            margin,
            leverage,
            pnl: dec!(0.0),
            position_type,
            liquidation_price,
        };

        self.positions.insert(position.id, position.clone());

        Ok(position)
    }

    pub fn update_price(&mut self, new_price: Decimal) -> Result<(), String> {
        if new_price <= dec!(0.0) {
            return Err("Price must be positive".into());
        }
        self.current_price = new_price;

        let to_liquidate: Vec<Uuid> = self
            .positions
            .values()
            .filter(|p| match p.position_type {
                PositionType::Long => self.current_price <= p.liquidation_price,
                PositionType::Short => self.current_price >= p.liquidation_price,
            })
            .map(|p| p.id)
            .collect();

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

        for id in to_liquidate {
            println!("Liquidating position: {}", id);
            self.close_position(id);
        }

        Ok(())
    }

    pub fn close_position(&mut self, position_id: Uuid) -> Option<Position> {
        if let Some(position) = self.positions.remove(&position_id) {
            let trade = Trade {
                entry: position.entry_price,
                exit: self.current_price,
                pnl: position.pnl,
                position_type: position.position_type.clone(),
            };

            self.trade_history.push(trade);

            self.balance += position.margin + position.pnl;

            Some(position)
        } else {
            None
        }
    }
}
