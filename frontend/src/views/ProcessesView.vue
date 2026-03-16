<template>
    <div class="p-6 space-y-6 animate-in">
        <ListToolbar
            v-model:searchQuery="searchQuery"
            v-model:filterValue="statusFilter"
            :filterOptions="statusFilterOptions"
            :filteredCount="filteredCount"
            :totalCount="totalCount"
            itemLabel="processes"
            @search="debouncedSearch"
        >
            <template #actions>
                <!-- Resume: visible when nothing selected or process is stopped -->
                <BaseButton
                    v-if="!selectedProcess || selectedProcess.status === 'Stop'"
                    :disabled="!selectedProcess"
                    @click="
                        selectedProcess && resumeProcess(selectedProcess.pid)
                    "
                    class="hover:bg-success/20 text-success"
                    variant="square"
                    title="Resume Process"
                >
                    <svg
                        class="w-6 h-6"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                        />
                    </svg>
                </BaseButton>
                <!-- Stop: visible only when a non-stopped process is selected -->
                <BaseButton
                    v-if="selectedProcess && selectedProcess.status !== 'Stop'"
                    @click="stopProcess(selectedProcess.pid)"
                    class="hover:bg-warning/20 text-warning"
                    variant="square"
                    title="Stop Process"
                >
                    <svg
                        class="w-6 h-6"
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
                </BaseButton>
                <BaseButton
                    :disabled="!selectedProcess"
                    @click="
                        selectedProcess && confirmKillProcess(selectedProcess)
                    "
                    class="hover:bg-error/20 text-error"
                    variant="square"
                    title="Kill Process"
                >
                    <svg
                        class="w-6 h-6"
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
                </BaseButton>
            </template>
        </ListToolbar>

        <DataTable
            :columns="tableColumns"
            :loading="loading"
            :empty="displayedProcesses.length === 0"
            emptyText="No processes found matching your criteria"
            :sortBy="sortBy"
            :sortOrder="sortOrder"
            @sort="toggleSort"
        >
            <template #rows>
                <tr
                    v-for="process in displayedProcesses"
                    :key="process.pid"
                    class="cursor-pointer transition-colors"
                    :class="
                        selectedPid === process.pid
                            ? 'bg-reisa-lilac-500/10'
                            : 'hover:bg-bg-tertiary'
                    "
                    @click="
                        selectedPid =
                            selectedPid === process.pid ? null : process.pid
                    "
                >
                    <td class="font-mono text-sm">{{ process.pid }}</td>
                    <td>
                        <div class="max-w-xs">
                            <div class="font-medium text-text-primary truncate">
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
                                :class="getCpuColorClass(process.cpu_usage)"
                            >
                                {{ process.cpu_usage.toFixed(1) }}%
                            </span>
                            <div
                                class="w-16 h-1.5 bg-bg-tertiary rounded-full overflow-hidden"
                            >
                                <div
                                    class="h-full rounded-full transition-all"
                                    :class="getCpuBarClass(process.cpu_usage)"
                                    :style="{
                                        width:
                                            Math.min(process.cpu_usage, 100) +
                                            '%',
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
                </tr>
            </template>
        </DataTable>

        <div
            v-if="displayedProcesses.length > 0"
            class="px-6 py-3 border-t border-border flex items-center justify-between text-sm text-text-muted"
        >
            <div>Last updated: {{ lastUpdateTime }}</div>
            <div>Showing {{ displayedProcesses.length }} processes</div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { api } from '@/api'
import { useConnectionStore } from '@/stores/connection'
import ListToolbar from '@/components/universal/ListToolbar.vue'
import DataTable from '@/components/universal/DataTable.vue'
import BaseButton from '@/components/universal/BaseButton.vue'

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
const tableColumns = [
    { key: 'pid', label: 'PID', sortable: true },
    { key: 'name', label: 'Name', sortable: true },
    { key: 'cpu', label: 'CPU', sortable: true },
    { key: 'memory', label: 'Memory', sortable: true },
    { key: 'status', label: 'Status' },
    { key: 'user', label: 'User' },
]

const selectedPid = ref<number | null>(null)
const selectedProcess = computed(
    () => processes.value.find((p) => p.pid === selectedPid.value) ?? null,
)

const statusFilterOptions = [
    { value: 'all', label: 'All Status' },
    { value: 'Run', label: 'Running' },
    { value: 'Sleep', label: 'Sleeping' },
    { value: 'Stop', label: 'Stopped' },
    { value: 'Zombie', label: 'Zombie' },
]
const sortableFields = ['cpu', 'memory', 'name', 'pid'] as const
type SortField = (typeof sortableFields)[number]
const sortBy = ref<SortField>('cpu')
const sortOrder = ref<'asc' | 'desc'>('desc')
const connectionStore = useConnectionStore()
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

const isSortField = (field: string): field is SortField => {
    return (sortableFields as readonly string[]).includes(field)
}

const toggleSort = (field: string) => {
    if (!isSortField(field)) return

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
        Run: 'badge-success',
        Sleep: 'badge-info',
        Stop: 'badge-warning',
        Zombie: 'badge-error',
        Idle: 'badge-secondary',
    }
    return statusMap[status] || 'badge-secondary'
}

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

watch(displayedProcesses, (list) => {
    if (
        selectedPid.value !== null &&
        !list.some((p) => p.pid === selectedPid.value)
    ) {
        selectedPid.value = null
    }
})

watch(
    sseConnected,
    (v) => connectionStore.set(v ? 'connected' : 'connecting'),
    {
        immediate: true,
    },
)

onMounted(() => {
    isUnmounted.value = false
    fetchProcesses()
    connectProcessStream()
})

onUnmounted(() => {
    connectionStore.clear()
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
