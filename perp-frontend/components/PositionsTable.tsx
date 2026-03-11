"use client";

import { useEffect, useState } from "react";
import { getPositions, closePosition } from "../services/api";

export default function PositionsTable() {
  const [positions, setPositions] = useState<any[]>([]);

  const loadPositions = async () => {
    const data = await getPositions();
    setPositions(Object.values(data));
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

      <div className="overflow-x-auto rounded-lg border bg-white shadow">
        <table className="w-full table-auto border-collapse text-center">
          <thead className="bg-gray-100">
            <tr>
              <th className="px-4 py-3">ID</th>
              <th className="px-4 py-3">Entry</th>
              <th className="px-4 py-3">Size</th>
              <th className="px-4 py-3">Margin ($)</th>
              <th className="px-4 py-3">Leverage</th>
              <th className="px-4 py-3">PnL ($)</th>
              <th className="px-4 py-3">Type</th>
              <th className="px-4 py-3">Liquidation</th>
              <th className="px-4 py-3">Action</th>
            </tr>
          </thead>

          <tbody>
            {positions.map((p: any) => (
              <tr key={p.id} className="border-t">
                <td className="px-4 py-3">{p.id.slice(0, 8)}</td>
                <td className="px-4 py-3">{p.entry_price.toFixed(2)}</td>
                <td className="px-4 py-3">{(p.margin * p.leverage).toFixed(2)}</td>
                <td className="px-4 py-3">{p.margin.toFixed(2)}</td>
                <td className="px-4 py-3">{p.leverage}x</td>

                <td
                  className={`px-4 py-3 font-semibold ${
                    p.pnl >= 0 ? "text-green-600" : "text-red-600"
                  }`}
                >
                  {p.pnl.toFixed(2)}
                </td>

                <td className="px-4 py-3">
                  <span
                    className={
                      p.position_type === "Long"
                        ? "bg-green-100 text-green-700 px-3 py-1 rounded-md"
                        : "bg-red-100 text-red-700 px-3 py-1 rounded-md"
                    }
                  >
                    {p.position_type}
                  </span>
                </td>

                <td className="px-4 py-3">
                  <span
                    className={
                      p.position_type === "Long"
                        ? "bg-red-100 text-red-700 px-3 py-1 rounded-md"
                        : "bg-green-100 text-green-700 px-3 py-1 rounded-md"
                    }
                  >
                    {p.liquidation_price.toFixed(2)}
                  </span>
                </td>

                <td className="px-4 py-3">
                  <button
                    onClick={() => handleClose(p.id)}
                    className="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-md"
                  >
                    Close
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}