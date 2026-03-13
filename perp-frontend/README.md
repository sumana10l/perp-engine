## Perp Engine Frontend

Frontend interface for the **Perp Engine trading system**.
It provides a simple trading dashboard where users can view market data, open positions, and monitor trades.

### Features

* Market information display
* Price chart visualization
* Open long/short positions
* View active positions
* View trade history

### Tech Stack

* Next.js
* React
* TypeScript

### Project Structure

```
app/
  layout.tsx
  page.tsx
  globals.css

components/
  MarketInfo.tsx
  OpenPositionForm.tsx
  PositionsTable.tsx
  TradeChart.tsx
  TradeHistoryTable.tsx

services/
  api.ts
```

### Running Locally

```bash
npm install
npm run dev
```

The application will start on:

```
http://localhost:3000
```

---
