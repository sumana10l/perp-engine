use crate::engine::position::{Position, PositionType};
use crate::engine::trade::Trade;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::collections::VecDeque;
use uuid::Uuid;
use tracing::info;


#[derive(Clone, Debug)]
pub struct UpdateResult {
    pub new_price: Decimal,
    pub mark_price: Decimal,
    pub liquidated_positions: Vec<(Uuid, Decimal)>, 
    pub positions_affected: usize,
}

#[derive(Clone, Debug)]
pub struct FundingResult {
    pub rate: Decimal,
    pub total_funding_applied: Decimal,
    pub liquidated_ids: Vec<(Uuid, Decimal)>,
    pub timestamp: std::time::Instant,
}


pub struct Engine {
    pub positions: HashMap<Uuid, Position>,
    pub current_price: Decimal,
    pub balance: Decimal,
    pub trade_history: Vec<Trade>,
    pub funding_rate: Decimal,
    pub maintenance_margin_rate: Decimal,
    pub last_funding_time: std::time::Instant,
    pub price_history: VecDeque<Decimal>,
    pub max_leverage: Decimal,
    pub max_positions: usize,
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
            max_leverage: dec!(100),
            max_positions: 50,
        }
    }

    pub fn open_position(
        &mut self,
        asset: &str,
        margin: Decimal,
        leverage: Decimal,
        position_type: PositionType,
    ) -> Result<Uuid, String> {
    
        if asset.is_empty() || asset.len() > 20 {
            return Err("Invalid asset name. Must be 1-20 characters.".into());
        }
    
        if margin <= dec!(0.0) {
            return Err("Margin must be positive".into());
        }
    
        if margin > self.balance {
            return Err(format!(
                "Insufficient balance. Available: {}, Required: {}",
                self.balance, margin
            ));
        }
    
        if leverage < dec!(1.0) {
            return Err("Leverage must be at least 1x".into());
        }
    
        if leverage > self.max_leverage {
            return Err(format!(
                "Leverage must not exceed {}x",
                self.max_leverage
            ));
        }
    
        if self.current_price <= dec!(0.0) {
            return Err("Market price not initialized".into());
        }
    
        if self.positions.len() >= self.max_positions {
            return Err(format!(
                "Maximum number of positions ({}) reached",
                self.max_positions
            ));
        }
    
        self.balance -= margin;
    
        let position_size = margin * leverage;
        let quantity = position_size / self.current_price;
        let entry_price = self.current_price;
    
        let maintenance_buffer = dec!(1.0) - self.maintenance_margin_rate;
    
        let liquidation_price = match position_type {
            PositionType::Long => {
                entry_price * (dec!(1.0) - (maintenance_buffer / leverage))
            }
            PositionType::Short => {
                entry_price * (dec!(1.0) + (maintenance_buffer / leverage))
            }
        };
    
        let position = Position {
            id: Uuid::new_v4(),
            asset: asset.to_string(),
            entry_price,
            quantity,
            margin,
            leverage,
            pnl: dec!(0.0),
            position_type,
            liquidation_price,
        };
    
        let position_id = position.id;
        self.positions.insert(position_id, position);
    
        info!(
            "Position opened: id={}, asset={}, entry={}, qty={}, margin={}, leverage={}x",
            position_id, asset, entry_price, quantity, margin, leverage
        );
    
        Ok(position_id)
    }

    pub fn update_price(&mut self, new_price: Decimal) -> Result<UpdateResult, String> {
        if new_price <= dec!(0.0) {
            return Err("Price must be positive".into());
        }

        self.current_price = new_price;
        self.price_history.push_back(new_price);
        if self.price_history.len() > 10 {
            self.price_history.pop_front();
        }

        let mark_price = if self.price_history.is_empty() {
            new_price
        } else {
            let sum: Decimal = self.price_history.iter().sum();
            sum / Decimal::from(self.price_history.len())
        };

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

        let positions_affected = self.positions.len();

        // Find liquidations
        let to_liquidate: Vec<Uuid> = self
            .positions
            .values()
            .filter(|p| self.should_liquidate(p))
            .map(|p| p.id)
            .collect();

        let mut liquidated_positions = Vec::new();

        for id in to_liquidate {
            if let Some(pos) = self.positions.get(&id) {
                liquidated_positions.push((id, pos.pnl));
            }
            self.close_position(id)?;
        }

        Ok(UpdateResult {
            new_price,
            mark_price,
            liquidated_positions,
            positions_affected,
        })
    }

    pub fn should_liquidate(&self, position: &Position) -> bool {
        let current_equity = position.margin + position.pnl;
        let maintenance_threshold = position.margin * self.maintenance_margin_rate;
        current_equity <= maintenance_threshold
    }

    pub fn apply_funding(&mut self) -> Result<FundingResult, String> {
        let rate = self.funding_rate;
        let mut total_applied = Decimal::ZERO;

        // Apply funding to all positions
        for position in self.positions.values_mut() {
            let notional_value = position.quantity * self.current_price;
            let funding_amount = notional_value * rate;

            if position.position_type == PositionType::Long {
                position.pnl -= funding_amount;
                total_applied += funding_amount;
            } else {
                position.pnl += funding_amount;
                total_applied -= funding_amount;
            }
        }

        self.last_funding_time = std::time::Instant::now();

        let to_liquidate: Vec<Uuid> = self
            .positions
            .values()
            .filter(|p| self.should_liquidate(p))
            .map(|p| p.id)
            .collect();

        let mut liquidated_ids = Vec::new();

        for id in to_liquidate {
            if let Some(pos) = self.positions.get(&id) {
                liquidated_ids.push((id, pos.pnl));
            }
            self.close_position(id)?;
        }

        Ok(FundingResult {
            rate,
            total_funding_applied: total_applied,
            liquidated_ids,
            timestamp: self.last_funding_time,
        })
    }

    pub fn close_position(&mut self, position_id: Uuid) -> Result<Decimal, String> {
        let position = self.positions.remove(&position_id).ok_or_else(|| {
            format!("Position {} not found or already closed", position_id)
        })?;

        let current_equity = position.margin + position.pnl;

        let trade = Trade {
            entry: position.entry_price,
            exit: self.current_price,
            pnl: position.pnl,
            position_type: position.position_type.clone(),
        };
        self.trade_history.push(trade);

        self.balance += current_equity;

        info!(
            "Position closed: id={}, pnl={}, exit_price={}",
            position_id, position.pnl, self.current_price
        );

        Ok(current_equity)
    }

    pub fn get_position(&self, position_id: Uuid) -> Option<&Position> {
        self.positions.get(&position_id)
    }

    pub fn get_all_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    pub fn get_total_equity(&self) -> Decimal {
        let open_positions_pnl: Decimal = self.positions.values().map(|p| p.pnl).sum();
        self.balance + open_positions_pnl
    }
}