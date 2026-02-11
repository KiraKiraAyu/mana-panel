import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/api";

export interface SystemInfo {
    hostname: string;
    os: string;
    os_version: string;
    kernel: string;
    uptime: number;
    cpu_count: number;
    total_memory: number;
}

export interface DiskInfo {
    name: string;
    mount_point: string;
    total: number;
    used: number;
    available: number;
    percent: number;
}

export interface NetworkInfo {
    name: string;
    rx_bytes: number;
    tx_bytes: number;
    rx_packets: number;
    tx_packets: number;
}

export interface SystemStats {
    cpu_usage: number;
    memory_used: number;
    memory_total: number;
    memory_percent: number;
    swap_used: number;
    swap_total: number;
    disks: DiskInfo[];
    networks: NetworkInfo[];
    load_avg: [number, number, number];
    timestamp: number;
}

export const useSystemStore = defineStore("system", () => {
    const info = ref<SystemInfo | null>(null);
    const stats = ref<SystemStats | null>(null);
    const statsHistory = ref<SystemStats[]>([]);
    const eventSource = ref<EventSource | null>(null);
    const isConnected = ref(false);
    const retryTimer = ref<ReturnType<typeof setTimeout> | null>(null);

    const fetchInfo = async () => {
        const response = await api.get("/system/info");
        info.value = response.data;
    };

    const fetchStats = async () => {
        const response = await api.get("/system/stats");
        stats.value = response.data;
        addToHistory(response.data);
    };

    const addToHistory = (stat: SystemStats) => {
        statsHistory.value.push(stat);
        // Keep last 60 data points (1 minute of data at 1s intervals)
        if (statsHistory.value.length > 60) {
            statsHistory.value.shift();
        }
    };

    const clearRetryTimer = () => {
        if (retryTimer.value) {
            clearTimeout(retryTimer.value);
            retryTimer.value = null;
        }
    };

    const startStreaming = () => {
        if (eventSource.value) return;
        clearRetryTimer();

        eventSource.value = new EventSource("/api/system/stats/stream");

        eventSource.value.onopen = () => {
            isConnected.value = true;
        };

        eventSource.value.onmessage = (event) => {
            const data = JSON.parse(event.data) as SystemStats;
            stats.value = data;
            addToHistory(data);
        };

        eventSource.value.onerror = () => {
            isConnected.value = false;
            stopStreaming();
            // Retry after 5 seconds
            retryTimer.value = setTimeout(() => {
                retryTimer.value = null;
                startStreaming();
            }, 5000);
        };
    };

    const stopStreaming = () => {
        clearRetryTimer();
        if (eventSource.value) {
            eventSource.value.close();
            eventSource.value = null;
            isConnected.value = false;
        }
    };

    return {
        info,
        stats,
        statsHistory,
        isConnected,
        fetchInfo,
        fetchStats,
        startStreaming,
        stopStreaming,
    };
});
