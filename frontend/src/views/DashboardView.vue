<template>
    <div
        class="p-6 space-y-6 animate-[fade-in_0.3s_ease-out,slide-up_0.3s_ease-out]"
    >
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-text-primary">Dashboard</h1>
                <p class="text-text-muted mt-1">
                    System overview and real-time monitoring
                </p>
            </div>
            <div class="flex items-center gap-2">
                <span
                    class="w-2 h-2 rounded-full"
                    :class="
                        systemStore.isConnected
                            ? 'bg-success animate-pulse'
                            : 'bg-error'
                    "
                ></span>
                <span class="text-sm text-text-secondary">
                    {{ systemStore.isConnected ? 'Live' : 'Disconnected' }}
                </span>
            </div>
        </div>

        <!-- System Info Cards -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <!-- CPU Card -->
            <div
                class="relative overflow-hidden rounded-2xl border border-border-subtle bg-surface-elevated p-6 before:absolute before:inset-x-0 before:top-0 before:h-0.75 before:bg-linear-to-r before:from-reisa-lilac-500 before:via-reisa-pink-500 before:to-reisa-gold-500"
            >
                <div class="flex items-center justify-between mb-4">
                    <div
                        class="w-10 h-10 rounded-xl bg-reisa-lilac-500/20 flex items-center justify-center"
                    >
                        <svg
                            class="w-5 h-5 text-reisa-lilac-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
                            />
                        </svg>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full px-3 py-1 text-xs font-medium uppercase tracking-wider bg-[oklch(0.7_0.15_240/0.2)] text-[oklch(0.8_0.13_240)]"
                        >{{ systemStore.info?.cpu_count || 0 }} cores</span
                    >
                </div>
                <p
                    class="text-[2rem] font-bold tracking-[-0.02em] text-reisa-lilac-400"
                >
                    {{ systemStore.stats?.cpu_usage.toFixed(1) || 0 }}%
                </p>
                <p class="text-sm text-text-muted mt-1">CPU Usage</p>
                <div class="mt-4 h-2 overflow-hidden rounded bg-surface">
                    <div
                        class="h-full rounded bg-linear-to-r from-reisa-lilac-500 to-reisa-pink-500 transition-[width] duration-300 ease-in-out"
                        :style="{
                            width: (systemStore.stats?.cpu_usage || 0) + '%',
                        }"
                    ></div>
                </div>
            </div>

            <!-- Memory Card -->
            <div
                class="relative overflow-hidden rounded-2xl border border-border-subtle bg-surface-elevated p-6 before:absolute before:inset-x-0 before:top-0 before:h-0.75 before:bg-linear-to-r before:from-reisa-lilac-500 before:via-reisa-pink-500 before:to-reisa-gold-500"
            >
                <div class="flex items-center justify-between mb-4">
                    <div
                        class="w-10 h-10 rounded-xl bg-reisa-pink-500/20 flex items-center justify-center"
                    >
                        <svg
                            class="w-5 h-5 text-reisa-pink-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                            />
                        </svg>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full px-3 py-1 text-xs font-medium uppercase tracking-wider bg-[oklch(0.7_0.15_240/0.2)] text-[oklch(0.8_0.13_240)]"
                        >{{
                            formatBytes(systemStore.stats?.memory_total || 0)
                        }}</span
                    >
                </div>
                <p
                    class="text-[2rem] font-bold tracking-[-0.02em] text-reisa-pink-400"
                >
                    {{ systemStore.stats?.memory_percent.toFixed(1) || 0 }}%
                </p>
                <p class="text-sm text-text-muted mt-1">
                    Memory:
                    {{ formatBytes(systemStore.stats?.memory_used || 0) }} used
                </p>
                <div class="mt-4 h-2 overflow-hidden rounded bg-surface">
                    <div
                        class="h-full rounded transition-[width] duration-300 ease-in-out"
                        :class="
                            (systemStore.stats?.memory_percent || 0) < 60
                                ? 'bg-linear-to-r from-[oklch(0.65_0.16_142)] to-success'
                                : (systemStore.stats?.memory_percent || 0) < 85
                                  ? 'bg-linear-to-r from-[oklch(0.75_0.16_85)] to-warning'
                                  : 'bg-linear-to-r from-[oklch(0.55_0.18_25)] to-error'
                        "
                        :style="{
                            width:
                                (systemStore.stats?.memory_percent || 0) + '%',
                        }"
                    ></div>
                </div>
            </div>

            <!-- Load Average Card -->
            <div
                class="relative overflow-hidden rounded-2xl border border-border-subtle bg-surface-elevated p-6 before:absolute before:inset-x-0 before:top-0 before:h-0.75 before:bg-linear-to-r before:from-reisa-lilac-500 before:via-reisa-pink-500 before:to-reisa-gold-500"
            >
                <div class="flex items-center justify-between mb-4">
                    <div
                        class="w-10 h-10 rounded-xl bg-reisa-gold-500/20 flex items-center justify-center"
                    >
                        <svg
                            class="w-5 h-5 text-reisa-gold-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M13 10V3L4 14h7v7l9-11h-7z"
                            />
                        </svg>
                    </div>
                </div>
                <p
                    class="text-[2rem] font-bold tracking-[-0.02em] text-reisa-gold-400"
                >
                    {{ systemStore.stats?.load_avg[0].toFixed(2) || '0.00' }}
                </p>
                <p class="text-sm text-text-muted mt-1">Load Average (1m)</p>
                <div
                    class="mt-4 flex items-center gap-4 text-xs text-text-muted"
                >
                    <span
                        >5m:
                        {{
                            systemStore.stats?.load_avg[1].toFixed(2) || '0.00'
                        }}</span
                    >
                    <span
                        >15m:
                        {{
                            systemStore.stats?.load_avg[2].toFixed(2) || '0.00'
                        }}</span
                    >
                </div>
            </div>

            <!-- Uptime Card -->
            <div
                class="relative overflow-hidden rounded-2xl border border-border-subtle bg-surface-elevated p-6 before:absolute before:inset-x-0 before:top-0 before:h-0.75 before:bg-linear-to-r before:from-reisa-lilac-500 before:via-reisa-pink-500 before:to-reisa-gold-500"
            >
                <div class="flex items-center justify-between mb-4">
                    <div
                        class="w-10 h-10 rounded-xl bg-reisa-stripe-500/20 flex items-center justify-center"
                    >
                        <svg
                            class="w-5 h-5 text-reisa-stripe-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                        </svg>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full px-3 py-1 text-xs font-medium uppercase tracking-wider bg-[oklch(0.72_0.18_142/0.2)] text-[oklch(0.8_0.16_142)]"
                        >Online</span
                    >
                </div>
                <p
                    class="text-[2rem] font-bold tracking-[-0.02em] text-reisa-stripe-400"
                >
                    {{ formatUptime(systemStore.info?.uptime || 0) }}
                </p>
                <p class="text-sm text-text-muted mt-1">Uptime</p>
                <div class="mt-4 text-xs text-text-muted">
                    <span
                        >{{ systemStore.info?.os }}
                        {{ systemStore.info?.os_version }}</span
                    >
                </div>
            </div>
        </div>

        <!-- Charts Row -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- CPU Chart -->
            <div
                class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
            >
                <div class="flex items-center justify-between mb-4">
                    <h3 class="font-semibold text-text-primary">CPU Usage</h3>
                    <span class="text-sm text-reisa-lilac-400"
                        >{{ systemStore.stats?.cpu_usage.toFixed(1) }}%</span
                    >
                </div>
                <div class="h-48">
                    <Line :data="cpuChartData" :options="chartOptions" />
                </div>
            </div>

            <!-- Memory Chart -->
            <div
                class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
            >
                <div class="flex items-center justify-between mb-4">
                    <h3 class="font-semibold text-text-primary">
                        Memory Usage
                    </h3>
                    <span class="text-sm text-reisa-pink-400"
                        >{{
                            systemStore.stats?.memory_percent.toFixed(1)
                        }}%</span
                    >
                </div>
                <div class="h-48">
                    <Line :data="memoryChartData" :options="chartOptions" />
                </div>
            </div>
        </div>

        <!-- Disk & Network Row -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- Disk Usage -->
            <div
                class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
            >
                <h3 class="font-semibold text-text-primary mb-4">Disk Usage</h3>
                <div class="space-y-4">
                    <div
                        v-for="disk in systemStore.stats?.disks || []"
                        :key="disk.name"
                        class="space-y-2"
                    >
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-text-secondary">{{
                                disk.mount_point
                            }}</span>
                            <span class="text-text-muted">
                                {{ formatBytes(disk.used) }} /
                                {{ formatBytes(disk.total) }}
                            </span>
                        </div>
                        <div class="h-2 overflow-hidden rounded bg-surface">
                            <div
                                class="h-full rounded transition-[width] duration-300 ease-in-out"
                                :class="
                                    disk.percent < 70
                                        ? 'bg-linear-to-r from-[oklch(0.65_0.16_142)] to-success'
                                        : disk.percent < 90
                                          ? 'bg-linear-to-r from-[oklch(0.75_0.16_85)] to-warning'
                                          : 'bg-linear-to-r from-[oklch(0.55_0.18_25)] to-error'
                                "
                                :style="{ width: disk.percent + '%' }"
                            ></div>
                        </div>
                    </div>
                    <p
                        v-if="!systemStore.stats?.disks?.length"
                        class="text-text-muted text-sm"
                    >
                        No disk information available
                    </p>
                </div>
            </div>

            <!-- Network -->
            <div
                class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
            >
                <h3 class="font-semibold text-text-primary mb-4">
                    Network Interfaces
                </h3>
                <div class="space-y-3">
                    <div
                        v-for="net in systemStore.stats?.networks || []"
                        :key="net.name"
                        class="flex items-center justify-between p-3 rounded-lg bg-surface"
                    >
                        <div class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg bg-reisa-stripe-500/20 flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-reisa-stripe-400"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0"
                                    />
                                </svg>
                            </div>
                            <span
                                class="text-sm font-medium text-text-primary"
                                >{{ net.name }}</span
                            >
                        </div>
                        <div class="flex items-center gap-4 text-xs">
                            <div class="text-right">
                                <p class="text-success">
                                    ↓ {{ formatBytes(net.rx_bytes) }}
                                </p>
                            </div>
                            <div class="text-right">
                                <p class="text-reisa-pink-400">
                                    ↑ {{ formatBytes(net.tx_bytes) }}
                                </p>
                            </div>
                        </div>
                    </div>
                    <p
                        v-if="!systemStore.stats?.networks?.length"
                        class="text-text-muted text-sm"
                    >
                        No network information available
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useSystemStore } from '@/stores/system'
import { Line } from 'vue-chartjs'
import {
    Chart as ChartJS,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
    Filler,
} from 'chart.js'

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
    Filler,
)

