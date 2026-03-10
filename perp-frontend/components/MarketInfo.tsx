"use client";

import { useEffect, useState } from "react";
import { getPrice, getBalance } from "../services/api";

export default function MarketInfo() {
  const [price, setPrice] = useState(0);
  const [balance, setBalance] = useState(0);

  const fetchData = async () => {
    try {
      const priceData = await getPrice();
      const balanceData = await getBalance();

      setPrice(priceData.price);
      setBalance(balanceData.balance);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    fetchData();

    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex justify-between bg-white p-4 rounded-lg shadow mb-8">
      <div className="text-lg font-semibold">
        SOL/USDT: {price.toFixed(2)}
      </div>

      <div className="text-lg font-semibold">
        Balance: ${balance.toFixed(2)}
      </div>
    </div>
  );
}