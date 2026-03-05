<template>
    <div
        class="max-w-7xl mx-auto p-4 md:p-6 space-y-6 animate-in fade-in duration-500"
    >
        <div
            class="flex flex-col md:flex-row md:items-end justify-between gap-4"
        >
            <div>
                <h1 class="text-3xl font-bold text-text-primary tracking-tight">
                    Application Tasks
                </h1>
                <p class="text-text-muted mt-2 text-sm md:text-base max-w-2xl">
                    Real-time install task output and status updates via SSE.
                </p>
            </div>

            <div class="flex items-center gap-2">
                <span
                    class="px-2.5 py-1 rounded-full text-xs border"
                    :class="
                        listConnected
                            ? 'bg-success/10 text-success border-success/30'
                            : 'bg-warning/10 text-warning border-warning/30'
                    "
                >
                    List: {{ listConnected ? 'Connected' : 'Reconnecting' }}
                </span>
                <span
                    class="px-2.5 py-1 rounded-full text-xs border"
                    :class="
                        detailConnected
                            ? 'bg-success/10 text-success border-success/30'
                            : 'bg-warning/10 text-warning border-warning/30'
                    "
                >
                    Output: {{ detailConnected ? 'Connected' : 'Reconnecting' }}
                </span>
                <RouterLink
                    to="/applications"
                    class="btn btn-sm btn-ghost border border-border hover:border-reisa-lilac-500/40 text-text-secondary hover:text-text-primary"
                >
                    Back
                </RouterLink>
            </div>
        </div>

        <div
            v-if="error"
            class="p-3 rounded-xl bg-error/10 border border-error/20 text-error text-sm"
        >
            {{ error }}
        </div>

        <div class="grid grid-cols-1 xl:grid-cols-[340px_1fr] gap-4">
            <section
                class="rounded-2xl border border-border bg-surface/60 backdrop-blur-sm overflow-hidden"
            >
                <div
                    class="px-4 py-3 border-b border-border flex items-center justify-between"
                >
                    <h2 class="text-sm font-semibold text-text-primary">
                        Task Queue
                    </h2>
                    <button
                        class="btn btn-xs btn-ghost text-text-secondary"
                        :disabled="loading"
                        @click="refreshTasks"
                    >
                        Refresh
                    </button>
                </div>

                <div
                    v-if="loading && tasks.length === 0"
                    class="p-6 text-sm text-text-muted"
                >
                    Loading tasks...
                </div>

                <div
                    v-else-if="tasks.length === 0"
                    class="p-6 text-sm text-text-muted"
                >
                    No application tasks yet.
                </div>

                <div v-else class="max-h-[70vh] overflow-auto">
                    <button
                        v-for="task in tasks"
                        :key="task.id"
                        class="w-full text-left px-4 py-3 border-b border-border last:border-b-0 transition-colors"
                        :class="
                            task.id === selectedTaskId
                                ? 'bg-reisa-lilac-500/10'
                                : 'hover:bg-surface-secondary/50'
                        "
                        @click="selectedTaskId = task.id"
                    >
                        <div class="flex items-center justify-between gap-2">
                            <span
                                class="font-mono text-xs text-text-secondary truncate"
                            >
                                {{ task.id.slice(0, 8) }}
                            </span>
                            <span
                                class="px-2 py-0.5 rounded border text-[11px]"
                                :class="statusBadgeClass(task.status)"
                            >
                                {{ task.status }}
                            </span>
                        </div>
                        <div class="mt-1 text-sm text-text-primary truncate">
                            {{ task.requested_name || task.template_id }}
                        </div>
                        <div class="mt-1 text-[11px] text-text-muted">
                            {{ formatTime(task.created_at) }}
                        </div>
                    </button>
                </div>
            </section>

            <section
                class="rounded-2xl border border-border bg-surface/60 backdrop-blur-sm overflow-hidden"
            >
                <div
                    class="px-4 py-3 border-b border-border flex items-center justify-between"
                >
                    <h2 class="text-sm font-semibold text-text-primary">
                        Task Output
                    </h2>
                    <span class="text-xs text-text-muted font-mono">
                        {{ selectedTaskId ? selectedTaskId.slice(0, 8) : '-' }}
                    </span>
                </div>

                <div v-if="!selectedTask" class="p-6 text-sm text-text-muted">
                    Select a task from the left list to view output.
                </div>

                <div v-else class="flex flex-col">
                    <div
                        class="px-4 py-3 border-b border-border space-y-2 text-sm"
                    >
                        <div class="flex flex-wrap items-center gap-2">
                            <span
                                class="px-2 py-0.5 rounded border text-xs"
                                :class="statusBadgeClass(selectedTask.status)"
                            >
                                {{ selectedTask.status }}
                            </span>
                            <span class="text-text-secondary">
                                Template:
                                <span class="font-mono">{{
                                    selectedTask.template_id
                                }}</span>
                            </span>
                            <span
                                v-if="selectedTask.instance_id"
                                class="text-text-secondary"
                            >
                                Instance:
                                <span class="font-mono">
                                    {{ selectedTask.instance_id.slice(0, 8) }}
                                </span>
                            </span>
                        </div>
                        <p class="text-text-muted">
                            {{ selectedTask.summary || 'No summary yet' }}
                        </p>
                    </div>

                    <div
                        ref="logContainerRef"
                        class="h-[58vh] overflow-auto p-4 space-y-2 font-mono text-xs bg-background/70"
                    >
                        <div
                            v-if="selectedTask.logs.length === 0"
                            class="text-text-muted"
                        >
                            No output lines yet.
                        </div>

                        <div
                            v-for="(line, index) in selectedTask.logs"
                            :key="`${selectedTask.id}-${index}-${line.at}`"
                            class="grid grid-cols-[84px_52px_1fr] gap-2 items-start"
                        >
                            <span class="text-text-muted">
                                {{ formatTime(line.at, true) }}
                            </span>
                            <span
                                class="uppercase"
                                :class="logLevelClass(line.level)"
                            >
                                {{ line.level }}
                            </span>
                            <span class="text-text-primary break-words">
                                {{ line.message }}
                            </span>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

