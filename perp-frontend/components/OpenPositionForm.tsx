"use client";

import { useState } from "react";
import { openPosition } from "../services/api";

export default function OpenPositionForm() {
    const [margin, setMargin] = useState(100);
    const [leverage, setLeverage] = useState(5);
    const [positionType, setPositionType] = useState("Long");
    const [loading, setLoading] = useState(false);

    const handleSubmit = async () => {
        setLoading(true);

        try {
            await openPosition({
                asset: "SOL",
                margin,
                leverage,
                position_type: positionType,
            });

            alert("Position opened successfully");
        } catch (error) {
            console.error(error);
            alert("Failed to open position");
        }

        setLoading(false);
    };

    return (
        <div className="p-6 border rounded-lg w-96 bg-white shadow">
            <h2 className="text-xl font-semibold mb-4">Open Position</h2>

            <div className="mb-3">
                <label className="block mb-1">Margin</label>
                <input
                    type="number"
                    value={margin}
                    onChange={(e) => setMargin(Number(e.target.value))}
                    className="border p-2 w-full"
                />
            </div>

            <div className="mb-3">
                <label className="block mb-1">Leverage</label>
                <input
                    type="number"
                    value={leverage}
                    onChange={(e) => setLeverage(Number(e.target.value))}
                    className="border p-2 w-full"
                />
            </div>

            <div className="mb-4">
                <label className="block mb-1">Position Type</label>
                <select
                    value={positionType}
                    onChange={(e) => setPositionType(e.target.value)}
                    className="border p-2 w-full"
                >
                    <option value="Long">Long</option>
                    <option value="Short">Short</option>
                </select>
            </div>

            <button
                onClick={handleSubmit}
                disabled={loading}
                className="bg-blue-500 text-white px-4 py-2 rounded w-full"
            >
                {loading ? "Opening..." : "Open Position"}
            </button>
        </div>
    );
}