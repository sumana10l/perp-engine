# Perp Engine (Rust)

A minimal event-driven perpetual trading engine written in Rust that
simulates the core mechanics of a perpetual derivatives exchange.

---

# Features

- Open leveraged positions
- Long / Short trading
- Real-time PnL updates
- Position liquidation price calculation
- Close positions
- Trade history tracking
- Event-driven trading engine
- Live price feed from Binance WebSocket
- REST API using Actix Web
- In-memory engine state

---

# Architecture

```
Binance WebSocket
        │
        ▼
Market Data Feed
        │
        ▼
mpsc Channel
        │
        ▼
Engine Event Loop
        │
        ▼
Trading Engine State
        ▲
        │
     Actix API
```

---

# Tech Stack

- Rust
- Tokio
- Actix Web
- tokio-tungstenite
- serde / serde_json

---

# Project Structure

```
src/
 ├ engine/
 │   ├ engine.rs
 │   ├ position.rs
 │   └ event.rs
 │
 ├ api/
 │   └ position.rs
 │
 ├ market/
 │   └ ws.rs
 │
 └ main.rs
```

---

# Run the Server

Start the backend:

```bash
cargo run
```

Server runs on:

```
http://127.0.0.1:8080
```

The engine will automatically connect to the Binance WebSocket
and stream live SOL price updates.

---

# API Endpoints

### Open Position

```
POST /position/open
```

### Get Positions

```
GET /positions
```

### Close Position

```
POST /position/close
```

### Get Current Price
```
GET /price
```

### Get Balance
```
GET /balance
```

### Get Trade History
```
GET /trade-history
```

---
# Engine Logic
```
Position size:

position_size = margin * leverage

Quantity:

quantity = position_size / entry_price

PnL calculation:

Long:
pnl = (current_price - entry_price) * quantity

Short:
pnl = (entry_price - current_price) * quantity

Liquidation price:

Long:
liq = entry_price * (1 - 1/leverage)

Short:
liq = entry_price * (1 + 1/leverage)
```
# Example Request

Open a long position:

POST /position/open

```json
{
  "asset": "SOL",
  "margin": 100,
  "leverage": 5,
  "position_type": "Long"
}
```
Response :

```json
{
  "id": "590682d4-d1a3-4cfa-893e-5f5eb8c8533c",
  "asset": "SOL",
  "entry_price": 89.35,
  "quantity": 5.59,
  "margin": 100,
  "leverage": 5,
  "pnl": 0.50,
  "position_type": "Long",
  "liquidation_price": 71.48
}
```

---

# Testing the Engine

Run the test script to simulate a full trade lifecycle.

Start the server first:

```bash
cargo run
```

Then run:

```bash
./test_engine.sh
```

The script will:

```
open position
wait for market movement
verify pnl
close position
```