<template>
    <div class="p-6 space-y-6 animate-in">
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-text-primary">Processes</h1>
                <p class="text-text-muted mt-1">
                    Manage running system processes
                </p>
            </div>
            <div class="flex items-center gap-2">
                <span
                    class="text-xs"
                    :class="sseConnected ? 'text-success' : 'text-warning'"
                >
                    Stream: {{ sseConnected ? 'Connected' : 'Reconnecting...' }}
                </span>
                <button
                    @click="reconnectStream"
                    class="btn btn-ghost btn-sm"
                    :disabled="loading"
                >
                    Reconnect
                </button>
                <button
                    @click="refreshNow"
                    class="btn btn-ghost"
                    :disabled="loading"
                >
                    <svg
                        class="w-4 h-4"
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
            </div>
        </div>

        <!-- Search & Filters -->
        <div
            class="flex flex-col sm:flex-row items-stretch sm:items-center gap-4"
        >
            <div class="flex-1 relative">
                <svg
                    class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-text-muted"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>
                <input
                    v-model="searchQuery"
                    @input="debouncedSearch"
                    type="text"
                    class="input pl-10"
                    placeholder="Search by process name or PID..."
                />
                <button
                    v-if="searchQuery"
                    @click="clearSearch"
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary"
                >
                    <svg
                        class="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            <div class="flex items-center gap-2">
                <select v-model="statusFilter" class="input">
                    <option value="all">All Status</option>
                    <option value="Run">Running</option>
                    <option value="Sleep">Sleeping</option>
                    <option value="Stop">Stopped</option>
                    <option value="Zombie">Zombie</option>
                </select>

                <span class="text-sm text-text-muted whitespace-nowrap">
                    {{ filteredCount }} of {{ totalCount }} processes
                </span>
            </div>
        </div>

        <!-- Process Table -->
        <div class="card p-0 overflow-hidden">
            <div class="overflow-x-auto">
                <table class="table">
                    <thead>
                        <tr>
                            <th
                                class="cursor-pointer hover:text-text-primary select-none"
                                @click="toggleSort('pid')"
                            >
                                <div class="flex items-center gap-1">
                                    PID
                                    <svg
                                        v-if="sortBy === 'pid'"
                                        class="w-4 h-4"
                                        :class="{
                                            'rotate-180': sortOrder === 'desc',
                                        }"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M5 15l7-7 7 7"
                                        />
                                    </svg>
                                </div>
                            </th>
                            <th
                                class="cursor-pointer hover:text-text-primary select-none"
                                @click="toggleSort('name')"
                            >
                                <div class="flex items-center gap-1">
                                    Name
                                    <svg
                                        v-if="sortBy === 'name'"
                                        class="w-4 h-4"
                                        :class="{
                                            'rotate-180': sortOrder === 'desc',
                                        }"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M5 15l7-7 7 7"
                                        />
                                    </svg>
                                </div>
                            </th>
                            <th
                                class="cursor-pointer hover:text-text-primary select-none"
                                @click="toggleSort('cpu')"
                            >
                                <div class="flex items-center gap-1">
                                    CPU
                                    <svg
                                        v-if="sortBy === 'cpu'"
                                        class="w-4 h-4"
                                        :class="{
                                            'rotate-180': sortOrder === 'desc',
                                        }"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M5 15l7-7 7 7"
                                        />
                                    </svg>
                                </div>
                            </th>
                            <th
                                class="cursor-pointer hover:text-text-primary select-none"
                                @click="toggleSort('memory')"
                            >
                                <div class="flex items-center gap-1">
                                    Memory
                                    <svg
                                        v-if="sortBy === 'memory'"
                                        class="w-4 h-4"
                                        :class="{
                                            'rotate-180': sortOrder === 'desc',
                                        }"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M5 15l7-7 7 7"
                                        />
                                    </svg>
                                </div>
                            </th>
                            <th>Status</th>
                            <th>User</th>
                            <th class="text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            v-for="process in displayedProcesses"
                            :key="process.pid"
                            class="hover:bg-bg-tertiary transition-colors"
                        >
                            <td class="font-mono text-sm">{{ process.pid }}</td>
                            <td>
                                <div class="max-w-xs">
                                    <div
                                        class="font-medium text-text-primary truncate"
                                    >
                                        {{ process.name }}
                                    </div>
                                    <div
                                        v-if="process.cmd.length > 0"
                                        class="text-xs text-text-muted font-mono truncate"
                                        :title="process.cmd.join(' ')"
                                    >
                                        {{ getCommandPreview(process.cmd) }}
                                    </div>
                                </div>
                            </td>
                            <td>
                                <div class="flex items-center gap-2">
                                    <span
                                        class="font-mono text-sm"
                                        :class="
                                            getCpuColorClass(process.cpu_usage)
                                        "
                                    >
                                        {{ process.cpu_usage.toFixed(1) }}%
                                    </span>
                                    <div
                                        class="w-16 h-1.5 bg-bg-tertiary rounded-full overflow-hidden"
                                    >
                                        <div
                                            class="h-full rounded-full transition-all"
                                            :class="
                                                getCpuBarClass(
                                                    process.cpu_usage,
                                                )
                                            "
                                            :style="{
                                                width:
                                                    Math.min(
                                                        process.cpu_usage,
                                                        100,
                                                    ) + '%',
                                            }"
                                        ></div>
                                    </div>
                                </div>
                            </td>
                            <td class="font-mono text-sm">
                                {{ formatBytes(process.memory) }}
                            </td>
                            <td>
                                <span
                                    class="badge text-xs"
                                    :class="getStatusBadgeClass(process.status)"
                                >
                                    {{ process.status }}
                                </span>
                            </td>
                            <td class="text-sm text-text-secondary">
                                {{ process.user }}
                            </td>
                            <td>
                                <div
                                    class="flex items-center justify-end gap-1"
                                >
                                    <button
                                        v-if="process.status === 'Stop'"
                                        @click="resumeProcess(process.pid)"
                                        class="p-1.5 rounded-lg hover:bg-success/20 text-success transition-colors"
                                        title="Resume Process"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                                            />
                                        </svg>
                                    </button>
                                    <button
                                        v-else
                                        @click="stopProcess(process.pid)"
                                        class="p-1.5 rounded-lg hover:bg-warning/20 text-warning transition-colors"
                                        title="Stop Process"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
                                            />
                                        </svg>
                                    </button>
                                    <button
                                        @click="confirmKillProcess(process)"
                                        class="p-1.5 rounded-lg hover:bg-error/20 text-error transition-colors"
                                        title="Kill Process"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M6 18L18 6M6 6l12 12"
                                            />
                                        </svg>
                                    </button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div
                    v-if="loading && processes.length === 0"
                    class="p-8 text-center"
                >
                    <div class="shimmer h-8 w-48 mx-auto rounded mb-4"></div>
                    <div class="shimmer h-4 w-32 mx-auto rounded"></div>
                </div>

                <div
                    v-if="!loading && displayedProcesses.length === 0"
                    class="p-8 text-center text-text-muted"
                >
                    <svg
                        class="w-12 h-12 mx-auto mb-2 opacity-50"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                    </svg>
                    <p>No processes found matching your criteria</p>
                </div>
            </div>

            <!-- Pagination (if needed in the future) -->
            <div
                v-if="displayedProcesses.length > 0"
                class="px-6 py-3 border-t border-border flex items-center justify-between text-sm text-text-muted"
            >
                <div>Last updated: {{ lastUpdateTime }}</div>
                <div>Showing {{ displayedProcesses.length }} processes</div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { api } from '@/api'

