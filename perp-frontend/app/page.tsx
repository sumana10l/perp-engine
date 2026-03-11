"use client";

import { useState } from "react";
import OpenPositionForm from "../components/OpenPositionForm";
import PositionsTable from "../components/PositionsTable";
import MarketInfo from "../components/MarketInfo";
import TradeHistoryTable from "../components/TradeHistoryTable";

export default function Home() {
  const [balance, setBalance] = useState(0);

  return (
    <main className="min-h-screen p-10 bg-gray-100 text-black">
    <h1 className="text-3xl font-bold mb-10">Perp Trading Dashboard</h1>
  
    <MarketInfo balance={balance} setBalance={setBalance} />
  
    <div className="grid grid-cols-3 gap-10 mt-10">
      
      <div className="col-span-1">
        <OpenPositionForm balance={balance} />
      </div>
  
      <div className="col-span-2 space-y-10">
        <PositionsTable />
        <TradeHistoryTable />
      </div>
  
    </div>
  </main>
  );
}