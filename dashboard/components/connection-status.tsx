"use client";

import { Wifi, WifiOff, RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";

interface ConnectionStatusProps {
    connected: boolean;
    connecting: boolean;
    onReconnect?: () => void;
}

export function ConnectionStatus({ connected, connecting, onReconnect }: ConnectionStatusProps) {
    return (
        <div className="flex items-center gap-2">
            {connecting ? (
                <>
                    <RefreshCw className="h-4 w-4 animate-spin text-yellow-500" />
                    <span className="text-xs text-yellow-500">Connecting...</span>
                </>
            ) : connected ? (
                <>
                    <div className="relative">
                        <Wifi className="h-4 w-4 text-primary" />
                        <span className="absolute -top-0.5 -right-0.5 h-2 w-2 bg-primary rounded-full animate-pulse" />
                    </div>
                    <span className="text-xs text-primary">Live</span>
                </>
            ) : (
                <>
                    <WifiOff className="h-4 w-4 text-destructive" />
                    <span className="text-xs text-destructive">Disconnected</span>
                    {onReconnect && (
                        <button
                            onClick={onReconnect}
                            className="text-xs text-muted-foreground hover:text-foreground underline"
                        >
                            Retry
                        </button>
                    )}
                </>
            )}
        </div>
    );
}