interface Process {
    pid: number
    name: string
    cmd: string[]
    cpu_usage: number
    memory: number
    status: string
    user: string
    start_time: number
}

const processes = ref<Process[]>([])
const loading = ref(false)
const searchQuery = ref('')
const statusFilter = ref('all')
const sortBy = ref<'cpu' | 'memory' | 'name' | 'pid'>('cpu')
const sortOrder = ref<'asc' | 'desc'>('desc')
const sseConnected = ref(false)
const reconnectAttempts = ref(0)
const lastUpdateTime = ref('')

let eventSource: EventSource | null = null
let reconnectTimer: number | null = null
let searchTimeout: number | null = null
let streamSessionId = 0
const isUnmounted = ref(false)

const totalCount = computed(() => processes.value.length)

const displayedProcesses = computed(() => {
    let filtered = processes.value

    // Filter by status
    if (statusFilter.value !== 'all') {
        filtered = filtered.filter((p) => p.status === statusFilter.value)
    }

    return filtered
})

const filteredCount = computed(() => displayedProcesses.value.length)

const fetchProcesses = async () => {
    loading.value = true
    try {
        const response = await api.get('/processes', {
            params: {
                sort_by: sortBy.value,
                order: sortOrder.value,
                search: searchQuery.value || undefined,
            },
        })
        processes.value = response.data
        lastUpdateTime.value = new Date().toLocaleTimeString()
    } catch (e) {
        console.error('Failed to fetch processes:', e)
    } finally {
        loading.value = false
    }
}

const buildStreamUrl = (): string => {
    const params = new URLSearchParams()
    params.set('sort_by', sortBy.value)
    params.set('order', sortOrder.value)
    if (searchQuery.value.trim()) {
        params.set('search', searchQuery.value.trim())
    }
    return `/api/processes/stream?${params.toString()}`
}

const clearReconnectTimer = () => {
    if (reconnectTimer !== null) {
        clearTimeout(reconnectTimer)
        reconnectTimer = null
    }
}

const disconnectProcessStream = () => {
    const current = eventSource
    if (current) {
        current.onopen = null
        current.onmessage = null
        current.onerror = null
        current.close()
        if (eventSource === current) {
            eventSource = null
        }
    }
    sseConnected.value = false
}

