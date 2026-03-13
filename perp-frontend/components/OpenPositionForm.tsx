"use client";

import { useState } from "react";
import { openPosition } from "../services/api";
type Props = {
  balance: number;
  price: number;
};
export default function OpenPositionForm({ balance, price }: Props) {
  const [margin, setMargin] = useState(100);
  const [leverage, setLeverage] = useState(5);
  const [positionType, setPositionType] = useState<"Long" | "Short">("Long");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (type: "Long" | "Short") => {
    console.log(type);
    setPositionType(type);
    if (margin <= 0) {
      alert("Margin must be greater than 0");
      return;
    }

    if (leverage > 50) {
      alert("Max leverage is 50x");
      return;
    }

    if (margin > balance) {
      alert("Not enough balance");
      return;
    }
    setLoading(true);


    try {
      await openPosition({
        asset: "SOL",
        margin,
        leverage,
        position_type: type
      });

      alert("Position opened successfully");
      setMargin(100);
      setLeverage(5);
      setPositionType("Long");
    } catch (error) {
      console.error(error);
      alert("Failed to open position");
    }

    setLoading(false);

  };

  const calculateLiqPrice = (type: "Long" | "Short") => {
    if (price <= 0) return null;

    if (type === "Long") {
      return price * (1 - 1 / leverage);
    } else {
      return price * (1 + 1 / leverage);
    }
  };

  const liquidationPrice = calculateLiqPrice(positionType);

  return (
    <div className="bg-gray-900 p-6 rounded-xl shadow-lg flex flex-col space-y-6 text-white">

      <div className="flex space-x-3">
        <button
          disabled={loading}
          onClick={() => handleSubmit("Long")}
          className={`flex-grow py-2 rounded font-semibold ${positionType === "Long" ? "bg-green-600" : "bg-gray-700"
            }`}
        >
          Buy / Long
        </button>
        <button
          disabled={loading}
          onClick={() => handleSubmit("Short")}
          className={`flex-grow py-2 rounded font-semibold ${positionType === "Short" ? "bg-red-600" : "bg-gray-700"
            }`}
        >
          Sell / Short
        </button>
      </div>

      {/* Todo: add position type selection */}

      {/* <div className="flex space-x-3 text-sm font-semibold mb-3">
        {["Limit", "Market", "Conditional"].map((type) => (
          <button
          disabled
            key={type}
            onClick={() => setPositionType(type)}
            className={`px-3 py-1 rounded ${positionType === type ? "bg-gray-700" : "bg-gray-800"
              }`}
          >
            {type}
          </button>
        ))}
      </div> */}

      <div>
        <label className="block text-xs mb-1">Market Price</label>
        <div className="flex items-center space-x-2">
          <input
            type="number"
            value={price}
            className="flex-grow bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white"
            placeholder="Price"
            readOnly
          />
          {/*Todo : add mid and bbo */}
          {/* <div className="space-x-2 text-xs">
            <button className="px-2 py-1 bg-gray-700 rounded">Mid</button>
            <button className="px-2 py-1 bg-gray-700 rounded">BBO</button>
          </div> */}
        </div>
      </div>

      <div>
        <label className="block text-xs mb-1">Margin</label>
        <div className="flex items-center space-x-2 mb-1">
          <input
            type="number"
            value={margin}
            onChange={(e) => setMargin(Number(e.target.value))}
            className="flex-grow bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white"
            placeholder="Margin"
          />
          <div className="text-xs text-gray-400">USD</div>
        </div>

        <input
          type="range"
          min={0}
          max={balance}
          value={margin}
          onChange={(e) => setMargin(Number(e.target.value))}
          className="w-full"
        />
      </div>

      <div>
        <label className="block text-xs mb-1">Position Size</label>
        <input
          type="number"
          value={(margin * leverage).toFixed(2)}
          readOnly
          className="bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white w-full"
        />
      </div>
      <div>
        <label className="block text-xs mb-2">Leverage</label>

        <div className="grid grid-cols-4 gap-2">
          {[5, 10, 20, 50].map((l) => (
            <button
              key={l}
              onClick={() => setLeverage(l)}
              className={`py-2 rounded text-sm font-semibold ${leverage === l ? "bg-blue-600" : "bg-gray-700"
                }`}
            >
              {l}x
            </button>
          ))}
        </div>
      </div>

      <div className="text-xs text-gray-400 space-y-1">
        <div>Available Equity: ${balance.toFixed(2)}</div>
        <div>Margin Required: ${margin.toFixed(2)}</div>
        <div>
          Est. Liquidation Price: {liquidationPrice ? `$${liquidationPrice.toFixed(2)}` : "-"}

        </div>

        {/* <button className="bg-white text-black rounded py-3 font-semibold w-full"
          onClick={handleSubmit}
          disabled={loading}
        >
          {loading ? "Opening..." : "Open Position"}
        </button> */}

      </div>
    </div>
  );
}