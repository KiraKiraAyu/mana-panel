<template>
    <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="$emit('close')"
    >
        <div
            class="flex items-center justify-between p-4 border-b border-border-subtle"
        >
            <div>
                <h3 class="font-semibold text-text-primary">
                    Stats: {{ containerName }}
                </h3>
                <p class="text-xs text-text-muted font-mono mt-0.5">
                    {{ containerId }}
                </p>
            </div>
            <div class="flex items-center gap-2">
                <button
                    @click="fetchStats"
                    class="btn btn-ghost text-xs"
                    :disabled="loading"
                >
                    <svg
                        class="w-3.5 h-3.5"
                        :class="{ 'animate-spin': loading }"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                        />
                    </svg>
                    Refresh
                </button>
                <button @click="$emit('close')" class="btn btn-ghost text-xs">
                    Close
                </button>
            </div>
        </div>

        <!-- Stats Content -->
        <div class="p-6 space-y-6" v-if="stats">
            <!-- CPU -->
            <div>
                <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium text-text-secondary"
                        >CPU Usage</span
                    >
                    <span class="text-sm font-mono text-reisa-lilac-400">
                        {{ stats.cpu_percent.toFixed(2) }}%
                    </span>
                </div>
                <div class="progress">
                    <div
                        class="progress-bar"
                        :style="{
                            width: Math.min(stats.cpu_percent, 100) + '%',
                        }"
                    ></div>
                </div>
            </div>

            <!-- Memory -->
            <div>
                <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium text-text-secondary"
                        >Memory Usage</span
                    >
                    <span class="text-sm font-mono text-reisa-pink-400">
                        {{ formatBytes(stats.memory_usage) }} /
                        {{ formatBytes(stats.memory_limit) }}
                        ({{ stats.memory_percent.toFixed(1) }}%)
                    </span>
                </div>
                <div class="progress">
                    <div
                        class="progress-bar"
                        :class="{
                            success: stats.memory_percent < 60,
                            warning:
                                stats.memory_percent >= 60 &&
                                stats.memory_percent < 85,
                            danger: stats.memory_percent >= 85,
                        }"
                        :style="{ width: stats.memory_percent + '%' }"
                    ></div>
                </div>
            </div>

            <!-- Network & Block I/O -->
            <div class="grid grid-cols-2 gap-4">
                <div class="p-4 rounded-lg bg-surface">
                    <p class="text-xs text-text-muted mb-1">Network RX</p>
                    <p class="text-lg font-semibold text-success">
                        &darr; {{ formatBytes(stats.network_rx) }}
                    </p>
                </div>
                <div class="p-4 rounded-lg bg-surface">
                    <p class="text-xs text-text-muted mb-1">Network TX</p>
                    <p class="text-lg font-semibold text-reisa-pink-400">
                        &uarr; {{ formatBytes(stats.network_tx) }}
                    </p>
                </div>
                <div class="p-4 rounded-lg bg-surface">
                    <p class="text-xs text-text-muted mb-1">Block Read</p>
                    <p class="text-lg font-semibold text-reisa-stripe-400">
                        {{ formatBytes(stats.block_read) }}
                    </p>
                </div>
                <div class="p-4 rounded-lg bg-surface">
                    <p class="text-xs text-text-muted mb-1">Block Write</p>
                    <p class="text-lg font-semibold text-reisa-gold-400">
                        {{ formatBytes(stats.block_write) }}
                    </p>
                </div>
            </div>

            <!-- PIDs -->
            <div
                class="w-full max-w-2xl bg-surface-elevated rounded-xl shadow-2xl animate-in"
            >
                <div
                    class="flex items-center justify-between p-4 border-b border-border-subtle"
                >
                    <div>
                        <h3 class="font-semibold text-text-primary">
                            Stats: {{ containerName }}
                        </h3>
                        <p class="text-xs text-text-muted font-mono mt-0.5">
                            {{ containerId }}
                        </p>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            @click="fetchStats"
                            class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
                            :disabled="loading"
                        >
                            <svg
                                class="w-3.5 h-3.5"
                                :class="{ 'animate-spin': loading }"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                />
                            </svg>
                            Refresh
                        </button>
                        <button
                            @click="$emit('close')"
                            class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
                        >
                            Close
                        </button>
                    </div>
                </div>

                <!-- Stats Content -->
                <div class="p-6 space-y-6" v-if="stats">
                    <!-- CPU -->
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <span
                                class="text-sm font-medium text-text-secondary"
                                >CPU Usage</span
                            >
                            <span
                                class="text-sm font-mono text-reisa-lilac-400"
                            >
                                {{ stats.cpu_percent.toFixed(2) }}%
                            </span>
                        </div>
                        <div class="h-2 overflow-hidden rounded bg-surface">
                            <div
                                class="h-full rounded bg-linear-to-r from-reisa-lilac-500 to-reisa-pink-500 transition-[width] duration-300 ease-in-out"
                                :style="{
                                    width:
                                        Math.min(stats.cpu_percent, 100) + '%',
                                }"
                            ></div>
                        </div>
                    </div>

                    <!-- Memory -->
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <span
                                class="text-sm font-medium text-text-secondary"
                                >Memory Usage</span
                            >
                            <span class="text-sm font-mono text-reisa-pink-400">
                                {{ formatBytes(stats.memory_usage) }} /
                                {{ formatBytes(stats.memory_limit) }}
                                ({{ stats.memory_percent.toFixed(1) }}%)
                            </span>
                        </div>
                        <div class="h-2 overflow-hidden rounded bg-surface">
                            <div
                                class="h-full rounded transition-[width] duration-300 ease-in-out"
                                :class="
                                    stats.memory_percent < 60
                                        ? 'bg-linear-to-r from-[oklch(0.65_0.16_142)] to-success'
                                        : stats.memory_percent < 85
                                          ? 'bg-linear-to-r from-[oklch(0.75_0.16_85)] to-warning'
                                          : 'bg-linear-to-r from-[oklch(0.55_0.18_25)] to-error'
                                "
                                :style="{ width: stats.memory_percent + '%' }"
                            ></div>
                        </div>
                    </div>

                    <!-- Network & Block I/O -->
                    <div class="grid grid-cols-2 gap-4">
                        <div class="p-4 rounded-lg bg-surface">
                            <p class="text-xs text-text-muted mb-1">
                                Network RX
                            </p>
                            <p class="text-lg font-semibold text-success">
                                &darr; {{ formatBytes(stats.network_rx) }}
                            </p>
                        </div>
                        <div class="p-4 rounded-lg bg-surface">
                            <p class="text-xs text-text-muted mb-1">
                                Network TX
                            </p>
                            <p
                                class="text-lg font-semibold text-reisa-pink-400"
                            >
                                &uarr; {{ formatBytes(stats.network_tx) }}
                            </p>
                        </div>
                        <div class="p-4 rounded-lg bg-surface">
                            <p class="text-xs text-text-muted mb-1">
                                Block Read
                            </p>
                            <p
                                class="text-lg font-semibold text-reisa-stripe-400"
                            >
                                {{ formatBytes(stats.block_read) }}
                            </p>
                        </div>
                        <div class="p-4 rounded-lg bg-surface">
                            <p class="text-xs text-text-muted mb-1">
                                Block Write
                            </p>
                            <p
                                class="text-lg font-semibold text-reisa-gold-400"
                            >
                                {{ formatBytes(stats.block_write) }}
                            </p>
                        </div>
                    </div>

                    <!-- PIDs -->
                    <div
                        class="flex items-center justify-between p-4 rounded-lg bg-surface"
                    >
                        <span class="text-sm text-text-secondary">PIDs</span>
                        <span class="text-sm font-mono text-text-primary">
                            {{ stats.pids }}
                        </span>
                    </div>
                </div>

                <!-- Loading State -->
                <div v-else-if="loading" class="p-12 text-center">
                    <svg
                        class="w-8 h-8 animate-spin mx-auto mb-4 text-reisa-lilac-400"
                        fill="none"
                        viewBox="0 0 24 24"
                    >
                        <circle
                            class="opacity-25"
                            cx="12"
                            cy="12"
                            r="10"
                            stroke="currentColor"
                            stroke-width="4"
                        ></circle>
                        <path
                            class="opacity-75"
                            fill="currentColor"
                            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                        ></path>
                    </svg>
                    <p class="text-text-muted text-sm">Loading stats...</p>
                </div>

                <!-- Error State -->
                <div v-else-if="error" class="p-8 text-center">
                    <p class="text-error text-sm mb-4">{{ error }}</p>
                    <button
                        @click="fetchStats"
                        class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
                    >
                        Retry
                    </button>
                </div>
            </div>
        </div>

        <!-- Loading State -->
        <div v-else-if="loading" class="p-12 text-center">
            <svg
                class="w-8 h-8 animate-spin mx-auto mb-4 text-reisa-lilac-400"
                fill="none"
                viewBox="0 0 24 24"
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                ></circle>
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
            </svg>
            <p class="text-text-muted text-sm">Loading stats...</p>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="p-8 text-center">
            <p class="text-error text-sm mb-4">{{ error }}</p>
            <button @click="fetchStats" class="btn btn-ghost text-xs">
                Retry
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { api } from '@/api'

interface ContainerStatsData {
    cpu_percent: number
    memory_usage: number
    memory_limit: number
    memory_percent: number
    network_rx: number
    network_tx: number
    block_read: number
    block_write: number
    pids: number
}

const props = defineProps<{
    containerId: string
    containerName: string
}>()

defineEmits<{
    (e: 'close'): void
}>()

const stats = ref<ContainerStatsData | null>(null)
const loading = ref(false)
const error = ref('')
let refreshInterval: ReturnType<typeof setInterval> | null = null

const fetchStats = async () => {
    loading.value = true
    error.value = ''
    try {
        const response = await api.get(
            `/docker/containers/${props.containerId}/stats`,
        )
        stats.value = response.data
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to fetch stats'
    } finally {
        loading.value = false
    }
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

onMounted(() => {
    fetchStats()
    // Auto-refresh every 3 seconds
    refreshInterval = setInterval(fetchStats, 3000)
})

onUnmounted(() => {
    if (refreshInterval) {
        clearInterval(refreshInterval)
        refreshInterval = null
    }
})
</script>
