# 📈 Perp Engine

A full-stack perpetual futures trading platform. Rust backend with real-time risk management + Next.js frontend dashboard.

---

## Overview

- **Backend (Rust):** High-performance trading engine with live Binance price feed
- **Frontend (Next.js):** Real-time trading dashboard with position management
- **Architecture:** Event-driven, non-blocking async with fixed-point math

---

## Quick Start

### Backend
```bash
cd backend
cargo run
# Server: http://localhost:8080
```

### Frontend
```bash
cd frontend
npm install
npm run dev
# Dashboard: http://localhost:3000
```

---

## Core Features

- **Leveraged Trading:** 1x–100x leverage with real-time validation
- **Auto-Liquidation:** Positions close at maintenance margin threshold
- **Fixed-Point Math:** `rust_decimal` for zero floating-point errors
- **Live Prices:** Real-time WebSocket feed from Binance
- **Risk Engine:** State machine with concurrent position management

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust, Actix-Web, Tokio, RwLock |
| Frontend | Next.js, React, TypeScript, Tailwind |
| Math | rust_decimal (fixed-point) |
| WebSocket | tokio-tungstenite (Binance) |

---

## Testing
```bash
cd backend
./test_engine.sh
```

Verifies: position creation, price updates, PnL calculations, liquidations.

---

## Project Structure
```
perp-engine/
├─ backend/
│  ├─ src/
│  │  ├─ engine/      # Trading logic
│  │  ├─ api/         # HTTP handlers
│  │  ├─ market/      # WebSocket feed
│  │  └─ main.rs      # Server entry
│  └─ Cargo.toml
├─ frontend/
│  ├─ components/     # React components
│  ├─ services/       # API client
│  └─ package.json
└─ README.md
```

---
