// PnL model:
//
// Core idea:
// - PnL = (current_price - entry_price) * position_size (LONG)
// - PnL = (entry_price - current_price) * position_size (SHORT)
//
// Key drivers:
// - Price movement (primary)
// - Leverage amplifies position size → magnifies PnL
//
// Invariants:
// - PnL = 0 at entry price
// - Symmetric behavior for LONG vs SHORT
// - Unrealized PnL updates on every price change
//
// System impact:
// - PnL directly affects equity → drives liquidation decisions
#[cfg(test)]
mod pnl_tests {
    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    /// Verifies LONG PnL increases when price rises:
    /// - Positive price delta → positive PnL
    /// Confirms correct directional behavior.
    #[test]
    fn test_long_position_pnl_profit_on_rise() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(110))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(100));
    }

    /// Verifies SHORT PnL increases when price falls:
    /// - Negative price delta → positive PnL for SHORT
    /// Confirms inverse relationship vs LONG.
    #[test]
    fn test_short_position_pnl_profit_on_fall() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open position");

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(100));
    }
    /// Ensures PnL is zero at break-even:
    /// - current_price == entry_price → pnl = 0
    /// Validates core invariant.
    #[test]
    fn test_pnl_at_break_even() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(100))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(0));
    }
    /// Validates PnL updates across multiple price movements:
    /// - PnL recalculates based on latest price (no accumulation drift)
    /// Confirms stateless calculation relative to entry.
    #[test]
    fn test_pnl_with_various_price_movements() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
            .expect("Failed to open position");

        engine.update_price(dec!(105)).unwrap();
        assert_eq!(engine.get_position(position_id).unwrap().pnl, dec!(25));

        engine.update_price(dec!(98)).unwrap();
        assert_eq!(engine.get_position(position_id).unwrap().pnl, dec!(-10));

        engine.update_price(dec!(110)).unwrap();
        assert_eq!(engine.get_position(position_id).unwrap().pnl, dec!(50));
    }
    /// Tests large gains under high leverage:
    /// - Higher leverage → amplified PnL
    /// Confirms linear scaling with position size.
    #[test]
    fn test_pnl_with_large_gains() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(100), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(200))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(10000));
    }
    /// Ensures losses are computed correctly before liquidation:
    /// - Position remains active if threshold not breached
    /// Confirms PnL accuracy independent of liquidation logic.
    #[test]
    fn test_pnl_with_large_losses_before_liquidation() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(2), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(60))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position should exist");
        assert_eq!(position.pnl, dec!(-80));
    }
    /// Validates aggregate PnL across multiple positions:
    /// - LONG and SHORT offsets should net to ~0
    /// Confirms system-level balance.
    /// Also verifies total equity calculation.
    #[test]
    fn test_aggregate_multiple_positions_pnl() {
        let mut engine = Engine::new(5000.0);
        engine.current_price = dec!(100);

        let _pos1_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .unwrap();
        let _pos2_id = engine
            .open_position("ETH", dec!(100), dec!(10), PositionType::Short)
            .unwrap();

        engine
            .update_price(dec!(105))
            .expect("Failed to update price");

        let total_pnl: Decimal = engine.get_all_positions().iter().map(|p| p.pnl).sum();
        assert_eq!(total_pnl, dec!(0));

        assert_eq!(engine.get_total_equity(), dec!(4800));
    }
    /// Ensures PnL consistency across price fluctuations:
    /// - Returning to entry price resets PnL to zero
    /// Confirms no historical drift or accumulation error.
    #[test]
    fn test_pnl_consistency_same_price() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(2), PositionType::Long)
            .unwrap();

        let prices = vec![dec!(150), dec!(110), dec!(70), dec!(100)];

        for price in prices {
            engine.update_price(price).unwrap();
        }

        let position = engine
            .get_position(position_id)
            .expect("Position should have survived");
        assert_eq!(
            position.pnl,
            dec!(0),
            "PnL must return to 0 when price returns to entry"
        );
    }
    /// Ensures funding directly impacts PnL:
    /// - Funding applied as adjustment to PnL
    /// Confirms integration between funding and PnL systems.
    #[test]
    fn test_funding_impact_on_pnl() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);
        engine.funding_rate = dec!(0.01);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine.apply_funding().expect("Failed to apply funding");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(-10));
    }
    /// Validates post-liquidation state:
    /// - Position removed from active state
    /// - Final PnL recorded in trade history
    /// Confirms correct lifecycle transition.
    #[test]
    fn test_pnl_after_liquidation_closed_state() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(50))
            .expect("Failed to update price");

        assert!(engine.get_position(position_id).is_none());

        let last_trade = engine
            .trade_history
            .last()
            .expect("Trade history should have entry");
        assert!(last_trade.pnl < dec!(0));
        assert_eq!(last_trade.exit, dec!(50));
    }
}