const systemStore = useSystemStore()

onMounted(() => {
    systemStore.fetchStats()
})

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const mins = Math.floor((seconds % 3600) / 60)
    return `${days}d ${hours}h ${mins}m`
}

const cpuChartData = computed(() => ({
    labels: systemStore.statsHistory.map(() => ''),
    datasets: [
        {
            label: 'CPU Usage',
            data: systemStore.statsHistory.map((s) => s.cpu_usage),
            borderColor: 'oklch(0.66 0.058 301)',
            backgroundColor: 'oklch(0.66 0.058 301 / 0.1)',
            fill: true,
            tension: 0.4,
            pointRadius: 0,
        },
    ],
}))

const memoryChartData = computed(() => ({
    labels: systemStore.statsHistory.map(() => ''),
    datasets: [
        {
            label: 'Memory Usage',
            data: systemStore.statsHistory.map((s) => s.memory_percent),
            borderColor: 'oklch(0.65 0.210 356)',
            backgroundColor: 'oklch(0.65 0.210 356 / 0.1)',
            fill: true,
            tension: 0.4,
            pointRadius: 0,
        },
    ],
}))

const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
        legend: { display: false },
        tooltip: {
            backgroundColor: 'oklch(0.18 0.018 280)',
            titleColor: 'oklch(0.95 0.01 280)',
            bodyColor: 'oklch(0.75 0.015 280)',
            borderColor: 'oklch(0.28 0.025 280)',
            borderWidth: 1,
        },
    },
    scales: {
        x: {
            display: false,
        },
        y: {
            min: 0,
            max: 100,
            ticks: {
                color: 'oklch(0.55 0.012 280)',
                callback: (value: string | number) => value + '%',
            },
            grid: {
                color: 'oklch(0.22 0.020 280)',
            },
        },
    },
}
</script>
