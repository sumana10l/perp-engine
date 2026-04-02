// Liquidation model:
// - Positions are liquidated when losses reach ~95% of margin
// - Remaining ~5% acts as buffer (fees / insurance)
// - Triggered on:
//   1. Price updates (mark price crossing threshold)
//   2. Accumulated funding reducing margin
// - Liquidated positions are fully removed from engine state

#[cfg(test)]
mod liquidation_tests {
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;
    
    /// Verifies liquidation price formula for LONG positions.
    /// Expected: liq_price = entry * (1 - 0.95 / leverage)
    /// Ensures margin buffer is enforced.
   
    #[test]
    fn test_liquidation_price_long() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        let expected_liq_price = dec!(90.50);

        assert_eq!(position.liquidation_price, expected_liq_price);
    }

    /// Verifies liquidation price formula for SHORT positions.
    /// Expected: liq_price = entry * (1 + 0.95 / leverage
  
    #[test]
    fn test_liquidation_price_short() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        let expected_liq_price = dec!(109.50);

        assert_eq!(position.liquidation_price, expected_liq_price);
    }
    /// Ensures liquidation triggers correctly:
    /// - No liquidation above threshold
    /// - Liquidation at or below threshold
    /// Also verifies position removal from engine state.

    #[test]
    fn test_should_liquidate_function() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert!(!engine.should_liquidate(position));

        engine
            .update_price(dec!(90.50))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(
            position.is_none(),
            "Position should be liquidated at liq price"
        );
    }
    /// Edge case: liquidation occurs exactly at threshold price.
    /// Confirms boundary condition is handled correctly.
    #[test]
    fn test_liquidation_at_exact_threshold() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(90.50))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(
            position.is_none(),
            "Position should be liquidated at exact threshold"
        );
    }
    /// Ensures liquidation when price crosses BELOW threshold.
    /// Confirms strict enforcement (no delayed liquidation).
    #[test]
    fn test_liquidation_below_threshold() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(
            position.is_none(),
            "Position should be liquidated and removed"
        );
    }

    /// Ensures position safety ABOVE threshold:
    /// - Position remains active
    /// - PnL evolves without triggering liquidation
    /// Validates no premature liquidation.
    #[test]
    fn test_safe_position_above_threshold() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(98))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(position.is_some(), "Position should not be liquidated");

        let _pos = position.expect("Position not found");

        engine
            .update_price(dec!(99.5))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);
        assert!(position.is_some(), "Position should still exist");

        let pos = position.expect("Position not found");
        assert!(pos.pnl > dec!(-95));
    }

    /// Prevents double liquidation / double close:
    /// - Once removed, position cannot be closed again
    /// - Engine must return error on invalid action
    #[test]
    fn test_double_liquidation_prevention() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        assert!(engine.get_position(position_id).is_none());

        let result = engine.close_position(position_id);

        assert!(result.is_err(), "Double close should return error");
        assert!(
            result
                .err()
                .unwrap()
                .contains("not found or already closed")
        );
    }
   
    /// Ensures funding-driven liquidation:
    /// - Funding reduces effective margin over time
    /// - Liquidation occurs even without price movement
    #[test]
    fn test_funding_triggered_liquidation() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine.funding_rate = dec!(0.1);

        for _ in 0..11 {
            let result = engine.apply_funding();
            assert!(result.is_ok(), "Funding application should succeed");

            let position = engine.get_position(position_id);
            if position.is_none() {
                break;
            }
        }

        let position = engine.get_position(position_id);
        assert!(
            position.is_none(),
            "Position should be liquidated by accumulated funding"
        );
    }
    /// Tests selective liquidation across multiple positions:
    /// - Only positions breaching threshold are removed
    /// - Others remain intact
    /// Confirms isolation between positions.
    #[test]
    fn test_selective_liquidation_multiple_positions() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let pos1_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position 1");

        let pos2_id = engine
            .open_position("ETH", dec!(500), dec!(5), PositionType::Long)
            .expect("Failed to open position 2");

        let pos3_id = engine
            .open_position("SOL", dec!(200), dec!(8), PositionType::Long)
            .expect("Failed to open position 3");

        assert!(engine.get_position(pos1_id).is_some());
        assert!(engine.get_position(pos2_id).is_some());
        assert!(engine.get_position(pos3_id).is_some());

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        assert!(
            engine.get_position(pos1_id).is_none(),
            "Position 1 should be liquidated"
        );

        assert!(engine.get_position(pos2_id).is_some());
        assert!(engine.get_position(pos3_id).is_some());
    }
    /// Validates liquidation formula across multiple leverages:
    /// - Higher leverage → closer liquidation price
    /// - Formula scales correctly with leverage
    #[test]
    fn test_liquidation_price_different_leverages() {
        let leverages = vec![dec!(1), dec!(5), dec!(10), dec!(50), dec!(100)];
        let entry_price = dec!(100);

        for leverage in leverages {
            let mut engine = Engine::new(10000.0);
            engine.current_price = entry_price;

            let position_id = engine
                .open_position("BTC", dec!(100), leverage, PositionType::Long)
                .expect("Failed to open position");

            let position = engine
                .get_position(position_id)
                .expect("Position not found");

            let expected = entry_price * (dec!(1.0) - (dec!(0.95) / leverage));
            assert_eq!(position.liquidation_price, expected);
        }
    }
}


// Margin Buffer: It uses a 95% loss threshold (likely leaving 5% as a "liquidation fee" or insurance fund contribution).

// Price Updates: Liquidation is checked every time update_price is called.

// Position Lifecycle: A liquidated position is completely removed (is_none()) from the engine's internal storage.