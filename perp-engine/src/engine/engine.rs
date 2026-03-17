use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::collections::VecDeque;
use uuid::Uuid;

pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: Decimal,
    pub balance: Decimal,
    pub trade_history: Vec<Trade>,
    pub funding_rate: Decimal,
    pub maintenance_margin_rate: Decimal,
    pub last_funding_time: std::time::Instant,
    pub price_history: VecDeque<Decimal>,
}

impl Engine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            positions: HashMap::new(),
            current_price: dec!(0.0),
            balance: Decimal::from_f64(initial_balance).unwrap_or(dec!(0.0)),
            trade_history: Vec::new(),
            funding_rate: dec!(0.0001),
            maintenance_margin_rate: dec!(0.05),
            last_funding_time: std::time::Instant::now(),
            price_history: VecDeque::with_capacity(10),
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

        let maintenance_buffer = dec!(1.0) - self.maintenance_margin_rate; // e.g., 0.95

        let liquidation_price = if position_type == PositionType::Long {
            entry_price * (dec!(1.0) - (maintenance_buffer / leverage))
        } else {
            entry_price * (dec!(1.0) + (maintenance_buffer / leverage))
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
        self.price_history.push_back(new_price);
        if self.price_history.len() > 10 {
            self.price_history.pop_front();
        }

        let sum: Decimal = self.price_history.iter().sum();
        let mark_price = sum / Decimal::from(self.price_history.len());

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

        let to_liquidate: Vec<Uuid> = self
            .positions
            .values()
            .filter(|p| {
                let pnl_at_mark = match p.position_type {
                    PositionType::Long => (mark_price - p.entry_price) * p.quantity,
                    PositionType::Short => (p.entry_price - mark_price) * p.quantity,
                };

                let equity_at_mark = p.margin + pnl_at_mark;
                let maintenance_threshold = p.margin * self.maintenance_margin_rate;

                equity_at_mark <= maintenance_threshold
            })
            .map(|p| p.id)
            .collect();

        for id in to_liquidate {
            println!(
                "⚠️ MARK PRICE LIQUIDATION: Triggered at smoothed price ${}",
                mark_price
            );
            self.close_position(id);
        }

        Ok(())
    }

    pub fn apply_funding(&mut self) {
        let rate = self.funding_rate;

        self.last_funding_time = std::time::Instant::now();

        for position in self.positions.values_mut() {
            let notional_value = position.quantity * self.current_price;
            let funding_amount = notional_value * rate;

            if position.position_type == PositionType::Long {
                position.pnl -= funding_amount;
            } else {
                position.pnl += funding_amount;
            }
        }

        let to_liquidate: Vec<Uuid> = self
            .positions
            .values()
            .filter(|p| {
                let current_equity = p.margin + p.pnl;
                let threshold = p.margin * self.maintenance_margin_rate;
                current_equity <= threshold
            })
            .map(|p| p.id)
            .collect();

        for id in to_liquidate {
            println!(
                "⚠️ FUNDING LIQUIDATION: {} pushed below maintenance margin",
                id
            );
            self.close_position(id);
        }

        println!(
            "Funding applied at rate: {}%. Next window open.",
            rate * dec!(100)
        );
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

            let current_equity = position.margin + position.pnl;

            if current_equity <= (position.margin * self.maintenance_margin_rate) {
                self.balance += current_equity;
                println!(
                    "Liquidation fee of {} added to exchange reserves.",
                    current_equity
                );
            } else {
                self.balance += current_equity;
            }

            Some(position)
        } else {
            None
        }
    }
}
