"use client";

import { Header } from "@/components/header";
import { HotAccountsTable } from "@/components/hot-accounts-table";
import { ContentionChart } from "@/components/contention-chart";
import { StatsCards } from "@/components/stats-cards";
import { useWebSocket } from "@/hooks/use-websocket";

export default function Page() {
  const { connected, connecting, reconnect } = useWebSocket();

  return (
    <div className="min-h-screen bg-black">
      <Header
        wsConnected={connected}
        wsConnecting={connecting}
        onReconnect={reconnect}
      />

      <main className="max-w-6xl mx-auto px-8 py-12 space-y-8">
        {/* Stats */}
        <StatsCards />

        {/* Main Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2">
            <HotAccountsTable />
          </div>
          <div>
            <ContentionChart />
          </div>
        </div>
      </main>
    </div>
  );
}