const scheduleReconnect = () => {
    if (isUnmounted.value) return

    clearReconnectTimer()
    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value), 30000)
    reconnectTimer = window.setTimeout(() => {
        reconnectTimer = null
        if (isUnmounted.value) return
        reconnectAttempts.value += 1
        connectProcessStream()
    }, delay)
}

const connectProcessStream = () => {
    if (isUnmounted.value) return

    disconnectProcessStream()
    clearReconnectTimer()

    const url = buildStreamUrl()
    const sessionId = ++streamSessionId
    const source = new EventSource(url)
    eventSource = source

    source.onopen = () => {
        if (
            isUnmounted.value ||
            eventSource !== source ||
            sessionId !== streamSessionId
        ) {
            return
        }
        sseConnected.value = true
        reconnectAttempts.value = 0
    }

    source.onmessage = (event) => {
        if (
            isUnmounted.value ||
            eventSource !== source ||
            sessionId !== streamSessionId
        ) {
            return
        }

        try {
            const data = JSON.parse(event.data) as Process[]
            processes.value = data
            lastUpdateTime.value = new Date().toLocaleTimeString()
            loading.value = false
        } catch (e) {
            console.error('Failed to parse process stream message:', e)
        }
    }

    source.onerror = () => {
        if (
            isUnmounted.value ||
            eventSource !== source ||
            sessionId !== streamSessionId
        ) {
            return
        }
        disconnectProcessStream()
        scheduleReconnect()
    }
}

const restartProcessStream = () => {
    connectProcessStream()
}

const reconnectStream = () => {
    reconnectAttempts.value = 0
    connectProcessStream()
}

const refreshNow = async () => {
    await fetchProcesses()
    reconnectStream()
}

const killProcess = async (pid: number) => {
    try {
        await api.post(`/processes/${pid}/kill`)
        await fetchProcesses()
    } catch (e: any) {
        const errorMsg =
            e.response?.data?.error?.message || 'Failed to kill process'
        alert(errorMsg)
    }
}

const confirmKillProcess = (process: Process) => {
    if (
        confirm(
            `Are you sure you want to kill process "${process.name}" (PID: ${process.pid})?\n\nThis action cannot be undone.`,
        )
    ) {
        killProcess(process.pid)
    }
}

const stopProcess = async (pid: number) => {
    try {
        await api.post(`/processes/${pid}/stop`)
        await fetchProcesses()
    } catch (e: any) {
        const errorMsg =
            e.response?.data?.error?.message || 'Failed to stop process'
        alert(errorMsg)
    }
}

const resumeProcess = async (pid: number) => {
    try {
        await api.post(`/processes/${pid}/resume`)
        await fetchProcesses()
    } catch (e: any) {
        const errorMsg =
            e.response?.data?.error?.message || 'Failed to resume process'
        alert(errorMsg)
    }
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const toggleSort = (field: typeof sortBy.value) => {
    if (sortBy.value === field) {
        sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    } else {
        sortBy.value = field
        sortOrder.value = 'desc'
    }
    fetchProcesses()
    restartProcessStream()
}

const getCpuColorClass = (usage: number): string => {
    if (usage < 30) return 'text-success'
    if (usage < 70) return 'text-warning'
    return 'text-error'
}

const getCpuBarClass = (usage: number): string => {
    if (usage < 30) return 'bg-success'
    if (usage < 70) return 'bg-warning'
    return 'bg-error'
}

const getStatusBadgeClass = (status: string): string => {
    const statusMap: Record<string, string> = {
        Run: "badge-success",
        Sleep: "badge-info",
        Stop: "badge-warning",
        Zombie: "badge-error",
        Idle: "badge-secondary",
    };
    return statusMap[status] || "badge-secondary";
};

const getCommandPreview = (cmd: string[]): string => {
    const fullCmd = cmd.join(' ')
    const maxLength = 60
    if (fullCmd.length <= maxLength) return fullCmd
    return fullCmd.substring(0, maxLength) + '...'
}

const debouncedSearch = () => {
    if (searchTimeout !== null) {
        clearTimeout(searchTimeout)
    }
    searchTimeout = window.setTimeout(() => {
        searchTimeout = null
        if (isUnmounted.value) return
        fetchProcesses()
        restartProcessStream()
    }, 300)
}

const clearSearch = () => {
    searchQuery.value = ''
    fetchProcesses()
    restartProcessStream()
}

onMounted(() => {
    isUnmounted.value = false
    fetchProcesses()
    connectProcessStream()
})

onUnmounted(() => {
    isUnmounted.value = true
    streamSessionId += 1
    disconnectProcessStream()
    clearReconnectTimer()
    if (searchTimeout !== null) {
        clearTimeout(searchTimeout)
        searchTimeout = null
    }
})
</script>

<style scoped>
.animate-in {
    animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.shimmer {
    background: linear-gradient(
        90deg,
        rgba(255, 255, 255, 0.05) 25%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.05) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
    0% {
        background-position: 200% 0;
    }
    100% {
        background-position: -200% 0;
    }
}
</style>
