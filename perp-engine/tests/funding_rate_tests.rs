// Funding model:
//
// Core idea:
// - Funding transfers value between longs and shorts based on funding_rate
// - Positive funding → longs pay shorts
// - Negative funding → shorts pay longs
//
// Formula:
// funding_payment = notional * funding_rate
//
// Effects:
// - Directly modifies PnL (not margin)
// - Impacts equity → can trigger liquidation
//
// Invariant:
// - Funding is zero-sum across positions (one side pays, other receives)
#[cfg(test)]
mod funding_rate_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;

    /// Verifies basic funding impact on LONG:
    /// - Positive funding → LONG pays → PnL decreases
    /// Confirms correct sign and magnitude of funding deduction.

    #[test]
    fn test_funding_long_position_basic() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position_before = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position_before.pnl, dec!(0));

        engine.funding_rate = dec!(0.01);

        engine.apply_funding().expect("Failed to apply funding");

        let position_after = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position_after.pnl, dec!(-10));
    }

    /// Verifies basic funding impact on SHORT:
    /// - Positive funding → SHORT receives → PnL increases
    /// Confirms correct directional transfer.
    #[test]
    fn test_funding_short_position_basic() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open position");

        let position_before = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position_before.pnl, dec!(0));

        engine.funding_rate = dec!(0.01);

        engine.apply_funding().expect("Failed to apply funding");

        let position_after = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position_after.pnl, dec!(10));
    }

    /// Ensures funding reduces LONG equity:
    /// - Equity = margin + pnl
    /// - Funding loss directly reduces total equity
    /// Confirms risk increases even without price movement.
    #[test]
    fn test_funding_reduces_long_equity() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let equity_before = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.margin + pos.pnl
        };
        assert_eq!(equity_before, dec!(100));

        engine.funding_rate = dec!(0.01);
        engine.apply_funding().expect("Failed to apply funding");

        let equity_after = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.margin + pos.pnl
        };

        assert_eq!(equity_after, dec!(90));
    }

    /// Ensures funding increases SHORT equity:
    /// - Funding gain increases total position equity
    /// Validates asymmetry between LONG and SHORT.
    #[test]
    fn test_funding_increases_short_equity() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open position");

        let equity_before = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.margin + pos.pnl
        };
        assert_eq!(equity_before, dec!(100));

        engine.funding_rate = dec!(0.01);
        engine.apply_funding().expect("Failed to apply funding");

        let equity_after = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.margin + pos.pnl
        };

        assert_eq!(equity_after, dec!(110));
    }
    /// Validates funding calculation across multiple rates:
    /// - funding = notional * rate
    /// Ensures linear scaling with funding_rate.
    #[test]
    fn test_funding_with_different_rates() {
        let rates = vec![dec!(0.001), dec!(0.01), dec!(0.02), dec!(0.03)];

        for rate in rates {
            let mut engine = Engine::new(10000.0);
            engine.current_price = dec!(100);

            let position_id = engine
                .open_position("BTC", dec!(500), dec!(5), PositionType::Long)
                .expect("Failed to open position");

            engine.funding_rate = rate;
            engine.apply_funding().expect("Failed to apply funding");

            let position = engine
                .get_position(position_id)
                .expect("Position not found");

            let expected_funding = dec!(2500) * rate;
            let expected_pnl = dec!(0) - expected_funding;

            assert_eq!(position.pnl, expected_pnl);
        }
    }
    /// Validates funding scales with leverage:
    /// - Higher leverage → larger notional → higher funding impact
    /// Confirms leverage amplifies funding cost.
    #[test]
    fn test_funding_with_different_leverage() {
        let leverages = vec![dec!(1), dec!(5), dec!(10), dec!(50)];

        for leverage in leverages {
            let mut engine = Engine::new(10000.0);
            engine.current_price = dec!(100);

            let position_id = engine
                .open_position("BTC", dec!(100), leverage, PositionType::Long)
                .expect("Failed to open position");

            engine.funding_rate = dec!(0.01);

            engine.apply_funding().expect("Failed to apply funding");

            let position = engine
                .get_position(position_id)
                .expect("Position not found");

            let notional = leverage * dec!(100);
            let expected_funding = notional * dec!(0.01);
            let expected_pnl = dec!(0) - expected_funding;

            assert_eq!(position.pnl, expected_pnl);
        }
    }
    /// Ensures funding accumulates over time:
    /// - Repeated funding applications compound linearly
    /// Confirms no reset or overwrite of PnL.
    #[test]
    fn test_funding_multiple_periods_cumulative() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine.funding_rate = dec!(0.01);

        for period in 1..=5 {
            engine
                .apply_funding()
                .expect(&format!("Failed to apply funding in period {}", period));

            let position = engine
                .get_position(position_id)
                .expect("Position not found");

            let expected_pnl = dec!(-10) * Decimal::from(period);
            assert_eq!(position.pnl, expected_pnl);
        }
    }
    /// Ensures extreme funding can trigger liquidation:
    /// - Funding alone reduces equity below threshold
    /// Confirms funding is part of liquidation pathway.
    #[test]
    fn test_funding_triggers_liquidation_long() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine.funding_rate = dec!(1.0);

        engine.apply_funding().expect("Failed to apply funding");

        let position = engine.get_position(position_id);

        assert!(
            position.is_none(),
            "Position should be liquidated by funding"
        );
    }
    /// Ensures SHORT is not liquidated under positive funding:
    /// - SHORT benefits → equity increases
    /// Confirms no false liquidation.
    #[test]
    fn test_funding_does_not_liquidate_short() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open position");

        engine.funding_rate = dec!(1.0);

        engine.apply_funding().expect("Failed to apply funding");

        let position = engine.get_position(position_id);

        assert!(
            position.is_some(),
            "Short position should not be liquidated by positive funding"
        );
    }
    /// Validates funding across multiple positions:
    /// - Each position updated independently
    /// - LONG loses, SHORT gains
    /// Confirms isolation + correct distribution.
    #[test]
    fn test_funding_on_multiple_positions() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let pos1_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position 1");

        let pos2_id = engine
            .open_position("ETH", dec!(200), dec!(5), PositionType::Short)
            .expect("Failed to open position 2");

        let pos3_id = engine
            .open_position("SOL", dec!(150), dec!(8), PositionType::Long)
            .expect("Failed to open position 3");

        engine.funding_rate = dec!(0.01); // 1%

        engine.apply_funding().expect("Failed to apply funding");

        let pos1 = engine.get_position(pos1_id).expect("Position 1 not found");
        let pos2 = engine.get_position(pos2_id).expect("Position 2 not found");
        let pos3 = engine.get_position(pos3_id).expect("Position 3 not found");

        assert_eq!(pos1.pnl, dec!(-10));

        assert_eq!(pos2.pnl, dec!(10));

        assert_eq!(pos3.pnl, dec!(-12));
    }
    /// Ensures funding applies on top of existing PnL:
    /// - Funding adjusts current PnL, not resets it
    /// Confirms additive behavior.
    #[test]
    fn test_funding_with_existing_pnl() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(105))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.pnl, dec!(50));

        engine.funding_rate = dec!(0.01);

        engine.apply_funding().expect("Failed to apply funding");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.pnl, dec!(39.50));
    }
    /// Edge case: zero funding rate:
    /// - No PnL change should occur
    /// Confirms no-op behavior.
    #[test]
    fn test_zero_funding_rate() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let pnl_before = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.pnl
        };

        engine.funding_rate = dec!(0);

        engine.apply_funding().expect("Failed to apply funding");

        let pnl_after = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.pnl
        };

        assert_eq!(pnl_before, pnl_after);
    }
    /// Validates negative funding:
    /// - LONG receives, SHORT pays
    /// Confirms sign inversion logic.
    #[test]
    fn test_negative_funding_rate() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let long_pos_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open long position");

        let short_pos_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open short position");

        engine.funding_rate = dec!(-0.01);

        engine.apply_funding().expect("Failed to apply funding");

        let long_pos = engine
            .get_position(long_pos_id)
            .expect("Long position not found");
        let short_pos = engine
            .get_position(short_pos_id)
            .expect("Short position not found");

        assert!(
            long_pos.pnl > dec!(0),
            "Long should benefit from negative funding"
        );

        assert!(short_pos.pnl < dec!(0), "Short should pay negative funding");
    }
    /// Ensures decimal precision is handled correctly:
    /// - Non-integer values produce stable, non-zero PnL
    /// Guards against rounding errors.
    #[test]
    fn test_funding_decimal_precision() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100.123456);

        let position_id = engine
            .open_position("BTC", dec!(123.456789), dec!(7.654321), PositionType::Long)
            .expect("Failed to open position");

        engine.funding_rate = dec!(0.00123456);

        engine.apply_funding().expect("Failed to apply funding");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert!(position.pnl != dec!(0), "PnL should be non-zero");
        assert!(
            position.pnl < dec!(0),
            "Long should have negative PnL from funding"
        );
    }
    /// Validates zero-sum funding transfer:
    /// - LONG loss == SHORT gain
    /// Confirms conservation of value in system.
    #[test]
    fn test_funding_payment_transfer() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let long_pos_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open long position");

        let short_pos_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Short)
            .expect("Failed to open short position");

        engine.funding_rate = dec!(0.01);

        engine.apply_funding().expect("Failed to apply funding");

        let long_pos = engine
            .get_position(long_pos_id)
            .expect("Long position not found");
        let short_pos = engine
            .get_position(short_pos_id)
            .expect("Short position not found");

        assert_eq!(long_pos.pnl, dec!(-10));

        assert_eq!(short_pos.pnl, dec!(10));

        assert_eq!(long_pos.pnl, -short_pos.pnl);
    }
}
