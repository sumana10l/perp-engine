"use client";

import { useEffect, useState } from "react";
import { getTradeHistory } from "../services/api";

export default function TradeHistoryTable() {
    const [trades, setTrades] = useState<any[]>([]);

    const loadTrades = async () => {
      try {
        const data = await getTradeHistory();
        setTrades(data);
      } catch (err) {
        console.error(err);
      }
    };
  
    useEffect(() => {
      loadTrades();
  
      const interval = setInterval(loadTrades, 3000);
      return () => clearInterval(interval);
    }, []);

  return (
    <div className="mt-10 max-w-4xl mx-auto">
    <h2 className="text-xl font-bold mb-4">Trade History</h2>
  
    <div className="bg-white rounded-lg border shadow overflow-hidden">
      <table className="w-full text-center">
        <thead className="bg-gray-100">
          <tr>
            <th className="py-3">Entry</th>
            <th className="py-3">Exit</th>
            <th className="py-3">PnL ($)</th>
            <th className="py-3">Type</th>
          </tr>
        </thead>
  
        <tbody>
          {trades.map((t, i) => (
            <tr key={i} className="border-t hover:bg-gray-50">
              <td className="py-3">{t.entry.toFixed(2)}</td>
              <td className="py-3">{t.exit.toFixed(2)}</td>
  
              <td
                className={`py-3 font-semibold ${
                  t.pnl >= 0 ? "text-green-600" : "text-red-600"
                }`}
              >
                {t.pnl.toFixed(2)}
              </td>
  
              <td className="py-3">
                <span
                  className={
                    t.position_type === "Long"
                      ? "bg-green-100 text-green-700 px-3 py-1 rounded-md font-medium"
                      : "bg-red-100 text-red-700 px-3 py-1 rounded-md font-medium"
                  }
                >
                  {t.position_type}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  </div>
  );
}