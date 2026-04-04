#[cfg(test)]
mod position_opening_tests {
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;
    /// Verifies basic LONG position creation:
    /// - All fields correctly initialized
    /// - Entry price = current price
    /// - PnL starts at 0
    #[test]
    fn test_open_long_position_basic() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open long position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.asset, "BTC");
        assert_eq!(position.margin, dec!(100));
        assert_eq!(position.leverage, dec!(10));
        assert_eq!(position.position_type, PositionType::Long);
        assert_eq!(position.entry_price, dec!(100));
        assert_eq!(position.pnl, dec!(0));
    }
    /// Verifies basic SHORT position creation:
    /// - Confirms correct type and initialization
    /// Ensures symmetry with LONG setup.
    #[test]
    fn test_open_short_position_basic() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("ETH", dec!(200), dec!(5), PositionType::Short)
            .expect("Failed to open short position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.asset, "ETH");
        assert_eq!(position.margin, dec!(200));
        assert_eq!(position.leverage, dec!(5));
        assert_eq!(position.position_type, PositionType::Short);
        assert_eq!(position.entry_price, dec!(100));
        assert_eq!(position.pnl, dec!(0));
    }
    /// Ensures position opens with sufficient margin:
    /// - Valid balance → position successfully created
    #[test]
    fn test_open_position_sufficient_margin() {
        let mut engine = Engine::new(5000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(500), dec!(10), PositionType::Long);

        assert!(
            result.is_ok(),
            "Should allow position with sufficient margin"
        );
        let position_id = result.unwrap();
        assert!(engine.get_position(position_id).is_some());
    }
    /// Ensures position is rejected with insufficient margin:
    /// - Prevents over-leveraging beyond account balance
    #[test]
    fn test_open_position_insufficient_margin() {
        let mut engine = Engine::new(500.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(1000), dec!(10), PositionType::Long);

        assert!(
            result.is_err(),
            "Should reject position with insufficient margin"
        );
        assert!(
            result
                .err()
                .unwrap()
                .to_lowercase()
                .contains("insufficient balance")
        );
    }
    /// Validates accepted leverage range:
    /// - Engine allows leverage within defined bounds
    #[test]
    fn test_open_position_valid_leverage() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let valid_leverages = vec![dec!(1), dec!(5), dec!(10), dec!(25), dec!(50), dec!(100)];

        for leverage in valid_leverages {
            let result = engine.open_position("BTC", dec!(100), leverage, PositionType::Long);
            assert!(result.is_ok(), "Should accept leverage: {}", leverage);
        }
    }
    /// Ensures leverage below minimum (1x) is rejected:
    /// - Prevents invalid or undefined exposure
    #[test]
    fn test_open_position_leverage_below_minimum() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(100), dec!(0.5), PositionType::Long);

        assert!(result.is_err(), "Should reject leverage below 1x");
    }
    /// Ensures leverage above maximum (100x) is rejected:
    /// - Enforces system risk limits
    #[test]
    fn test_open_position_leverage_above_maximum() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(100), dec!(150), PositionType::Long);

        assert!(result.is_err(), "Should reject leverage above 100x");
    }
    /// Verifies multiple positions can coexist:
    /// - Each position stored independently
    /// - IDs must be unique
    #[test]
    fn test_open_multiple_positions() {
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

        assert!(engine.get_position(pos1_id).is_some());
        assert!(engine.get_position(pos2_id).is_some());
        assert!(engine.get_position(pos3_id).is_some());

        assert_ne!(pos1_id, pos2_id);
        assert_ne!(pos2_id, pos3_id);
        assert_ne!(pos1_id, pos3_id);
    }
    /// Ensures position IDs are unique:
    /// - No collisions across multiple openings
    /// Confirms identity integrity.
    #[test]
    fn test_position_id_uniqueness() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let mut position_ids = Vec::new();

        for i in 0..10 {
            let pos_id = engine
                .open_position(
                    "BTC",
                    dec!(50),
                    dec!(5),
                    if i % 2 == 0 {
                        PositionType::Long
                    } else {
                        PositionType::Short
                    },
                )
                .expect("Failed to open position");
            position_ids.push(pos_id);
        }

        let unique_count = position_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, 10, "All position IDs should be unique");
    }
    /// Validates initial position state:
    /// - PnL = 0
    /// - Entry price = current price
    /// - Liquidation price correctly computed
    #[test]
    fn test_position_initial_state() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.pnl, dec!(0), "Initial PnL should be 0");
        assert_eq!(
            position.entry_price,
            dec!(100),
            "Entry price should be current price"
        );
        assert!(
            position.liquidation_price > dec!(0),
            "Liquidation price should be calculated"
        );

        assert_eq!(position.liquidation_price, dec!(90.50));
    }
    /// Ensures position metadata is preserved:
    /// - ID, asset, and type remain consistent
    #[test]
    fn test_position_metadata() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert!(position.id == position_id);
        assert_eq!(position.asset, "BTC");
        assert_eq!(position.position_type, PositionType::Long);
    }
    /// Ensures margin is deducted from balance on open:
    /// - Balance reflects locked capital
    #[test]
    fn test_balance_deduction_after_opening() {
        let initial_balance = dec!(1000);
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let margin_used = dec!(300);

        engine
            .open_position("BTC", margin_used, dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let remaining_balance = engine.balance;

        let expected_remaining = initial_balance - margin_used;
        assert_eq!(remaining_balance, expected_remaining);
    }
    /// Validates cumulative margin deduction:
    /// - Multiple positions correctly reduce balance   
    #[test]
    fn test_multiple_margin_deductions() {
        let initial_balance = dec!(1000);
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let margins = vec![dec!(100), dec!(200), dec!(150)];
        let total_margin: rust_decimal::Decimal = margins.iter().sum();

        for margin in margins {
            engine
                .open_position("BTC", margin, dec!(10), PositionType::Long)
                .expect("Failed to open position");
        }

        let remaining_balance = engine.balance;
        let expected_remaining = initial_balance - total_margin;

        assert_eq!(remaining_balance, expected_remaining);
    }
    /// Ensures entry price matches market at open:
    /// - Liquidation price scales with entry price
    /// Confirms formula consistency across prices
    #[test]
    fn test_open_position_at_different_prices() {
        let prices = vec![dec!(50), dec!(100), dec!(500), dec!(1000)];

        for price in prices {
            let mut engine = Engine::new(10000.0);
            engine.current_price = price;

            let position_id = engine
                .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
                .expect("Failed to open position");

            let position = engine
                .get_position(position_id)
                .expect("Position not found");

            assert_eq!(position.entry_price, price);

            let expected_liq_price = price * (dec!(1.0) - (dec!(0.95) / dec!(10)));
            assert_eq!(position.liquidation_price, expected_liq_price);
        }
    }
    /// Ensures asset identifiers are preserved:
    /// - No mutation or mismatch in stored asset names
    #[test]
    fn test_asset_name_preservation() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let assets = vec!["BTC", "ETH", "SOL", "ADA", "DOT"];

        for asset in assets {
            let position_id = engine
                .open_position(asset, dec!(100), dec!(10), PositionType::Long)
                .expect(&format!("Failed to open position for {}", asset));

            let position = engine
                .get_position(position_id)
                .expect("Position not found");
            assert_eq!(position.asset, asset);
        }
    }
    /// Ensures zero margin positions are rejected:
    /// - Prevents invalid position creation
    #[test]
    fn test_open_position_zero_margin() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(0), dec!(10), PositionType::Long);

        assert!(result.is_err(), "Should reject position with zero margin");
    }
    /// Ensures negative margin is rejected:
    /// - Guards against invalid financial state
    #[test]
    fn test_open_position_negative_margin() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(-100), dec!(10), PositionType::Long);

        assert!(
            result.is_err(),
            "Should reject position with negative margin"
        );
    }
    /// Ensures zero leverage is rejected:
    /// - Prevents undefined exposure
    #[test]
    fn test_open_position_zero_leverage() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(100), dec!(0), PositionType::Long);

        assert!(result.is_err(), "Should reject position with zero leverage");
    }
    /// Ensures negative leverage is rejected:
    /// - Guards against invalid risk configuration
    #[test]
    fn test_open_position_negative_leverage() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(100), dec!(-10), PositionType::Long);

        assert!(
            result.is_err(),
            "Should reject position with negative leverage"
        );
    }
    /// Validates decimal precision handling:
    /// - High precision inputs preserved accurately
    /// Confirms no rounding loss during initialization
    #[test]
    fn test_position_decimal_precision() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100.123456789);

        let margin = dec!(123.456789012);
        let leverage = dec!(7.123456789);

        let position_id = engine
            .open_position("BTC", margin, leverage, PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.margin, margin);
        assert_eq!(position.leverage, leverage);
        assert_eq!(position.entry_price, dec!(100.123456789));
    }
}
// If position initialization is incorrect:
// - PnL, funding, and liquidation logic become unreliable
// - System risk model breaks at the foundation
