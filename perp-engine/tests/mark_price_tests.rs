// Mark price model:
//
// Core idea:
// - Mark price = smoothed price derived from recent spot prices
// - Used instead of raw spot to prevent manipulation and unfair liquidations
//
// Mechanism:
// - Maintains rolling buffer (max 10 prices)
// - Mark price = average of recent price history
//
// Design goals:
// - Reduce volatility impact (flash crashes/spikes)
// - Prevent liquidation from short-lived price manipulation
// - Still track real market trend over time
//
// Invariant:
// - mark_price ∈ [min(price_history), max(price_history)]
// - mark_price lags spot but converges over time
#[cfg(test)]
mod mark_price_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;

    /// Ensures initial mark price equals spot price:
    /// - No history → mark = current price
    /// Confirms correct initialization behavior.
    #[test]
    fn test_initial_mark_price_equals_spot() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine
            .update_price(dec!(100))
            .expect("Failed to update price");

        assert_eq!(result.new_price, dec!(100));
        assert_eq!(result.mark_price, dec!(100));
    }
    /// Validates moving average calculation:
    /// - Mark price = average of recent prices
    /// Confirms smoothing logic over price history.
    #[test]
    fn test_moving_average_calculation() {
        let mut engine = Engine::new(1000.0);

        let prices = vec![dec!(100), dec!(101), dec!(102), dec!(103), dec!(104)];

        for price in prices.iter() {
            engine.update_price(*price).expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(104))
            .expect("Failed to update price");

        let expected_mark =
            (dec!(100) + dec!(101) + dec!(102) + dec!(103) + dec!(104) + dec!(104)) / dec!(6);
        assert_eq!(result.mark_price, expected_mark);
    }
    /// Ensures price history buffer is capped at 10 entries:
    /// - Oldest prices are dropped as new ones arrive
    /// Confirms bounded memory + rolling window behavior.
    #[test]
    fn test_price_history_buffer_max_10() {
        let mut engine = Engine::new(1000.0);

        for i in 0..15 {
            let price = dec!(100) + Decimal::from(i);
            engine.update_price(price).expect("Failed to update price");
        }

        assert_eq!(engine.price_history.len(), 10);

        let oldest_in_buffer = engine.price_history.front().unwrap();
        assert_eq!(*oldest_in_buffer, dec!(105));
    }
    /// Ensures mark price lags sudden spot changes:
    /// - Mark reacts slower than spot
    /// Confirms smoothing prevents abrupt jumps.
    #[test]
    fn test_mark_price_lags_behind_spot() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(110))
            .expect("Failed to update price");

        assert_eq!(result.new_price, dec!(110));
        assert!(
            result.mark_price < dec!(110),
            "Mark price should lag spot price"
        );
        assert!(
            result.mark_price > dec!(100),
            "Mark price should increase but not fully"
        );
    }
    /// Validates flash crash protection:
    /// - Sudden drop in spot should not fully reflect in mark price
    /// - Prevents immediate liquidation due to temporary spikes
    /// Confirms safety against extreme short-term volatility.
    #[test]
    fn test_flash_crash_protection() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let position_id = engine
            .open_position("BTC", dec!(500), dec!(5), PositionType::Long)
            .expect("Failed to open position");

        let equity_before_crash = {
            let pos = engine
                .get_position(position_id)
                .expect("Position not found");
            pos.margin + pos.pnl
        };
        assert_eq!(equity_before_crash, dec!(500));

        let result = engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        assert_eq!(result.new_price, dec!(90));
        assert!(
            result.mark_price > dec!(90),
            "Mark price should be smoothed above spot"
        );

        let position = engine.get_position(position_id);

        if let Some(pos) = position {
            let equity_after = pos.margin + pos.pnl;
            assert!(
                equity_after > dec!(0),
                "Position should survive flash crash with mark price smoothing"
            );
        } else {
            panic!("Position liquidated even with mark price smoothing - crash too severe");
        }
    }
    /// Ensures gradual price changes are tracked closely:
    /// - Mark follows trend with limited lag
    /// Confirms balance between responsiveness and smoothing.
    #[test]
    fn test_gradual_price_change_tracks_quickly() {
        let mut engine = Engine::new(1000.0);

        for i in 0..5 {
            engine
                .update_price(dec!(100) + Decimal::from(i))
                .expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(104))
            .expect("Failed to update price");

        let expected_mark =
            (dec!(100) + dec!(101) + dec!(102) + dec!(103) + dec!(104) + dec!(104)) / dec!(6);
        assert_eq!(result.mark_price, expected_mark);

        let diff = result.new_price - result.mark_price;
        assert!(diff < dec!(2), "Gradual changes should have reasonable lag");
        assert!(diff > dec!(1), "Mark price should lag behind spot price");
    }
    /// Ensures buffer fills incrementally:
    /// - Mark calculation adapts as history builds
    /// Confirms correct behavior before buffer saturation.
    #[test]
    fn test_buffer_fills_gradually() {
        let mut engine = Engine::new(1000.0);

        for i in 0..3 {
            let price = dec!(100) + Decimal::from(i);
            engine.update_price(price).expect("Failed to update price");
        }

        assert_eq!(engine.price_history.len(), 3);

        for i in 3..10 {
            let price = dec!(100) + Decimal::from(i);
            engine.update_price(price).expect("Failed to update price");
        }

        assert_eq!(engine.price_history.len(), 10);
    }
    /// Validates behavior under extreme volatility:
    /// - Mark smooths both crash and recovery
    /// Confirms stability across large oscillations.
    #[test]
    fn test_mark_price_extreme_volatility() {
        let mut engine = Engine::new(1000.0);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let crash_result = engine
            .update_price(dec!(50))
            .expect("Failed to update price");

        assert!(
            crash_result.mark_price > dec!(50),
            "Mark price should smooth extreme moves"
        );
        assert!(
            crash_result.mark_price < dec!(100),
            "Mark price should still reflect the move"
        );

        let recovery_result = engine
            .update_price(dec!(100))
            .expect("Failed to update price");

        assert!(
            recovery_result.mark_price > crash_result.mark_price,
            "Mark price should increase on recovery"
        );
    }
    /// Ensures mark price respects historical bounds:
    /// - Mark must lie within observed price range
    /// Confirms no artificial distortion.
    #[test]
    fn test_mark_price_respects_history() {
        let mut engine = Engine::new(1000.0);

        let prices = vec![dec!(100), dec!(101), dec!(102), dec!(103), dec!(104)];

        for price in prices {
            engine.update_price(price).expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(104))
            .expect("Failed to update price");

        let min_in_buffer = engine.price_history.iter().min().unwrap();

        assert!(result.mark_price >= *min_in_buffer);
    }
    /// Ensures consistent directional updates:
    /// - Sustained upward movement → mark increases gradually
    /// Confirms trend-following behavior.
    #[test]
    fn test_multiple_updates_same_direction() {
        let mut engine = Engine::new(1000.0);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        for i in 1..=5 {
            let price = dec!(100) + Decimal::from(i);
            engine.update_price(price).expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(105))
            .expect("Failed to update price");

        assert!(
            result.mark_price > dec!(100),
            "Mark price should increase with prices"
        );
        assert!(
            result.mark_price < dec!(106),
            "Mark price should not exceed highest recent price significantly"
        );
    }
    /// Edge case: single price entry:
    /// - Mark equals spot when only one data point exists
    /// Confirms base case correctness.
    #[test]
    fn test_mark_price_single_entry() {
        let mut engine = Engine::new(1000.0);

        let result = engine
            .update_price(dec!(100))
            .expect("Failed to update price");

        assert_eq!(result.mark_price, dec!(100));
    }
    /// Ensures decimal precision is preserved:
    /// - High precision inputs produce accurate mark values
    /// Guards against rounding errors in averaging.
    #[test]
    fn test_mark_price_decimal_precision() {
        let mut engine = Engine::new(1000.0);

        let prices = vec![
            dec!(100.123456),
            dec!(100.234567),
            dec!(100.345678),
            dec!(100.456789),
            dec!(100.567890),
        ];

        for price in prices.iter() {
            engine.update_price(*price).expect("Failed to update price");
        }

        let result = engine
            .update_price(dec!(100.567890))
            .expect("Failed to update price");

        let expected = (dec!(100.123456)
            + dec!(100.234567)
            + dec!(100.345678)
            + dec!(100.456789)
            + dec!(100.567890)
            + dec!(100.567890))
            / dec!(6);
        assert_eq!(result.mark_price, expected);
    }
    /// Ensures mark price prevents false liquidation:
    /// - Temporary dips should not trigger liquidation
    /// Confirms protective role of mark price in risk system.
    #[test]
    fn test_mark_price_prevents_false_liquidations() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(95))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(position.is_some(), "Position should survive temporary dip");
    }
    /// Ensures mark price convergence:
    /// - Sustained price → mark approaches spot over time
    /// Confirms long-term alignment with market.
    #[test]
    fn test_mark_price_convergence() {
        let mut engine = Engine::new(1000.0);

        engine
            .update_price(dec!(110))
            .expect("Failed to update price");

        for i in 0..9 {
            let result = engine
                .update_price(dec!(110))
                .expect("Failed to update price");

            assert_eq!(result.new_price, dec!(110));

            if i > 0 {
                assert!(
                    result.mark_price >= dec!(105),
                    "Mark should be converging to 110"
                );
            }
        }

        let final_result = engine
            .update_price(dec!(110))
            .expect("Failed to update price");
        assert!(
            final_result.mark_price >= dec!(108),
            "Mark price should converge after sustained price"
        );
    }
    /// Ensures oldest prices are evicted correctly:
    /// - Rolling window removes stale data
    /// Confirms FIFO behavior of price buffer.
    #[test]
    fn test_old_prices_removed_from_buffer() {
        let mut engine = Engine::new(1000.0);

        for i in 0..10 {
            engine
                .update_price(dec!(100) + Decimal::from(i))
                .expect("Failed to update price");
        }

        assert_eq!(engine.price_history.len(), 10);
        assert_eq!(*engine.price_history.front().unwrap(), dec!(100));

        engine
            .update_price(dec!(110))
            .expect("Failed to update price");

        assert_eq!(engine.price_history.len(), 10);
        assert_eq!(
            *engine.price_history.front().unwrap(),
            dec!(101),
            "Oldest price should be 101 now"
        );
    }
}

// If mark price fails → system becomes exploitable:
// - Traders can trigger forced liquidations via price manipulation
// - Leads to unfair liquidations and potential bad debt
