"use client";

import { useEffect, useState, useCallback, useRef } from "react";

interface HotAccountData {
    pubkey: string;
    contention_score: number;
    lock_attempts: number;
    avg_priority_fee: number;
}

interface WebSocketMessage {
    type: string;
    message?: string;
    data?: HotAccountData[];
}

interface UseWebSocketResult {
    connected: boolean;
    connecting: boolean;
    hotAccounts: HotAccountData[];
    error: string | null;
    reconnect: () => void;
}

export function useWebSocket(): UseWebSocketResult {
    const [connected, setConnected] = useState(false);
    const [connecting, setConnecting] = useState(false);
    const [hotAccounts, setHotAccounts] = useState<HotAccountData[]>([]);
    const [error, setError] = useState<string | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    const connect = useCallback(() => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            return;
        }

        setConnecting(true);
        setError(null);

        try {
            const ws = new WebSocket("ws://localhost:3001/ws");
            wsRef.current = ws;

            ws.onopen = () => {
                console.log("WebSocket connected");
                setConnected(true);
                setConnecting(false);
                setError(null);

                // Subscribe to hot-accounts channel
                ws.send(JSON.stringify({
                    type: "subscribe",
                    channel: "hot-accounts"
                }));
            };

            ws.onmessage = (event) => {
                try {
                    const message: WebSocketMessage = JSON.parse(event.data);

                    switch (message.type) {
                        case "connected":
                            console.log("Server message:", message.message);
                            break;
                        case "hot-accounts-update":
                            if (message.data) {
                                setHotAccounts(message.data);
                            }
                            break;
                        case "error":
                            console.error("Server error:", message.message);
                            setError(message.message || "Unknown error");
                            break;
                    }
                } catch (e) {
                    console.error("Failed to parse WebSocket message:", e);
                }
            };

            ws.onerror = () => {
                console.error("WebSocket error occurred");
                setError("Connection error");
            };

            ws.onclose = () => {
                console.log("WebSocket disconnected");
                setConnected(false);
                setConnecting(false);
                wsRef.current = null;

                // Attempt to reconnect after 5 seconds
                reconnectTimeoutRef.current = setTimeout(() => {
                    console.log("Attempting to reconnect...");
                    connect();
                }, 5000);
            };
        } catch (e) {
            console.error("Failed to create WebSocket:", e);
            setConnecting(false);
            setError("Failed to connect");
        }
    }, []);

    const reconnect = useCallback(() => {
        if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current);
        }
        if (wsRef.current) {
            wsRef.current.close();
        }
        connect();
    }, [connect]);

    useEffect(() => {
        connect();

        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            if (wsRef.current) {
                wsRef.current.close();
            }
        };
    }, [connect]);

    return {
        connected,
        connecting,
        hotAccounts,
        error,
        reconnect,
    };
}
