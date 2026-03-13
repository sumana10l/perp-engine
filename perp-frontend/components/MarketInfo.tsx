
"use client";

import { useEffect, useState } from "react";
import { getPrice, getBalance } from "../services/api";

type MarketInfoProps = {
  price: number;
  setPrice: React.Dispatch<React.SetStateAction<number>>;
  balance: number;
  setBalance: React.Dispatch<React.SetStateAction<number>>;
};

export default function MarketInfo({
  price,
  setPrice,
  balance,
  setBalance,
}: MarketInfoProps) {

  const [startPrice, setStartPrice] = useState<number | null>(null);
  const [high, setHigh] = useState<number | null>(null);
  const [low, setLow] = useState<number | null>(null);
  useEffect(() => {
    const fetchData = async () => {
      try {
        const priceData = await getPrice();
        const balanceData = await getBalance();
        const newPrice = priceData.price;
        setStartPrice((prev) => {
          if (prev === null && newPrice > 0) return newPrice;
          return prev;
        });
        setPrice(newPrice);
        setBalance(balanceData.balance);
      } catch (err) {
        console.error(err);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, []);

  const priceChange = startPrice ? price - startPrice : 0;
  const pricePercent = startPrice ? (priceChange / startPrice) * 100 : 0;


  useEffect(() => {
    if (price <= 0) return;
    if (high === null || price > high) setHigh(price);
    if (low === null || price < low) setLow(price);
  }, [price]);

  return (
    <div className="flex justify-between items-center bg-gray-900 text-white p-3 rounded-lg shadow mb-2 text-sm font-semibold">
      <div className="flex items-center space-x-2 min-w-[120px]">
        <span>SOL-PERP</span>
        <span className="bg-blue-600 text-xs px-2 py-1 rounded">50x</span>
      </div>

      <div className="flex flex-col items-center min-w-[100px]">
        <span className="text-lg font-bold">{price.toFixed(2)}</span>
        <span className="text-xs font-normal text-gray-400">Index Price {price.toFixed(2)}</span>
      </div>

      <div className={`min-w-[100px] ${priceChange >= 0 ? "text-green-400" : "text-red-400"}`}>
        {priceChange >= 0 ? "+" : ""}{priceChange.toFixed(2)} ({pricePercent.toFixed(2)}%)
      </div>
      <div className="min-w-[70px] text-gray-300">
        High: {high ? high.toFixed(2) : "-"}
      </div>

      <div className="min-w-[70px] text-gray-300">
        Low: {low ? low.toFixed(2) : "-"}
      </div>

      <div className="min-w-[120px] text-gray-300">
        27,343,333.62
      </div>

      <div className="min-w-[110px] text-white font-bold">
        Balance: ${balance.toFixed(2)}
      </div>
    </div>
  );
}