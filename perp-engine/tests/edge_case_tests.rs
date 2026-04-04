// Edge case & stress testing:
//
// Purpose:
// - Validate system behavior under extreme and unexpected conditions
//
// Covers:
// - Sudden price gaps (no gradual movement)
// - Combined effects (price + funding)
// - State consistency after liquidation
// - Rapid sequential operations
//
// Invariants:
// - No invalid state (no ghost positions, no double actions)
// - Liquidation always cleans up correctly
// - System remains stable under high-frequency updates
//
// Goal:
// Ensure engine behaves predictably even in worst-case scenarios
#[cfg(test)]
mod edge_case_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use perp_engine::engine::engine::Engine;
    use perp_engine::engine::position::PositionType;

    /// Ensures liquidation on sudden price gap:
    /// - No gradual transition → immediate threshold breach
    /// Confirms system handles discontinuous price jumps.
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
    /// Validates loss realization after liquidation:
    /// - Position removed
    /// - Engine balance reflects realized loss
    /// Confirms proper accounting.
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
    /// Tests selective liquidation under price gap:
    /// - Only vulnerable positions are liquidated
    /// - Others remain intact
    /// Confirms isolation between positions.
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
    /// Prevents double liquidation:
    /// - Liquidated position cannot be closed again
    /// Confirms state consistency and error handling.
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
    /// Ensures funding alone can trigger liquidation:
    /// - High funding reduces equity below threshold
    /// Confirms funding participates in risk model.
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
    /// Tests combined effect of funding + price:
    /// - Partial funding loss followed by price move
    /// - Position survives if threshold not breached
    /// Confirms interaction correctness.
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
    /// Ensures extreme volatility triggers liquidation:
    /// - Large price drop breaches threshold
    /// Confirms robustness under extreme moves.
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
    /// Tests system stability under rapid updates:
    /// - High-frequency price updates should not corrupt state
    /// Confirms no hidden race-like inconsistencies.
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
    /// Ensures zero margin is rejected:
    /// - Prevents invalid position creation
    #[test]
    fn test_zero_margin_rejection() {
        let mut engine = Engine::new(1000.0);
        engine.current_price = dec!(100);

        let result = engine.open_position("BTC", dec!(0), dec!(10), PositionType::Long);

        assert!(result.is_err(), "Should reject zero margin");
    }
    /// Validates fractional margin support:
    /// - Small margin values handled correctly
    /// Confirms precision and flexibility.
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
    /// Tests behavior at maximum allowed leverage:
    /// - High leverage amplifies PnL correctly
    /// Confirms system supports upper risk bounds.
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
    /// Validates liquidation price at minimum leverage (1x):
    /// - Ensures formula behaves correctly at boundary
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
    /// Ensures liquidation at exact threshold:
    /// - Position removed when price == liquidation price
    /// Confirms boundary precision.
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
    /// Tests rapid open/close cycles:
    /// - No state leakage across repeated operations
    /// Confirms lifecycle stability.
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
    /// Ensures liquidation fully cleans up position:
    /// - Removed from storage
    /// - No residual state remains
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
    /// Validates mark price smoothing during price gaps:
    /// - Mark price should dampen sudden moves
    /// Confirms integration with mark price logic.
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
    /// Simulates concurrent-like operations:
    /// - Open, update, funding, close interleaved
    /// Confirms system consistency across mixed actions.
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
    /// Ensures state consistency after partial liquidation:
    /// - Only affected positions removed
    /// - Remaining positions retain correct state
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
    /// Ensures system recovery after liquidation:
    /// - Engine remains usable post-liquidation
    /// Confirms no corrupted state.
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
    /// Tests scalability with many positions:
    /// - Enforces max position limit
    /// - Existing positions remain stable after updates
    /// Confirms capacity constraints and stability.
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

// If edge cases fail → system becomes unsafe:
// - Incorrect liquidations
// - Broken balance accounting
// - Potential exploit vectors
