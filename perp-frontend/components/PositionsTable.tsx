"use client";

import { useEffect, useState } from "react";
import { getPositions, closePosition } from "../services/api";

export default function PositionsTable() {
  const [positions, setPositions] = useState<any[]>([]);

  const loadPositions = async () => {
    try {
      const data = await getPositions();
      setPositions(data.positions || []);
    } catch (error) {
      console.error("Error loading positions:", error);
    }
  };

  useEffect(() => {
    loadPositions();
    const interval = setInterval(loadPositions, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleClose = async (id: string) => {
    await closePosition(id);
    loadPositions();
  };

  return (
    <div className="mt-10 max-w-5xl mx-auto">
      <h2 className="text-xl font-bold mb-4">Open Positions</h2>
      <p className="text-sm text-gray-500 mb-4">Updating every 2 seconds</p>

      <div className="overflow-x-auto rounded-lg border border-gray-700 bg-gray-800 shadow-inner">
        <table className="w-full table-auto border-collapse text-sm text-center text-gray-300">
          <thead className="bg-gray-700">
            <tr>
              <th className="px-3 py-2 font-semibold">ID</th>
              <th className="px-3 py-2 font-semibold">Entry</th>
              <th className="px-3 py-2 font-semibold">Size</th>
              <th className="px-3 py-2 font-semibold">Margin ($)</th>
              <th className="px-3 py-2 font-semibold">Leverage</th>
              <th className="px-3 py-2 font-semibold">PnL ($)</th>
              <th className="px-3 py-2 font-semibold">Type</th>
              <th className="px-3 py-2 font-semibold">Liquidation</th>
              <th className="px-3 py-2 font-semibold">Action</th>
            </tr>
          </thead>
          <tbody>
            {positions.map((p: any) => {
              const shortId = p.id ? String(p.id).slice(0, 8) : "N/A";

              return (
                <tr key={p.id} className="border-t border-gray-700 hover:bg-gray-700">
                  <td className="px-3 py-2 font-mono text-xs text-gray-500">{shortId}</td>
                  <td className="px-3 py-2">{(p.entry_price ?? 0).toFixed(2)}</td>
                  <td className="px-3 py-2">{(p.quantity ?? 0).toFixed(4)}</td>
                  <td className="px-3 py-2">{(p.margin ?? 0).toFixed(2)}</td>
                  <td className="px-3 py-2">{p.leverage}x</td>
                  <td className={`px-3 py-2 font-semibold ${(p.pnl ?? 0) >= 0 ? "text-green-400" : "text-red-500"}`}>
                    {(p.pnl ?? 0) > 0 ? "+" : ""}{(p.pnl ?? 0).toFixed(4)}
                  </td>
                  <td className="px-3 py-2">
                    <span className={`px-2 py-1 rounded-md font-medium ${p.position_type === "Long" ? "bg-green-800 text-green-300" : "bg-red-800 text-red-300"}`}>
                      {p.position_type}
                    </span>
                  </td>
                  <td className="px-3 py-2">
                    <span className="text-orange-400 font-medium">
                      {(p.liquidation_price ?? 0).toFixed(2)}
                    </span>
                  </td>
                  <td className="px-3 py-2">
                    <button
                      onClick={() => handleClose(p.id)}
                      className="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded-md text-sm transition-colors"
                    >
                      Close
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}