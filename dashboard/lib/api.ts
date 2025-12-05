export interface HotAccount {
    account_pubkey: string;
    lock_attempts: number;
    successful_locks: number;
    success_rate: number;
    avg_contention: number;
    max_contention: number;
    avg_priority_fee: number;
    max_priority_fee: number;
}

export interface AccountStats {
    pubkey: string;
    total_lock_attempts: number;
    successful_locks: number;
    failed_locks: number;
    success_rate: number;
    avg_contention: number;
    avg_priority_fee: number;
    max_priority_fee: number;
}

export interface DashboardStats {
    unique_accounts: number;
    total_events: number;
    high_contention_accounts: number;
    avg_success_rate: number;
}

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

export async function fetchHotAccounts(
    limit: number = 20,
    window: number = 5
): Promise<HotAccount[]> {
    const response = await fetch(
        `${API_BASE_URL}/api/hot-accounts?limit=${limit}&window=${window}`
    );
    if (!response.ok) {
        throw new Error("Failed to fetch hot accounts");
    }
    return response.json();
}

export async function fetchAccountStats(
    pubkey: string,
    window: number = 24
): Promise<AccountStats> {
    const response = await fetch(
        `${API_BASE_URL}/api/accounts/${pubkey}/stats?window=${window}`
    );
    if (!response.ok) {
        throw new Error("Failed to fetch account stats");
    }
    return response.json();
}

export async function estimatePriorityFee(
    accounts: string[]
): Promise<{ recommended_fee_lamports: number; recommended_fee_sol: number }> {
    const response = await fetch(`${API_BASE_URL}/api/priority-fees/estimate`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ accounts }),
    });
    if (!response.ok) {
        throw new Error("Failed to estimate priority fee");
    }
    return response.json();
}

export async function fetchDashboardStats(
    window: number = 5
): Promise<DashboardStats> {
    const response = await fetch(
        `${API_BASE_URL}/api/stats?window=${window}`
    );
    if (!response.ok) {
        throw new Error("Failed to fetch dashboard stats");
    }
    return response.json();
}

export interface LiveFeeEstimate {
    account: string;
    queue_depth: number;
    p90_fee_lamports: number;
    recommended_fee_lamports: number;
    recommended_fee_sol: number;
    avg_contention: number;
    slots_observed: number;
    freshness_seconds: number;
}

export async function fetchLiveFee(pubkey: string): Promise<LiveFeeEstimate> {
    const response = await fetch(
        `${API_BASE_URL}/api/accounts/${pubkey}/fee-now`
    );
    if (!response.ok) {
        throw new Error("Failed to fetch live fee");
    }
    return response.json();
}


