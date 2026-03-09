const API_BASE = "http://127.0.0.1:8080";

export async function getPositions() {
    const res = await fetch(`${API_BASE}/positions`);
    if (!res.ok) throw new Error("Failed to fetch positions");
    return res.json();
}

export async function openPosition(data: {
    asset: string;
    margin: number;
    leverage: number;
    position_type: string;
}) {
    const res = await fetch(`${API_BASE}/position/open`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    });

    if (!res.ok) throw new Error("Failed to open position");
    return res.json();
}

export async function closePosition(positionId: string) {
    const res = await fetch(`${API_BASE}/position/close`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            position_id: positionId,
        }),
    });

    if (!res.ok) throw new Error("Failed to close position");
    return res.json();
}