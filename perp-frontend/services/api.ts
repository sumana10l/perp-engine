const API_BASE = "http://127.0.0.1:8080";

function getAuthHeaders() {
    const token = localStorage.getItem("token");
    return {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
    };
}

export async function login(username: string, password: string) {
    const res = await fetch(`${API_BASE}/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error("Login failed");
    const data = await res.json();
    localStorage.setItem("token", data.token);
    return data;
}

export async function getPositions() {
    const res = await fetch(`${API_BASE}/positions`, {
        headers: getAuthHeaders(),
    });
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
        headers: getAuthHeaders(),
        body: JSON.stringify(data),
    });
    if (!res.ok) {
        const errorText = await res.text();
        throw new Error(`Failed to open position: ${errorText}`);
    }
    return res.json();
}

export async function closePosition(positionId: string) {
    const res = await fetch(`${API_BASE}/position/close`, {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify({ position_id: positionId }),
    });
    if (!res.ok) throw new Error("Failed to close position");
    return res.json();
}

export async function getPrice() {
    const res = await fetch(`${API_BASE}/price`, { headers: getAuthHeaders() });
    if (!res.ok) throw new Error("Failed to fetch price");
    return res.json();
}

export async function getBalance() {
    const res = await fetch(`${API_BASE}/balance`, { headers: getAuthHeaders() });
    if (!res.ok) throw new Error("Failed to fetch balance");
    return res.json();
}

export async function getTradeHistory() {
    const res = await fetch(`${API_BASE}/trade-history`, { headers: getAuthHeaders() });
    if (!res.ok) throw new Error("Failed to fetch trade history");
    return res.json();
}