<template>
    <div class="space-y-4">
        <!-- Toolbar -->
        <div class="flex items-center gap-4">
            <div class="flex-1">
                <BaseInput v-model="searchQuery" variant="search" />
            </div>
            <label
                class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer select-none"
            >
                <input
                    type="checkbox"
                    v-model="showAll"
                    @change="$emit('refresh', showAll)"
                    class="accent-reisa-lilac-500 w-4 h-4"
                />
                Show stopped
            </label>
            <BaseButton
                @click="$emit('create')"
                variant="outline"
                text="Create"
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
                        d="M12 4v16m8-8H4"
                    />
                </svg>
            </BaseButton>
            <BaseButton
                @click="$emit('refresh', showAll)"
                variant="outline"
                :disabled="loading"
                text="Refresh"
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
            </BaseButton>
            <div class="flex items-center gap-1 ml-auto">
                <!-- Start: visible when nothing selected or container is not running -->
                <BaseButton
                    v-if="
                        !selectedContainer ||
                        selectedContainer.state !== 'running'
                    "
                    :disabled="!selectedContainer"
                    @click="
                        selectedContainer && emit('start', selectedContainer.id)
                    "
                    class="text-success hover:bg-success/20"
                    variant="square"
                    title="Start"
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
                <!-- Stop/Restart/Stats: only when running container is selected -->
                <BaseButton
                    v-if="
                        selectedContainer &&
                        selectedContainer.state === 'running'
                    "
                    @click="emit('stop', selectedContainer.id)"
                    class="text-warning hover:bg-warning/20"
                    variant="square"
                    title="Stop"
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
                        <rect
                            x="9"
                            y="9"
                            width="6"
                            height="6"
                            rx="1"
                            stroke-width="2"
                        />
                    </svg>
                </BaseButton>
                <BaseButton
                    v-if="
                        selectedContainer &&
                        selectedContainer.state === 'running'
                    "
                    @click="emit('restart', selectedContainer.id)"
                    class="text-reisa-lilac-400 hover:bg-reisa-lilac-500/20"
                    variant="square"
                    title="Restart"
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
                            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                        />
                    </svg>
                </BaseButton>
                <BaseButton
                    :disabled="!selectedContainer"
                    @click="
                        selectedContainer && emit('logs', selectedContainer)
                    "
                    class="text-reisa-stripe-400 hover:bg-reisa-stripe-500/20"
                    variant="square"
                    title="Logs"
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
                            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        />
                    </svg>
                </BaseButton>
                <BaseButton
                    v-if="
                        selectedContainer &&
                        selectedContainer.state === 'running'
                    "
                    @click="emit('stats', selectedContainer)"
                    class="text-reisa-gold-400 hover:bg-reisa-gold-500/20"
                    variant="square"
                    title="Stats"
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
                            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                        />
                    </svg>
                </BaseButton>
                <BaseButton
                    :disabled="!selectedContainer"
                    @click="
                        selectedContainer && emit('remove', selectedContainer)
                    "
                    class="text-error hover:bg-error/20"
                    variant="square"
                    title="Remove"
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
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                        />
                    </svg>
                </BaseButton>
            </div>
        </div>

        <DataTable
            :columns="tableColumns"
            :loading="loading"
            :empty="filteredContainers.length === 0"
            :emptyText="
                containers.length === 0
                    ? 'No containers found'
                    : 'No containers match your search'
            "
        >
            <template #rows>
                <tr
                    v-for="c in filteredContainers"
                    :key="c.id"
                    class="cursor-pointer transition-colors"
                    :class="
                        selectedContainerId === c.id
                            ? 'bg-reisa-lilac-500/10'
                            : 'hover:bg-bg-tertiary'
                    "
                    @click="
                        selectedContainerId =
                            selectedContainerId === c.id ? null : c.id
                    "
                >
                    <td>
                        <div class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg flex items-center justify-center"
                                :class="
                                    c.state === 'running'
                                        ? 'bg-success/20'
                                        : 'bg-surface'
                                "
                            >
                                <svg
                                    class="w-4 h-4"
                                    :class="
                                        c.state === 'running'
                                            ? 'text-success'
                                            : 'text-text-muted'
                                    "
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                                    />
                                </svg>
                            </div>
                            <div>
                                <p class="font-medium text-text-primary">
                                    {{ c.names[0] || shortId(c.id) }}
                                </p>
                                <p class="text-xs text-text-muted font-mono">
                                    {{ shortId(c.id) }}
                                </p>
                            </div>
                        </div>
                    </td>
                    <td>
                        <span class="text-sm text-text-secondary font-mono">{{
                            c.image
                        }}</span>
                    </td>
                    <td>
                        <span class="badge" :class="stateBadge(c.state)">{{
                            c.state
                        }}</span>
                        <p class="text-xs text-text-muted mt-1">
                            {{ c.status }}
                        </p>
                    </td>
                    <td>
                        <div class="flex flex-wrap gap-1">
                            <span
                                v-for="(p, pi) in c.ports.slice(0, 3)"
                                :key="pi"
                                class="text-xs font-mono bg-surface px-2 py-0.5 rounded"
                            >
                                {{ p.public_port ? p.public_port + ':' : ''
                                }}{{ p.private_port }}/{{
                                    p.port_type.toLowerCase() || 'tcp'
                                }}
                            </span>
                            <span
                                v-if="c.ports.length > 3"
                                class="text-xs text-text-muted"
                            >
                                +{{ c.ports.length - 3 }}
                            </span>
                            <span
                                v-if="c.ports.length === 0"
                                class="text-xs text-text-muted"
                                >&mdash;</span
                            >
                        </div>
                    </td>
                    <td class="text-sm text-text-secondary">
                        {{ formatTimestamp(c.created) }}
                    </td>
                </tr>
            </template>
        </DataTable>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import BaseInput from '@/components/universal/BaseInput.vue'
