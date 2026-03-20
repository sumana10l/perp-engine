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

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/position/open` | Open new position |
| GET | `/positions` | List all positions |
| POST | `/position/close` | Close position |
| GET | `/price` | Current price & mark price |
| GET | `/balance` | Balance & total equity |
| GET | `/funding-rate` | Funding rate info |
| GET | `/trade-history` | Closed trades |
| GET | `/health` | Health check |

---

## Quick Start
```bash
# Start engine
cargo run

# Server runs on http://localhost:8080
```

## Testing
```bash
# Run automated test suite
chmod +x test_engine.sh
./test_engine.sh
```

Tests verify: position creation, price updates, PnL calculations, liquidations, and position closure.

---

## Project Structure
```
src/
├─ engine/          # Core trading logic
│  ├─ engine.rs     # Main state machine
│  ├─ position.rs   # Position data models
│  ├─ trade.rs      # Trade records
│  └─ event.rs      # Event types
├─ api/
│  ├─ position.rs   # Actix-web handlers with RwLock async access
│  └─ mod.rs        # Module exports
├─ market/
│  ├─ ws.rs         # Resilient Binance WebSocket with exponential backoff
│  └─ mod.rs        # Module exports
└─ main.rs          # Server entry point
```

---
