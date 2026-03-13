```markdown
# 🦀 Perp Engine (Backend)

A high-performance, event-driven perpetual trading engine written in **Rust**. This system simulates the core mechanics of a perpetual derivatives exchange, handling real-time risk management, position state transitions, and live market data synchronization.

---

## 🏗 Key Engineering Highlights

* **Fixed-Point Precision:** Implements `rust_decimal` across the entire stack. This eliminates binary floating-point rounding errors (common in `f64`), ensuring financial integrity for balance and PnL calculations.
* **Event-Driven Concurrency:** Utilizes `tokio` MPSC channels to decouple the high-frequency market data feed from the core trading logic.
* **Panic-Resilient Architecture:** * **Safe Mutexes:** Uses pattern matching on `data.lock()` to handle potential Mutex poisoning gracefully without crashing the API thread.
    * **Resilient WebSockets:** Features a non-blocking reconnection loop that automatically recovers the Binance price feed during network interruptions.
* **Real-Time Risk Monitoring:** The engine evaluates every incoming price tick to trigger automated liquidations and update unrealized PnL.

---

## 📐 Engine Logic & Math

The engine enforces strict financial rules to maintain exchange solvency:

### 1. Liquidation Logic
Liquidations occur automatically when the mark price crosses the threshold where the user's collateral (margin) would be exhausted based on their leverage.

* **Long:** $Price_{liq} = Price_{entry} \times (1 - \frac{1}{Leverage})$
* **Short:** $Price_{liq} = Price_{entry} \times (1 + \frac{1}{Leverage})$

### 2. PnL Calculation
* **Long:** $PnL = (Price_{current} - Price_{entry}) \times Quantity$
* **Short:** $PnL = (Price_{entry} - Price_{current}) \times Quantity$

---

## 🛠 Tech Stack

| Component | Technology |
| :--- | :--- |
| **Language** | Rust (Edition 2021) |
| **Async Runtime** | `tokio` |
| **Web Framework** | `actix-web` |
| **Numeric Type** | `rust_decimal` (Fixed-point arithmetic) |
| **Serialization** | `serde` / `serde_json` |
| **WebSocket** | `tokio-tungstenite` |

---

## 🚦 API Reference

### Positions
* `POST /position/open` - Opens a new Long/Short position. Validates margin against current balance.
* `GET /positions` - Returns a map of all active positions.
* `POST /position/close` - Realizes PnL, settles the balance, and moves the position to trade history.

### Market & Wallet
* `GET /price` - Returns the current live Index Price (SOL/USDT).
* `GET /balance` - Returns the user's available collateral balance.
* `GET /trade-history` - List of all settled (closed/liquidated) trades.

---

## 📂 Project Structure

```text
src/
 ├ engine/
 │   ├ engine.rs     # Core state machine & liquidation logic
 │   ├ position.rs   # Position data models (Decimal-based)
 │   ├ trade.rs      # Trade history models
 │   └ event.rs      # MPSC event definitions
 ├ api/
 │   └ position.rs   # Actix-web route handlers & safe locking
 ├ market/
 │   └ ws.rs         # Resilient Binance WebSocket client
 └ main.rs           # App entry point & background worker loop

```

---

## 🧪 Testing the Engine

To verify the full trade lifecycle (Open -> Price Move -> PnL Check -> Close):

1. **Start the engine:**
```bash
cargo run

```


2. **Execute the automated test suite:**
```bash
chmod +x test_engine.sh
./test_engine.sh

```