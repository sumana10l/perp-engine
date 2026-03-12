
"use client";

import { useEffect } from "react";
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
    <div className="flex justify-between items-center bg-gray-900 text-white p-3 rounded-lg shadow mb-2 text-sm font-semibold">
      <div className="flex items-center space-x-2 min-w-[120px]">
        <span>SOL-PERP</span>
        <span className="bg-blue-600 text-xs px-2 py-1 rounded">50x</span>
      </div>

      <div className="flex flex-col items-center min-w-[100px]">
        <span className="text-lg font-bold">{price.toFixed(2)}</span>
        <span className="text-xs font-normal text-gray-400">Index Price 85.99</span>
      </div>

      <div className="text-green-400 min-w-[100px]">
        +0.81 +0.95%
      </div>

      <div className="min-w-[140px] text-yellow-500">
        -0.0002% / 00:38:48
      </div>

      <div className="min-w-[70px] text-gray-300">
        88.09
      </div>

      <div className="min-w-[70px] text-gray-300">
        84.33
      </div>

      <div className="min-w-[140px] text-gray-300">
        77,932,723.51
      </div>

      <div className="min-w-[120px] text-gray-300">
        479,605.88
      </div>

      <div className="min-w-[110px] text-white font-bold">
        Balance: ${balance.toFixed(2)}
      </div>
    </div>
  );
}