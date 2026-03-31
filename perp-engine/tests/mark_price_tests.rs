#[cfg(test)]
mod mark_price_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;

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

    #[test]
    fn test_mark_price_single_entry() {
        let mut engine = Engine::new(1000.0);

        let result = engine
            .update_price(dec!(100))
            .expect("Failed to update price");

        assert_eq!(result.mark_price, dec!(100));
    }

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
