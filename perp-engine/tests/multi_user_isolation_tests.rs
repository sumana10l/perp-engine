// Multi-user isolation testing:
//
// Purpose:
// - Validate that each user's data is completely isolated
// - Ensure operations on one user don't affect another
// - Confirm per-user account separation works correctly
//
// Invariants:
// - User A's balance independent of User B's balance
// - User A's positions invisible to User B
// - User A's trades don't appear in User B's history
// - User A's liquidation doesn't trigger User B's liquidation
//
// Goal:
// Ensure multi-user engine provides complete isolation between accounts

#[cfg(test)]
mod multi_user_isolation_tests {
    use perp_engine::engine::multi_user_engine::MultiUserEngine;
    use perp_engine::engine::position::PositionType;
    use rust_decimal_macros::dec;

    #[test]
    fn test_multi_user_balance_isolation() {
        let mut multi_engine = MultiUserEngine::new();

        let user_a_engine = multi_engine.get_or_create_user("user_a", 1000.0);
        user_a_engine.engine.current_price = dec!(100);

        let user_b_engine = multi_engine.get_or_create_user("user_b", 5000.0);
        user_b_engine.engine.current_price = dec!(100);

        assert_eq!(
            multi_engine.get_user("user_a").unwrap().engine.balance,
            dec!(1000)
        );
        assert_eq!(
            multi_engine.get_user("user_b").unwrap().engine.balance,
            dec!(5000)
        );

        {
            let user_a = multi_engine.get_user_mut("user_a").unwrap();
            user_a
                .engine
                .open_position("BTC", dec!(200), dec!(5), PositionType::Long)
                .expect("Failed to open position for user A");
        }

        let user_a_balance = multi_engine.get_user("user_a").unwrap().engine.balance;
        assert_eq!(user_a_balance, dec!(800), "User A balance should be 800");

        let user_b_balance = multi_engine.get_user("user_b").unwrap().engine.balance;
        assert_eq!(
            user_b_balance,
            dec!(5000),
            "User B balance should remain 5000 (unaffected by User A)"
        );
    }

    #[test]
    fn test_multi_user_position_isolation() {
        let mut multi_engine = MultiUserEngine::new();

        let user_a_engine = multi_engine.get_or_create_user("user_a", 5000.0);
        user_a_engine.engine.current_price = dec!(100);

        let user_b_engine = multi_engine.get_or_create_user("user_b", 5000.0);
        user_b_engine.engine.current_price = dec!(100);

        assert_eq!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .positions
                .len(),
            0
        );
        assert_eq!(
            multi_engine
                .get_user("user_b")
                .unwrap()
                .engine
                .positions
                .len(),
            0
        );

        let user_a_pos1_id = {
            let user_a = multi_engine.get_user_mut("user_a").unwrap();
            user_a
                .engine
                .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
                .expect("Failed to open position 1 for user A")
        };

        let user_a_pos2_id = {
            let user_a = multi_engine.get_user_mut("user_a").unwrap();
            user_a
                .engine
                .open_position("ETH", dec!(150), dec!(5), PositionType::Short)
                .expect("Failed to open position 2 for user A")
        };

