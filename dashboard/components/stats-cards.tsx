"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchDashboardStats } from "@/lib/api";

interface StatProps {
    label: string;
    value: string | number;
    suffix?: string;
}

function Stat({ label, value, suffix }: StatProps) {
    return (
        <div className="card p-8">
            <h3 className="mb-6">{label}</h3>
            <div className="stat-number">
                {typeof value === 'number' ? value.toLocaleString() : value}
                {suffix && <span className="text-2xl text-[rgba(255,255,255,0.4)] ml-1">{suffix}</span>}
            </div>
        </div>
    );
}

export function StatsCards() {
    const { data: stats, isLoading } = useQuery({
        queryKey: ["dashboard-stats"],
        queryFn: () => fetchDashboardStats(5),
        refetchInterval: 5000,
    });

    if (isLoading || !stats) {
        return (
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                {[...Array(4)].map((_, i) => (
                    <div key={i} className="card p-8">
                        <div className="skeleton h-3 w-24 mb-6" />
                        <div className="skeleton h-12 w-20" />
                    </div>
                ))}
            </div>
        );
    }

    return (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <Stat label="Accounts Tracked" value={stats.unique_accounts} />
            <Stat label="High Contention" value={stats.high_contention_accounts} />
            <Stat label="Total Events" value={stats.total_events} />
            <Stat label="Success Rate" value={stats.avg_success_rate.toFixed(1)} suffix="%" />
        </div>
    );
}
