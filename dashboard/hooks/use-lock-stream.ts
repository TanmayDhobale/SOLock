"use client";

import { useEffect, useState } from "react";
import { io, Socket } from "socket.io-client";

interface HotAccountData {
    pubkey: string;
    contention_score: number;
    lock_attempts: number;
    avg_priority_fee: number;
}

interface ServerMessage {
    type: string;
    message?: string;
    data?: HotAccountData[];
}

export function useLockStream() {
    const [hotAccounts, setHotAccounts] = useState<HotAccountData[]>([]);
    const [isConnected, setIsConnected] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const socketUrl = process.env.NEXT_PUBLIC_WS_URL || "http://localhost:3001";

        const socket: Socket = io(socketUrl, {
            transports: ["websocket", "polling"],
            reconnection: true,
            reconnectionDelay: 1000,
            reconnectionAttempts: 5,
        });

        socket.on("connect", () => {
            console.log("WebSocket connected");
            setIsConnected(true);
            setError(null);
        });

        socket.on("disconnect", () => {
            console.log("WebSocket disconnected");
            setIsConnected(false);
        });

        socket.on("connect_error", (err) => {
            console.error("WebSocket connection error:", err);
            setError(err.message);
            setIsConnected(false);
        });

        // Listen for messages
        socket.onAny((eventName, ...args) => {
            try {
                const message: ServerMessage = JSON.parse(args[0]);

                if (message.type === "hot-accounts-update" && message.data) {
                    setHotAccounts(message.data);
                }
            } catch (e) {
                // Message might not be JSON, ignore
            }
        });

        return () => {
            socket.close();
        };
    }, []);

    return { hotAccounts, isConnected, error };
}
