"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchHotAccounts } from "@/lib/api";
import { shortenAddress } from "@/lib/utils";

export function ContentionChart() {
    const { data: accounts, isLoading } = useQuery({
        queryKey: ["hot-accounts"],
        queryFn: () => fetchHotAccounts(10, 5),
        refetchInterval: 5000,
    });

    const chartData = accounts?.slice(0, 6) ?? [];
    const maxContention = Math.max(...(chartData.map(a => a.avg_contention) || [1]), 1);

    return (
        <div className="card h-full">
            {/* Header */}
            <div className="p-8 border-b border-[rgba(255,255,255,0.08)]">
                <h3 className="mb-2">Overview</h3>
                <h2 className="font-display">Top Accounts</h2>
            </div>

            {/* Chart */}
            <div className="p-8">
                {isLoading ? (
                    <div className="space-y-6">
                        {[...Array(5)].map((_, i) => (
                            <div key={i}>
                                <div className="skeleton h-3 w-20 mb-2" />
                                <div className="skeleton h-4 w-full" />
                            </div>
                        ))}
                    </div>
                ) : chartData.length > 0 ? (
                    <div className="space-y-6">
                        {chartData.map((account, index) => {
                            const percentage = (account.avg_contention / maxContention) * 100;

                            return (
                                <div key={account.account_pubkey}>
                                    <div className="flex items-center justify-between mb-2">
                                        <code className="mono text-[rgba(255,255,255,0.5)]">
                                            {shortenAddress(account.account_pubkey, 4)}
                                        </code>
                                        <span className="font-display text-lg text-white">
                                            {account.avg_contention.toFixed(1)}
                                        </span>
                                    </div>
                                    <div className="contention-bar">
                                        <div
                                            className="contention-bar-fill"
                                            style={{
                                                width: `${percentage}%`,
                                                background: index < 3 ? '#00d4aa' : 'rgba(255,255,255,0.2)'
                                            }}
                                        />
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                ) : (
                    <div className="text-center py-12 text-[rgba(255,255,255,0.4)]">
                        No data
                    </div>
                )}
            </div>
        </div>
    );
}
