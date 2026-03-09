# Perp Engine (Rust)

A minimal **event-driven perpetual trading backend** written in Rust.

The engine simulates core exchange mechanics including margin trading,
leverage, position management, and real-time PnL updates using live
market data from Binance.

---

# Features

- Open leveraged positions
- Long / Short support
- Real-time PnL updates
- Close positions
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

---

# Example Request

Open a long position:

```json
POST /position/open

{
  "asset": "SOL",
  "margin": 100,
  "leverage": 5,
  "position_type": "Long"
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