import BaseButton from '@/components/universal/BaseButton.vue'
import DataTable from '@/components/universal/DataTable.vue'

export interface ContainerPort {
    ip: string
    private_port: number
    public_port: number | null
    port_type: string
}

export interface ContainerInfo {
    id: string
    names: string[]
    image: string
    image_id: string
    command: string
    created: number
    state: string
    status: string
    ports: ContainerPort[]
    labels: Record<string, string>
    size_rw: number | null
    size_root_fs: number | null
}

const props = defineProps<{
    containers: ContainerInfo[]
    loading: boolean
}>()

const emit = defineEmits<{
    (e: 'refresh', all: boolean): void
    (e: 'create'): void
    (e: 'start', id: string): void
    (e: 'stop', id: string): void
    (e: 'restart', id: string): void
    (e: 'logs', container: ContainerInfo): void
    (e: 'stats', container: ContainerInfo): void
    (e: 'remove', container: ContainerInfo): void
}>()

const tableColumns = [
    { key: 'name', label: 'Name' },
    { key: 'image', label: 'Image' },
    { key: 'status', label: 'Status' },
    { key: 'ports', label: 'Ports' },
    { key: 'created', label: 'Created' },
]

const searchQuery = ref('')
const showAll = ref(true)
const selectedContainerId = ref<string | null>(null)

const filteredContainers = computed(() => {
    if (!searchQuery.value) return props.containers
    const q = searchQuery.value.toLowerCase()
    return props.containers.filter(
        (c) =>
            c.names.some((n) => n.toLowerCase().includes(q)) ||
            c.image.toLowerCase().includes(q) ||
            c.id.toLowerCase().includes(q) ||
            c.state.toLowerCase().includes(q),
    )
})

const selectedContainer = computed(() => {
    if (!selectedContainerId.value) return null
    return (
        filteredContainers.value.find(
            (c) => c.id === selectedContainerId.value,
        ) ?? null
    )
})

watch(filteredContainers, (list) => {
    if (
        selectedContainerId.value &&
        !list.some((c) => c.id === selectedContainerId.value)
    ) {
        selectedContainerId.value = null
    }
})

const shortId = (id: string): string => {
    return id.replace('sha256:', '').substring(0, 12)
}

const stateBadge = (state: string): string => {
    switch (state) {
        case 'running':
            return 'badge-success'
        case 'exited':
            return 'badge-error'
        case 'paused':
            return 'badge-warning'
        case 'created':
            return 'badge-info'
        case 'restarting':
            return 'badge-warning'
        default:
            return 'badge-info'
    }
}

const formatTimestamp = (ts: number): string => {
    if (!ts) return '—'
    return new Date(ts * 1000).toLocaleString()
}
</script>
