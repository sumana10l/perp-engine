# Perp Engine

A simplified perpetual futures trading engine with a **Rust backend** and a **Next.js trading dashboard**.  
The backend consumes real-time market prices and exposes APIs used by the frontend trading interface.

## Screenshot

![Trading Dashboard](./screenshots/dashboard.png)

## Architecture

```
Binance WebSocket → Rust Trading Engine → REST API → Frontend Dashboard
```

The backend subscribes to the **:contentReference[oaicite:0]{index=0} trade stream for live market prices**, processes them inside the engine, and exposes the state to the frontend.

## Repository Structure

```
perp-engine/
 ├─ backend/     # Trading engine and API
 ├─ frontend/    # Trading dashboard
 └─ README.md
```

## Tech Stack

**Backend**
- Rust
- Tokio
- WebSockets

**Frontend**
- Next.js
- React
- TypeScript

## Run Locally

### Start Backend
```
cd backend
cargo run
```

### Start Frontend
```
cd frontend
npm install
npm run dev
```

Open: `http://localhost:3000`