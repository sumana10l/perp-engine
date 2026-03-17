"use client";

import { useEffect, useRef } from "react";
import { createChart, UTCTimestamp, IChartApi, ISeriesApi } from "lightweight-charts";

type Candle = {
  time: UTCTimestamp;
  open: number;
  high: number;
  low: number;
  close: number;
};

export default function TradeChart({ price }: { price: number }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const candlesRef = useRef<Candle[]>([]);

  useEffect(() => {
    if (!containerRef.current) return;

    const chart = createChart(containerRef.current, {
      width: containerRef.current.clientWidth,
      height: 500,
      layout: {
        background: { color: "#1e1e2f" },
        textColor: "#d1d4dc"
      },
      grid: {
        vertLines: { color: "#2a2e42" },
        horzLines: { color: "#2a2e42" }
      },
      rightPriceScale: {
        autoScale: true,
        borderColor: "#2a2e42",
        visible: true,
      },

      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        barSpacing: 15,
        tickMarkFormatter: (time: number) => {
          const date = new Date(time * 1000);
          return date.toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
            hour12: false
          });
        },
      },
    });

    const candleSeries = chart.addCandlestickSeries({
      upColor: "#26a69a",
      downColor: "#ef5350",
      wickUpColor: "#26a69a",
      wickDownColor: "#ef5350",
      borderVisible: true,
      borderUpColor: "#26a69a",
      borderDownColor: "#ef5350",
    });

    if (price > 0) {
      const history = generateHistory(price, 50);
      candlesRef.current = history;
      candleSeries.setData(history);
    }

    chartRef.current = chart;
    candleSeriesRef.current = candleSeries;

    const handleResize = () => {
      if (containerRef.current && chartRef.current) {
        chartRef.current.applyOptions({ width: containerRef.current.clientWidth });
      }
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };

  }, [price === 0]);

  useEffect(() => {
    if (!candleSeriesRef.current || !price || price <= 0) return;

    const now = (Math.floor(Date.now() / 60000) * 60) as UTCTimestamp;
    const currentHistory = candlesRef.current;
    const lastCandle = currentHistory[currentHistory.length - 1];

    let currentCandle: Candle;

    if (!lastCandle || now > lastCandle.time) {
      currentCandle = {
        time: now,
        open: lastCandle ? lastCandle.close : price,
        high: price,
        low: price,
        close: price,
      };
      candlesRef.current.push(currentCandle);
    } else {
      currentCandle = {
        ...lastCandle,
        high: Math.max(lastCandle.high, price),
        low: Math.min(lastCandle.low, price),
        close: price,
      };
      candlesRef.current[candlesRef.current.length - 1] = currentCandle;
    }

    candleSeriesRef.current.update(currentCandle);

    if (chartRef.current) {
      chartRef.current.timeScale().scrollToRealTime();
    }

  }, [price]);

  return (
    <div
      ref={containerRef}
      className="w-full h-[500px] rounded-xl shadow-lg bg-[#1e1e2f]"
    />
  );
}

const generateHistory = (currentPrice: number, count: number): Candle[] => {
  const history: Candle[] = [];
  const nowInSeconds = Math.floor(Date.now() / 1000);
  const roundedNow = Math.floor(nowInSeconds / 60) * 60;
  let lastClose = currentPrice;

  for (let i = count; i > 0; i--) {
    const time = (roundedNow - i * 60) as UTCTimestamp;

    const move = (Math.random() - 0.5) * 0.15;
    const open = lastClose;
    const close = open + move;
    const high = Math.max(open, close) + Math.random() * 0.05;
    const low = Math.min(open, close) - Math.random() * 0.05;

    history.push({ time, open, high, low, close });
    lastClose = close;
  }
  return history;
};