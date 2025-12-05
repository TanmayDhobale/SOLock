"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchHotAccounts } from "@/lib/api";
import { ExternalLink, Copy, Check } from "lucide-react";
import { shortenAddress, formatNumber, formatLamports } from "@/lib/utils";
import { getAccountLabel } from "@/lib/known-accounts";
import { useState } from "react";

export function HotAccountsTable() {
    const [copiedAddress, setCopiedAddress] = useState<string | null>(null);

    const { data: accounts, isLoading } = useQuery({
        queryKey: ["hot-accounts"],
        queryFn: () => fetchHotAccounts(20, 5),
        refetchInterval: 5000,
    });

    function getBadgeClass(contention: number) {
        if (contention >= 10) return "badge-critical";
        if (contention >= 5) return "badge-high";
        if (contention >= 2) return "badge-medium";
        return "badge-low";
    }

    function getBadgeLabel(contention: number) {
        if (contention >= 10) return "Critical";
        if (contention >= 5) return "High";
        if (contention >= 2) return "Medium";
        return "Low";
    }

    async function copyToClipboard(address: string) {
        await navigator.clipboard.writeText(address);
        setCopiedAddress(address);
        setTimeout(() => setCopiedAddress(null), 2000);
    }

    return (
        <div className="card">
            {/* Header */}
            <div className="p-8 border-b border-[rgba(255,255,255,0.08)]">
                <h3 className="mb-2">Hot Accounts</h3>
                <h2 className="font-display">Write Lock Contention</h2>
            </div>

            {/* Table */}
            <div className="overflow-x-auto">
                <table>
                    <thead>
                        <tr>
                            <th className="w-16">#</th>
                            <th>Account</th>
                            <th>Contention</th>
                            <th>Attempts</th>
                            <th>Success</th>
                            <th>Avg Fee</th>
                        </tr>
                    </thead>
                    <tbody>
                        {isLoading ? (
                            [...Array(5)].map((_, i) => (
                                <tr key={i}>
                                    <td><div className="skeleton h-4 w-6" /></td>
                                    <td><div className="skeleton h-10 w-40" /></td>
                                    <td><div className="skeleton h-4 w-20" /></td>
                                    <td><div className="skeleton h-4 w-16" /></td>
                                    <td><div className="skeleton h-4 w-24" /></td>
                                    <td><div className="skeleton h-4 w-20" /></td>
                                </tr>
                            ))
                        ) : accounts && accounts.length > 0 ? (
                            accounts.map((account, index) => {
                                const knownAccount = getAccountLabel(account.account_pubkey);

                                return (
                                    <tr key={account.account_pubkey} className="group">
                                        <td>
                                            <span className="font-display text-lg text-[rgba(255,255,255,0.3)]">
                                                {index + 1}
                                            </span>
                                        </td>
                                        <td>
                                            <div className="flex flex-col gap-1.5">
                                                <div className="flex items-center gap-2">
                                                    {knownAccount ? (
                                                        <>
                                                            <span className="font-medium text-white">
                                                                {knownAccount.label}
                                                            </span>
                                                            <span className="tag">{knownAccount.category}</span>
                                                        </>
                                                    ) : (
                                                        <span className="text-[rgba(255,255,255,0.5)]">Unknown</span>
                                                    )}
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <code className="mono text-[rgba(255,255,255,0.4)]">
                                                        {shortenAddress(account.account_pubkey, 4)}
                                                    </code>
                                                    <button
                                                        onClick={() => copyToClipboard(account.account_pubkey)}
                                                        className="p-1 hover:bg-[rgba(255,255,255,0.05)] rounded opacity-0 group-hover:opacity-100 transition-opacity"
                                                    >
                                                        {copiedAddress === account.account_pubkey ? (
                                                            <Check className="h-3 w-3 text-[#00d4aa]" />
                                                        ) : (
                                                            <Copy className="h-3 w-3 text-[rgba(255,255,255,0.4)]" />
                                                        )}
                                                    </button>
                                                    <a
                                                        href={`https://solscan.io/account/${account.account_pubkey}`}
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        className="p-1 hover:bg-[rgba(255,255,255,0.05)] rounded opacity-0 group-hover:opacity-100 transition-opacity"
                                                    >
                                                        <ExternalLink className="h-3 w-3 text-[rgba(255,255,255,0.4)]" />
                                                    </a>
                                                </div>
                                            </div>
                                        </td>
                                        <td>
                                            <div className="flex items-center gap-3">
                                                <span className="font-display text-xl text-white">
                                                    {account.avg_contention.toFixed(1)}
                                                </span>
                                                <span className={`badge ${getBadgeClass(account.avg_contention)}`}>
                                                    {getBadgeLabel(account.avg_contention)}
                                                </span>
                                            </div>
                                        </td>
                                        <td>
                                            <span className="text-white">
                                                {formatNumber(account.lock_attempts)}
                                            </span>
                                        </td>
                                        <td>
                                            <div className="flex items-center gap-3">
                                                <div className="progress w-16">
                                                    <div
                                                        className="progress-fill"
                                                        style={{
                                                            width: `${account.success_rate}%`,
                                                            background: account.success_rate >= 80 ? '#00d4aa' : account.success_rate >= 50 ? '#ffa502' : '#ff4757'
                                                        }}
                                                    />
                                                </div>
                                                <span className="text-[rgba(255,255,255,0.6)]">
                                                    {account.success_rate.toFixed(0)}%
                                                </span>
                                            </div>
                                        </td>
                                        <td>
                                            <span className="mono text-[#00d4aa]">
                                                {formatLamports(account.avg_priority_fee)}
                                            </span>
                                        </td>
                                    </tr>
                                );
                            })
                        ) : (
                            <tr>
                                <td colSpan={6} className="text-center py-16 text-[rgba(255,255,255,0.4)]">
                                    No hot accounts detected
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
