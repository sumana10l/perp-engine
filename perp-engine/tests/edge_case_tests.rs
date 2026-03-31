#[cfg(test)]
mod edge_case_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;

    #[test]
    fn test_price_gap_instant_jump() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        for _ in 0..3 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(80))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);
        assert!(
            position.is_none(),
            "Position should be liquidated on price gap"
        );
    }

    #[test]
    fn test_negative_equity_realization() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(50))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);
        assert!(position.is_none(), "Position should be liquidated");

        assert!(engine.balance < dec!(1000), "Balance should reflect loss");
    }

    #[test]
    fn test_multiple_positions_price_gap() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        for _ in 0..3 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let pos1_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position 1");

        let pos2_id = engine
            .open_position("ETH", dec!(200), dec!(5), PositionType::Long)
            .expect("Failed to open position 2");

        let pos3_id = engine
            .open_position("SOL", dec!(150), dec!(8), PositionType::Short)
            .expect("Failed to open position 3");

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        let pos1 = engine.get_position(pos1_id);
        let pos2 = engine.get_position(pos2_id);
        let pos3 = engine.get_position(pos3_id);

        assert!(pos1.is_none(), "Long position 1 should be liquidated");

        assert!(pos2.is_some(), "Long position 2 should survive");

        assert!(pos3.is_some(), "Short position 3 should survive");
    }

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
        assert!(
            result.is_err(),
            "Should not allow closing already-liquidated position"
        );

        assert!(result.err().unwrap().to_lowercase().contains("not found"));
    }

    #[test]
    fn test_liquidation_during_funding() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        assert!(engine.get_position(position_id).is_some());

        engine.funding_rate = dec!(0.11);

        engine.apply_funding().expect("Failed to apply funding");

        assert!(engine.get_position(position_id).is_none());
    }

    #[test]
    fn test_funding_plus_price_liquidation() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine.funding_rate = dec!(0.03); 
        engine.apply_funding().expect("Failed to apply funding");

        let pos = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(pos.pnl, dec!(-30));

        engine
            .update_price(dec!(97))
            .expect("Failed to update price");

        let pos = engine.get_position(position_id);
        assert!(pos.is_some(), "Position should survive combined effect");
    }

    #[test]
    fn test_extreme_volatility() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(50))
            .expect("Failed to update price");

        let position = engine.get_position(position_id);

        assert!(position.is_none());
    }

    #[test]
    fn test_rapid_sequential_updates() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        for i in 0..100 {
            let price = dec!(100) + (Decimal::from(i % 10) - dec!(5));
            engine.update_price(price).expect("Failed to update price");
        }

        let position = engine.get_position(position_id);
        let _ = position;
    }

    #[test]
    fn test_zero_margin_rejection() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(0), dec!(10), PositionType::Long);

        assert!(result.is_err(), "Should reject zero margin");
    }

    #[test]
    fn test_fractional_margin() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(0.01), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.margin, dec!(0.01));
    }

    #[test]
    fn test_very_high_leverage() {
        let mut engine = Engine::new(100000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(100), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert_eq!(position.leverage, dec!(100));

        engine
            .update_price(dec!(101))
            .expect("Failed to update price");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");
        assert!(position.pnl > dec!(0));
    }

    #[test]
    fn test_liquidation_price_min_leverage() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(1), PositionType::Long)
            .expect("Failed to open position");

        let position = engine
            .get_position(position_id)
            .expect("Position not found");

        assert_eq!(position.liquidation_price, dec!(5));
    }

    #[test]
    fn test_close_at_liquidation_price() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let position_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        engine
            .update_price(dec!(90.50))
            .expect("Failed to update price");

        assert!(engine.get_position(position_id).is_none());
    }

    #[test]
    fn test_rapid_open_close_cycles() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        for i in 0..10 {
            let pos_id = engine
                .open_position(
                    "BTC",
                    dec!(100),
                    dec!(5),
                    if i % 2 == 0 {
                        PositionType::Long
                    } else {
                        PositionType::Short
                    },
                )
                .expect("Failed to open position");

            let result = engine.close_position(pos_id);
            assert!(result.is_ok(), "Should close position successfully");
        }

        let final_pos = engine
            .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
            .expect("Failed to open final position");

        assert!(engine.get_position(final_pos).is_some());
    }

    #[test]
    fn test_liquidation_cleans_up_position() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let pos_id = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .expect("Failed to open position");

        let initial_count = engine.positions.len();
        assert_eq!(initial_count, 1);

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        assert_eq!(engine.positions.len(), 0);
        assert!(engine.get_position(pos_id).is_none());
    }

    #[test]
    fn test_price_gap_with_mark_price() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        for _ in 0..5 {
            engine
                .update_price(dec!(100))
                .expect("Failed to update price");
        }

        let position_id = engine
            .open_position("BTC", dec!(200), dec!(5), PositionType::Long)
            .expect("Failed to open position");

        let result = engine
            .update_price(dec!(85))
            .expect("Failed to update price");

        assert!(result.mark_price > dec!(85), "Mark price should smooth gap");
        assert!(result.mark_price < dec!(100), "But still reflect the move");

        let position = engine.get_position(position_id);
        let _ = position;
    }

    #[test]
    fn test_concurrent_like_operations() {
        let mut engine = Engine::new(10000.0);
        engine.current_price = dec!(100);

        let pos1 = engine
            .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
            .unwrap();
        let pos2 = engine
            .open_position("ETH", dec!(200), dec!(5), PositionType::Long)
            .unwrap();
        let pos3 = engine
            .open_position("SOL", dec!(150), dec!(5), PositionType::Long)
            .unwrap();

        engine
            .update_price(dec!(105))
            .expect("Failed to update price");

        assert!(engine.get_position(pos1).is_some());
        assert!(engine.get_position(pos2).is_some());
        assert!(engine.get_position(pos3).is_some());

        engine.funding_rate = dec!(0.01);
        engine.apply_funding().expect("Failed to apply funding");

        assert!(engine.get_position(pos1).is_some());
        assert!(engine.get_position(pos2).is_some());
        assert!(engine.get_position(pos3).is_some());

        engine.close_position(pos1).expect("Failed to close pos1");
        assert!(engine.get_position(pos1).is_none());
        assert!(engine.get_position(pos2).is_some());
        assert!(engine.get_position(pos3).is_some());
    }

    #[test]
    fn test_state_consistency_after_liquidation() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let pos1 = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .unwrap();
        let pos2 = engine
            .open_position("ETH", dec!(200), dec!(5), PositionType::Long)
            .unwrap();

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");

        assert!(
            engine.get_position(pos1).is_none(),
            "Pos1 should be liquidated"
        );
        assert!(engine.get_position(pos2).is_some(), "Pos2 should survive");

        let pos2_data = engine.get_position(pos2).expect("Pos2 not found");
        assert_eq!(pos2_data.entry_price, dec!(100));
    }

    #[test]
    fn test_recovery_after_liquidation() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let pos1 = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .unwrap();

        engine
            .update_price(dec!(90))
            .expect("Failed to update price");
        assert!(engine.get_position(pos1).is_none());

        engine
            .update_price(dec!(105))
            .expect("Failed to update price");

        let pos2 = engine
            .open_position("BTC", dec!(100), dec!(10), PositionType::Long)
            .unwrap();
        assert!(engine.get_position(pos2).is_some());
    }

    #[test]
    fn test_large_number_of_positions() {
        let mut engine = Engine::new(1000000.0);
        engine.current_price = dec!(100);

        let mut position_ids = Vec::new();

        for i in 0..50 {
            let pos_id = engine
                .open_position(
                    "BTC",
                    dec!(100),
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

        assert_eq!(engine.positions.len(), 50);

        let result_51 = engine.open_position("BTC", dec!(100), dec!(5), PositionType::Long);

        assert!(
            result_51.is_err(),
            "Should reject position beyond max limit"
        );

        engine
            .update_price(dec!(102))
            .expect("Failed to update price");

        for pos_id in position_ids.iter() {
            assert!(engine.get_position(*pos_id).is_some());
        }
    }
}