import {
    applicationsApi,
    type ApplicationTask,
    type ApplicationTaskLogLevel,
    type ApplicationTaskStatus,
} from '@/api/applications'

const route = useRoute()

const loading = ref(false)
const error = ref('')
const tasks = ref<ApplicationTask[]>([])
const selectedTaskId = ref('')
const selectedTask = ref<ApplicationTask | null>(null)
const listConnected = ref(false)
const detailConnected = ref(false)
const isUnmounted = ref(false)
const logContainerRef = ref<HTMLElement | null>(null)

let listSource: EventSource | null = null
let detailSource: EventSource | null = null
let listReconnectTimer: number | null = null
let detailReconnectTimer: number | null = null
let listReconnectAttempts = 0
let detailReconnectAttempts = 0
let listSessionId = 0
let detailSessionId = 0

const sortTasks = (items: ApplicationTask[]): ApplicationTask[] => {
    const out = [...items]
    out.sort((a, b) => {
        const bt = new Date(b.created_at).getTime()
        const at = new Date(a.created_at).getTime()
        return bt - at
    })
    return out
}

const upsertTask = (task: ApplicationTask) => {
    const idx = tasks.value.findIndex((item) => item.id === task.id)
    if (idx === -1) {
        tasks.value = sortTasks([...tasks.value, task])
        return
    }

    const next = [...tasks.value]
    next[idx] = task
    tasks.value = sortTasks(next)
}

const disconnectListStream = () => {
    if (listSource) {
        listSource.onopen = null
        listSource.onmessage = null
        listSource.onerror = null
        listSource.close()
        listSource = null
    }
    listConnected.value = false
}

const disconnectDetailStream = () => {
    if (detailSource) {
        detailSource.onopen = null
        detailSource.onmessage = null
        detailSource.onerror = null
        detailSource.close()
        detailSource = null
    }
    detailConnected.value = false
}

const clearListReconnectTimer = () => {
    if (listReconnectTimer !== null) {
        window.clearTimeout(listReconnectTimer)
        listReconnectTimer = null
    }
}

const clearDetailReconnectTimer = () => {
    if (detailReconnectTimer !== null) {
        window.clearTimeout(detailReconnectTimer)
        detailReconnectTimer = null
    }
}

const scheduleListReconnect = () => {
    if (isUnmounted.value) return
    clearListReconnectTimer()
    const delay = Math.min(1000 * 2 ** listReconnectAttempts, 15000)
    listReconnectTimer = window.setTimeout(() => {
        listReconnectTimer = null
        listReconnectAttempts += 1
        connectListStream()
    }, delay)
}

const scheduleDetailReconnect = () => {
    if (isUnmounted.value || !selectedTaskId.value) return
    clearDetailReconnectTimer()
    const delay = Math.min(1000 * 2 ** detailReconnectAttempts, 15000)
    detailReconnectTimer = window.setTimeout(() => {
        detailReconnectTimer = null
        detailReconnectAttempts += 1
        connectDetailStream()
    }, delay)
}

