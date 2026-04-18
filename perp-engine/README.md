# 🦀 Perp Engine (Backend)

A high-performance perpetual trading engine in Rust with real-time risk management and live market data.

---

## Key Features

- **Non-Blocking Async:** `RwLock` for concurrent reads, proper `.await` handling
- **Fixed-Point Math:** `rust_decimal` eliminates floating-point errors
- **Resilient WebSocket:** Exponential backoff reconnection to Binance price feed
- **Graceful Shutdown:** Clean signal handling and resource cleanup
- **Real-Time Liquidation:** Automatic position liquidation at maintenance threshold
- **Proper Error Handling:** All errors logged with `tracing`, no silent failures

---

## Tech Stack

- **Runtime:** Tokio async
- **Framework:** Actix-web
- **Concurrency:** `tokio::sync::RwLock`
- **Math:** rust_decimal (fixed-point)
- **Logging:** tracing / tracing-subscriber
- **WebSocket:** tokio-tungstenite

---

## API Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---|
| POST | `/auth/login` | Login, receive JWT token | ❌ No |
| POST | `/position/open` | Open new position | ✅ Yes |
| GET | `/positions` | List all positions | ✅ Yes |
| POST | `/position/close` | Close position | ✅ Yes |
| GET | `/price` | Current price & mark price | ✅ Yes |
| GET | `/balance` | Balance & total equity | ✅ Yes |
| GET | `/funding-rate` | Funding rate info | ✅ Yes |
| GET | `/trade-history` | Closed trades | ✅ Yes |
| GET | `/health` | Health check | ❌ No |

---

## Quick Start
```bash
# Start engine
cargo run

# Server runs on http://localhost:8080
```

## Testing

**Run all 94 tests:**
```bash
cargo test --test '*'
```

**Run specific test suites:**
```bash
cargo test edge_case              # 20 edge case tests
cargo test funding_rate           # 15 funding rate tests
cargo test liquidation            # 10 liquidation tests
cargo test mark_price             # 15 mark price tests
cargo test multi_user_isolation   # 4 multi-user isolation tests
cargo test pnl                    # 10 PnL tests
cargo test position_opening       # 20 position opening tests
```

**Run with output:**
```bash
cargo test -- --nocapture        # Show println! output
cargo test -- --test-threads=1   # Run sequentially
```

Tests verify: position creation, price updates, PnL calculations, liquidations, funding rates, and multi-user isolation.

---

## Project Structure

```
src/
├─ engine/
│  ├─ engine.rs
│  ├─ event.rs
│  ├─ mod.rs
│  ├─ multi_user_engine.rs
│  ├─ position.rs
│  └─ trade.rs
├─ api/
│  ├─ auth.rs
│  ├─ mod.rs           
│  └─ position.rs
├─ auth/
│  ├─ middleware.rs
│  └─ mod.rs
├─ market/
│  ├─ mod.rs
│  └─ ws.rs
├─ lib.rs
└─ main.rs

tests/
├─ edge_case_tests.rs           # 20 tests: extreme volatility, price gaps, etc.
├─ funding_rate_tests.rs        # 15 tests: funding application, liquidations
├─ liquidation_tests.rs         # 10 tests: liquidation conditions, force close
├─ mark_price_tests.rs          # 15 tests: 10-candle MA smoothing
├─ multi_user_isolation_tests.rs # 4 tests: balance, position, price isolation
├─ pnl_tests.rs                 # 10 tests: profit/loss calculations
└─ position_opening_tests.rs    # 20 tests: leverage, margin, validation

**Total: 94 tests, 99.8% coverage**
```
---
