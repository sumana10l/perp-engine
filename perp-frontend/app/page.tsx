"use client";

import { useState, useEffect } from "react";
import OpenPositionForm from "../components/OpenPositionForm";
import PositionsTable from "../components/PositionsTable";
import MarketInfo from "../components/MarketInfo";
import TradeHistoryTable from "../components/TradeHistoryTable";
import TradeChart from "../components/TradeChart";
import Login from "../components/Login";

export default function Home() {
  const [authed, setAuthed] = useState(false);
  const [balance, setBalance] = useState(0);
  const [price, setPrice] = useState(0);
  const [activePanel, setActivePanel] = useState<"positions" | "history" | null>("positions");

  useEffect(() => {
    setAuthed(!!localStorage.getItem("token"));
  }, []);

  if (!authed) return <Login onLogin={() => setAuthed(true)} />;

  return (
    <main className="min-h-screen p-10 bg-gray-900 text-white">
      <div className="flex items-center justify-between mb-10">
        <h1 className="text-3xl font-bold">Perp Trading Dashboard</h1>
        <button
          onClick={() => {
            localStorage.removeItem("token");
            setAuthed(false);
          }}
          className="text-sm text-gray-400 hover:text-white transition"
        >
          Logout
        </button>
      </div>

      <div className="bg-gray-800 rounded-xl shadow p-4 mb-6">
        <MarketInfo
          price={price}
          setPrice={setPrice}
          balance={balance}
          setBalance={setBalance}
        />
      </div>

      <div className="grid grid-cols-12 gap-6 mt-8">
        <div className="col-span-8 flex flex-col">
          <div className="bg-gray-800 rounded-xl shadow p-4">
            <TradeChart price={price} />
          </div>

          <div className="flex space-x-4 mt-4 mb-2">
            <button
              onClick={() => setActivePanel("positions")}
              className={`px-4 py-2 rounded font-semibold ${activePanel === "positions" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-400"}`}
              aria-pressed={activePanel === "positions"}
              aria-label="Show Open Positions"
            >
              Open Positions
            </button>
            <button
              onClick={() => setActivePanel("history")}
              className={`px-4 py-2 rounded font-semibold ${activePanel === "history" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-400"}`}
              aria-pressed={activePanel === "history"}
              aria-label="Show Trade History"
            >
              Trade History
            </button>
          </div>

          <div className="overflow-auto max-h-[400px]">
            {activePanel === "positions" && (
              <div className="bg-gray-800 rounded-xl shadow p-4 overflow-auto">
                <PositionsTable />
              </div>
            )}
            {activePanel === "history" && (
              <div className="bg-gray-800 rounded-xl shadow p-4 overflow-auto">
                <TradeHistoryTable />
              </div>
            )}
          </div>
        </div>
        <div className="col-span-4">
          <div className="bg-gray-800 rounded-xl shadow p-4 sticky top-10">
            <OpenPositionForm balance={balance} price={price} />
          </div>
        </div>
      </div>
    </main>
  );
}