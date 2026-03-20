"use client";

import { useEffect, useState } from "react";
import { getTradeHistory } from "../services/api";

export default function TradeHistoryTable() {
  const [trades, setTrades] = useState<any[]>([]);

  const loadTrades = async () => {
    try {
      const data = await getTradeHistory();
      setTrades(data.trades || []);
    } catch (err) {
      console.error("Failed to load trades:", err);
      setTrades([]);
    }
  };

  useEffect(() => {
    loadTrades();

    const interval = setInterval(loadTrades, 3000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="mt-10 max-w-full mx-auto rounded-xl bg-gray-900 shadow p-4">
      <h2 className="text-lg font-bold text-white mb-4">Trade History</h2>

      <div className="overflow-y-auto max-h-[300px] rounded-lg border border-gray-700">
        <table className="w-full text-center text-sm text-gray-300">
          <thead className="bg-gray-800 text-gray-400 font-semibold">
            <tr>
              <th className="py-2">Entry</th>
              <th className="py-2">Exit</th>
              <th className="py-2">PnL ($)</th>
              <th className="py-2">Type</th>
            </tr>
          </thead>

          <tbody>
            {(trades || []).map((t, i) => (
              <tr key={i} className="border-t border-gray-700 hover:bg-gray-800">
                <td className="py-2">{t.entry.toFixed(4)}</td>
                <td className="py-2">{t.exit.toFixed(4)}</td>

                <td
                  className={`py-2 font-semibold ${t.pnl >= 0 ? "text-green-400" : "text-red-400"
                    }`}
                >
                  {t.pnl > 0 ? "+" : ""}{t.pnl.toFixed(4)}
                </td>

                <td className="py-2">
                  <span
                    className={`px-3 py-1 rounded-md font-medium ${t.position_type === "Long"
                      ? "bg-green-800 text-green-400"
                      : "bg-red-800 text-red-400"
                      }`}
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