const connectListStream = () => {
    if (isUnmounted.value) return
    disconnectListStream()
    clearListReconnectTimer()

    const url = applicationsApi.getTasksStreamUrl()
    const sessionId = ++listSessionId
    const source = new EventSource(url)
    listSource = source

    source.onopen = () => {
        if (
            isUnmounted.value ||
            listSource !== source ||
            sessionId !== listSessionId
        ) {
            return
        }
        listConnected.value = true
        listReconnectAttempts = 0
    }

    source.onmessage = (event) => {
        if (
            isUnmounted.value ||
            listSource !== source ||
            sessionId !== listSessionId
        ) {
            return
        }

        try {
            const incoming = JSON.parse(event.data) as ApplicationTask[]
            tasks.value = sortTasks(incoming)

            if (!selectedTaskId.value && tasks.value.length > 0) {
                selectedTaskId.value = tasks.value[0]?.id || ''
            } else if (
                selectedTaskId.value &&
                !tasks.value.some((task) => task.id === selectedTaskId.value)
            ) {
                selectedTaskId.value = tasks.value[0]?.id || ''
            }
        } catch {
            error.value = 'Failed to parse task list stream payload'
        }
    }

    source.onerror = () => {
        if (
            isUnmounted.value ||
            listSource !== source ||
            sessionId !== listSessionId
        ) {
            return
        }
        disconnectListStream()
        scheduleListReconnect()
    }
}

const connectDetailStream = () => {
    if (isUnmounted.value || !selectedTaskId.value) {
        disconnectDetailStream()
        return
    }

    disconnectDetailStream()
    clearDetailReconnectTimer()

    const taskId = selectedTaskId.value
    const sessionId = ++detailSessionId
    const source = new EventSource(applicationsApi.getTaskStreamUrl(taskId))
    detailSource = source

    source.onopen = () => {
        if (
            isUnmounted.value ||
            detailSource !== source ||
            sessionId !== detailSessionId
        ) {
            return
        }
        detailConnected.value = true
        detailReconnectAttempts = 0
    }

    source.onmessage = (event) => {
        if (
            isUnmounted.value ||
            detailSource !== source ||
            sessionId !== detailSessionId
        ) {
            return
        }

        try {
            const task = JSON.parse(event.data) as ApplicationTask
            if (!task.id) return

            upsertTask(task)
            if (task.id === selectedTaskId.value) {
                selectedTask.value = task
            }
        } catch {
            error.value = 'Failed to parse task output stream payload'
        }
    }

    source.onerror = () => {
        if (
            isUnmounted.value ||
            detailSource !== source ||
            sessionId !== detailSessionId
        ) {
            return
        }
        disconnectDetailStream()
        scheduleDetailReconnect()
    }
}

const refreshTasks = async () => {
    loading.value = true
    error.value = ''

    try {
        const list = await applicationsApi.listTasks()
        tasks.value = sortTasks(list)
        if (!selectedTaskId.value && tasks.value.length > 0) {
            selectedTaskId.value = tasks.value[0]?.id || ''
        }
    } catch (e: any) {
        error.value =
            e?.response?.data?.error?.message ||
            'Failed to fetch application tasks'
    } finally {
        loading.value = false
    }
}

const statusBadgeClass = (status: ApplicationTaskStatus): string => {
    if (status === 'succeeded')
        return 'bg-success/10 text-success border-success/30'
    if (status === 'failed') return 'bg-error/10 text-error border-error/30'
    if (status === 'running') return 'bg-info/10 text-info border-info/30'
    return 'bg-surface-secondary text-text-muted border-border'
}

const logLevelClass = (level: ApplicationTaskLogLevel): string => {
    if (level === 'error') return 'text-error'
    if (level === 'warn') return 'text-warning'
    return 'text-info'
}

const formatTime = (raw: string, compact = false): string => {
    const dt = new Date(raw)
    if (Number.isNaN(dt.getTime())) return raw
    if (compact) return dt.toLocaleTimeString()
    return dt.toLocaleString()
}

watch(selectedTaskId, (taskId) => {
    selectedTask.value = tasks.value.find((task) => task.id === taskId) || null
    connectDetailStream()
})

watch(
    () => selectedTask.value?.logs.length ?? 0,
    async () => {
        await nextTick()
        if (!logContainerRef.value) return
        logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    },
)

onMounted(async () => {
    isUnmounted.value = false

    const initialTaskId = String(route.query.task_id || '').trim()

    await refreshTasks()

    if (
        initialTaskId &&
        tasks.value.some((task) => task.id === initialTaskId)
    ) {
        selectedTaskId.value = initialTaskId
    } else if (!selectedTaskId.value && tasks.value.length > 0) {
        selectedTaskId.value = tasks.value[0]?.id || ''
    }

    connectListStream()
    connectDetailStream()
})

onUnmounted(() => {
    isUnmounted.value = true
    listSessionId += 1
    detailSessionId += 1
    disconnectListStream()
    disconnectDetailStream()
    clearListReconnectTimer()
    clearDetailReconnectTimer()
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
</style>