        assert_eq!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .positions
                .len(),
            2,
            "User A should have 2 positions"
        );

        assert_eq!(
            multi_engine
                .get_user("user_b")
                .unwrap()
                .engine
                .positions
                .len(),
            0,
            "User B should see 0 positions (User A's positions are hidden)"
        );

        let user_b_pos1_id = {
            let user_b = multi_engine.get_user_mut("user_b").unwrap();
            user_b
                .engine
                .open_position("SOL", dec!(200), dec!(3), PositionType::Long)
                .expect("Failed to open position for user B")
        };

        assert_eq!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .positions
                .len(),
            2,
            "User A should still have 2 positions"
        );
        assert_eq!(
            multi_engine
                .get_user("user_b")
                .unwrap()
                .engine
                .positions
                .len(),
            1,
            "User B should have 1 position"
        );

        assert!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .get_position(user_a_pos1_id)
                .is_some()
        );
        assert!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .get_position(user_a_pos2_id)
                .is_some()
        );

        assert!(
            multi_engine
                .get_user("user_b")
                .unwrap()
                .engine
                .get_position(user_b_pos1_id)
                .is_some()
        );

        assert!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .get_position(user_b_pos1_id)
                .is_none(),
            "User A should not see User B's position"
        );

        assert!(
            multi_engine
                .get_user("user_b")
                .unwrap()
                .engine
                .get_position(user_a_pos1_id)
                .is_none(),
            "User B should not see User A's positions"
        );
    }

    #[test]
    fn test_multi_user_price_update_isolation() {
        let mut multi_engine = MultiUserEngine::new();

        let user_a_engine = multi_engine.get_or_create_user("user_a", 10000.0);
        user_a_engine.engine.current_price = dec!(100);

        let user_b_engine = multi_engine.get_or_create_user("user_b", 10000.0);
        user_b_engine.engine.current_price = dec!(100);

        let user_a_pos_id = {
            let user_a = multi_engine.get_user_mut("user_a").unwrap();
            user_a
                .engine
                .open_position("BTC", dec!(500), dec!(10), PositionType::Long)
                .expect("Failed to open position for user A")
        };

        let user_b_pos_id = {
            let user_b = multi_engine.get_user_mut("user_b").unwrap();
            user_b
                .engine
                .open_position("BTC", dec!(500), dec!(10), PositionType::Short)
                .expect("Failed to open position for user B")
        };

        {
            let user_a = multi_engine.get_user("user_a").unwrap();
            let user_b = multi_engine.get_user("user_b").unwrap();

            let user_a_pnl = user_a.engine.get_position(user_a_pos_id).unwrap().pnl;
            let user_b_pnl = user_b.engine.get_position(user_b_pos_id).unwrap().pnl;

            assert_eq!(user_a_pnl, dec!(0), "User A PnL should be 0 at entry");
            assert_eq!(user_b_pnl, dec!(0), "User B PnL should be 0 at entry");
        }

        for price in [dec!(99), dec!(98), dec!(97), dec!(96), dec!(95)].iter() {
            {
                let user_a = multi_engine.get_user_mut("user_a").unwrap();
                user_a
                    .engine
                    .update_price(*price)
                    .expect("Failed to update price for user A");
            }

            {
                let user_b = multi_engine.get_user_mut("user_b").unwrap();
                user_b
                    .engine
                    .update_price(*price)
                    .expect("Failed to update price for user B");
            }
        }

        let user_a_price = multi_engine
            .get_user("user_a")
            .unwrap()
            .engine
            .current_price;
        let user_b_price = multi_engine
            .get_user("user_b")
            .unwrap()
            .engine
            .current_price;

        assert_eq!(user_a_price, dec!(95), "Both users see same current price");
        assert_eq!(user_b_price, dec!(95), "Both users see same current price");

        let user_a = multi_engine.get_user("user_a").unwrap();
        let user_a_pos = user_a.engine.get_position(user_a_pos_id);

        if let Some(pos) = user_a_pos {
            assert!(pos.pnl < dec!(0), "User A long should have negative PnL");
        }

        let user_b = multi_engine.get_user("user_b").unwrap();
        let user_b_pos = user_b.engine.get_position(user_b_pos_id);

        if let Some(pos) = user_b_pos {
            assert!(pos.pnl > dec!(0), "User B short should have positive PnL");
        }

        let user_a = multi_engine.get_user("user_a").unwrap();
        let user_b = multi_engine.get_user("user_b").unwrap();

        let user_a_pnl = user_a
            .engine
            .get_position(user_a_pos_id)
            .map(|p| p.pnl)
            .unwrap_or(dec!(0));
        let user_b_pnl = user_b
            .engine
            .get_position(user_b_pos_id)
            .map(|p| p.pnl)
            .unwrap_or(dec!(0));

        assert_eq!(
            user_a_pnl + user_b_pnl,
            dec!(0),
            "Symmetric long/short PnLs should sum to 0"
        );

        let user_a_balance = multi_engine.get_user("user_a").unwrap().engine.balance;
        let user_b_balance = multi_engine.get_user("user_b").unwrap().engine.balance;

        assert_eq!(
            user_a_balance,
            dec!(9500),
            "User A balance should be 9500 (1000 locked + 500 margin)"
        );
        assert_eq!(
            user_b_balance,
            dec!(9500),
            "User B balance should be 9500 (1000 locked + 500 margin)"
        );

        let user_a_trades = &multi_engine
            .get_user("user_a")
            .unwrap()
            .engine
            .trade_history;
        let user_b_trades = &multi_engine
            .get_user("user_b")
            .unwrap()
            .engine
            .trade_history;

        assert_eq!(
            user_a_trades.len(),
            0,
            "User A trade history should be empty (position still open)"
        );
        assert_eq!(
            user_b_trades.len(),
            0,
            "User B trade history should be empty (position still open)"
        );
    }

    #[test]
    fn test_multi_user_position_modification_isolation() {
        let mut multi_engine = MultiUserEngine::new();

        let user_a_engine = multi_engine.get_or_create_user("user_a", 5000.0);
        user_a_engine.engine.current_price = dec!(100);

        let user_b_engine = multi_engine.get_or_create_user("user_b", 5000.0);
        user_b_engine.engine.current_price = dec!(100);

        let user_a_pos_id = {
            let user_a = multi_engine.get_user_mut("user_a").unwrap();
            user_a
                .engine
                .open_position("BTC", dec!(100), dec!(5), PositionType::Long)
                .expect("Failed to open position for user A")
        };

        {
            let user_b = multi_engine.get_user_mut("user_b").unwrap();
            let result = user_b.engine.close_position(user_a_pos_id);

            assert!(
                result.is_err(),
                "User B should not be able to close User A's position"
            );
        }

        assert!(
            multi_engine
                .get_user("user_a")
                .unwrap()
                .engine
                .get_position(user_a_pos_id)
                .is_some(),
            "User A's position should still exist after User B's failed close attempt"
        );
    }
}
