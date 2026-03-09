"use client";

import { useEffect, useState } from "react";
import { getPositions, closePosition } from "../services/api";

export default function PositionsTable() {
  const [positions, setPositions] = useState<any>({});

  const loadPositions = async () => {
    const data = await getPositions();
    setPositions(data);
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
    <div className="mt-10">
      <h2 className="text-xl font-bold mb-4">Open Positions</h2>
      <p className="text-sm text-gray-500 mb-2">
        Updating every 2 seconds
      </p>
      <table className="border w-full">
        <thead>
          <tr className="border-b">
            <th>ID</th>
            <th>Entry</th>
            <th>Margin</th>
            <th>Leverage</th>
            <th>PnL</th>
            <th>Action</th>
          </tr>
        </thead>

        <tbody>
          {Object.values(positions).map((p: any) => (
            <tr key={p.id} className="border-b text-center">
              <td>{p.id.slice(0, 8)}</td>
              <td>{p.entry_price}</td>
              <td>{p.margin}</td>
              <td>{p.leverage}x</td>
              <td className={p.pnl >= 0 ? "text-green-600" : "text-red-600"}>
                {p.pnl.toFixed(2)}
              </td>
              <td>
                <button
                  onClick={() => handleClose(p.id)}
                  className="bg-red-500 text-white px-2 py-1 rounded"
                >
                  Close
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}