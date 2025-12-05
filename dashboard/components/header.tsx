"use client";

import { Zap } from "lucide-react";

interface HeaderProps {
    wsConnected?: boolean;
    wsConnecting?: boolean;
    onReconnect?: () => void;
}

export function Header({ wsConnected = false, wsConnecting = false, onReconnect }: HeaderProps) {
    return (
        <header className="sticky top-0 z-50 border-b border-[rgba(255,255,255,0.08)] bg-black/80 backdrop-blur-md">
            <div className="max-w-6xl mx-auto px-8 h-16 flex items-center justify-between">
                {/* Logo */}
                <div className="flex items-baseline gap-2">
                    <span className="font-display text-xl text-white">Solana</span>
                    <span className="font-display text-xl text-[#00d4aa]">Lock</span>
                </div>

                {/* Status */}
                <div className="flex items-center gap-6">
                    {/* Connection */}
                    <div className="flex items-center gap-2">
                        {wsConnecting ? (
                            <span className="text-sm text-[rgba(255,255,255,0.4)]">Connecting...</span>
                        ) : wsConnected ? (
                            <>
                                <span className="live-dot" />
                                <span className="text-sm text-[#00d4aa]">Live</span>
                            </>
                        ) : (
                            <>
                                <span className="text-sm text-[rgba(255,255,255,0.4)]">Offline</span>
                                {onReconnect && (
                                    <button onClick={onReconnect} className="text-sm text-[rgba(255,255,255,0.4)] hover:text-white underline">
                                        Retry
                                    </button>
                                )}
                            </>
                        )}
                    </div>

                    {/* Network */}
                    <div className="flex items-center gap-2 text-sm text-[rgba(255,255,255,0.4)]">
                        <Zap className="h-3.5 w-3.5" />
                        <span>Mainnet</span>
                    </div>
                </div>
            </div>
        </header>
    );
}
