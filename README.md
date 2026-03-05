# Perp Engine (Rust)

A minimal **perpetual trading backend** built with Actix Web.
The project simulates core exchange logic like margin trading, leverage, and PnL updates.

---

# Features

* Open leveraged positions
* Long / Short support
* Real-time PnL updates
* Close positions
* In-memory trading engine
* REST API

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

---

# API Endpoints

**Open Position**

```
POST /position/open
```

**Update Price**

```
POST /price/update
```

**Get Positions**

```
GET /positions
```

**Close Position**

```
POST /position/close
```

---

# Test the Engine

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
initialize price
open position
update price
verify pnl
close position
```

---

```
