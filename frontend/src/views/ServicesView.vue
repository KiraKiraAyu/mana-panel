<template>
    <div class="p-6 space-y-6 animate-in">
        <ListToolbar
            v-model:searchQuery="searchQuery"
            v-model:filterValue="statusFilter"
            :filterOptions="statusFilterOptions"
            :filteredCount="filteredCount"
            :totalCount="totalCount"
            itemLabel="services"
        />

        <DataTable
            :columns="tableColumns"
            :loading="loading"
            :empty="filteredServices.length === 0"
            emptyText="No services found matching your criteria"
        >
            <template #rows>
                <tr
                    v-for="service in filteredServices"
                    :key="service.name"
                    class="hover:bg-bg-tertiary transition-colors"
                >
                    <td class="font-medium text-text-primary">
                        {{ service.name }}
                    </td>
                    <td
                        class="text-sm text-text-muted max-w-xs truncate"
                        :title="service.description"
                    >
                        {{ service.description }}
                    </td>
                    <td class="text-sm text-text-secondary">
                        {{ service.load_state }}
                    </td>
                    <td class="text-sm text-text-secondary">
                        {{ service.sub_state }}
                    </td>
                    <td>
                        <span
                            :class="['badge text-xs', getStatusClass(service)]"
                        >
                            {{ service.active_state }}
                        </span>
                    </td>
                    <td>
                        <div class="flex items-center justify-end gap-1">
                            <button
                                v-if="service.active_state !== 'active'"
                                @click="startService(service.name)"
                                class="p-1.5 rounded-lg hover:bg-success/20 text-success transition-colors"
                                title="Start Service"
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
                                @click="stopService(service.name)"
                                class="p-1.5 rounded-lg hover:bg-warning/20 text-warning transition-colors"
                                title="Stop Service"
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
                                        d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                    />
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"
                                    />
                                </svg>
                            </button>
                            <button
                                @click="restartService(service.name)"
                                class="p-1.5 rounded-lg hover:bg-reisa-lilac-500/20 text-reisa-lilac-400 transition-colors"
                                title="Restart Service"
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
                                        d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                    />
                                </svg>
                            </button>
                            <button
                                @click="viewLogs(service)"
                                class="p-1.5 rounded-lg hover:bg-reisa-stripe-500/20 text-reisa-stripe-400 transition-colors"
                                title="View Logs"
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
                                        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                                    />
                                </svg>
                            </button>
                        </div>
                    </td>
                </tr>
            </template>
        </DataTable>

        <!-- Logs Modal -->
        <div
            v-if="showLogs"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
            @click.self="showLogs = false"
        >
            <div
                class="w-full max-w-4xl bg-surface-elevated rounded-xl shadow-2xl animate-in"
            >
                <div
                    class="flex items-center justify-between p-4 border-b border-border-subtle"
                >
                    <h3 class="font-semibold text-text-primary">
                        Logs: {{ selectedService?.name }}
                    </h3>
                    <button @click="showLogs = false" class="btn btn-ghost">
                        Close
                    </button>
                </div>
                <div
                    class="h-[60vh] overflow-auto p-4 bg-surface font-mono text-xs"
                >
                    <p
                        v-for="(line, index) in logs"
                        :key="index"
                        class="text-text-secondary whitespace-pre-wrap"
                    >
                        {{ line }}
                    </p>
                    <p v-if="logs.length === 0" class="text-text-muted">
                        No logs available
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/api'
import ListToolbar from '@/components/universal/ListToolbar.vue'
import DataTable from '@/components/universal/DataTable.vue'

interface Service {
    name: string
    description: string
    load_state: string
    active_state: string
    sub_state: string
}

const services = ref<Service[]>([])
const loading = ref(false)
const searchQuery = ref('')
const statusFilter = ref('all')
const selectedService = ref<Service | null>(null)
const logs = ref<string[]>([])
const showLogs = ref(false)

const tableColumns = [
    { key: 'name', label: 'Name' },
    { key: 'description', label: 'Description' },
    { key: 'load_state', label: 'Load State' },
    { key: 'sub_state', label: 'Sub State' },
    { key: 'status', label: 'Status' },
    { key: 'actions', label: 'Actions', align: 'right' as const },
]

const statusFilterOptions = [
    { value: 'all', label: 'All Status' },
    { value: 'active', label: 'Active' },
    { value: 'inactive', label: 'Inactive' },
    { value: 'failed', label: 'Failed' },
]

const filteredServices = computed(() => {
    let result = services.value

    if (statusFilter.value !== 'all') {
        result = result.filter((s) => s.active_state === statusFilter.value)
    }

    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase()
        result = result.filter(
            (s) =>
                s.name.toLowerCase().includes(query) ||
                s.description.toLowerCase().includes(query),
        )
    }

    return result
})

const totalCount = computed(() => services.value.length)
const filteredCount = computed(() => filteredServices.value.length)

const fetchServices = async () => {
    loading.value = true
    try {
        const response = await api.get('/services')
        services.value = response.data
    } catch (e) {
        console.error('Failed to fetch services:', e)
    } finally {
        loading.value = false
    }
}

const startService = async (name: string) => {
    try {
        await api.post(`/services/${name}/start`)
        await fetchServices()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to start service')
    }
}

const stopService = async (name: string) => {
    if (!confirm(`Are you sure you want to stop ${name}?`)) return
    try {
        await api.post(`/services/${name}/stop`)
        await fetchServices()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to stop service')
    }
}

const restartService = async (name: string) => {
    try {
        await api.post(`/services/${name}/restart`)
        await fetchServices()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to restart service')
    }
}

const viewLogs = async (service: Service) => {
    selectedService.value = service
    showLogs.value = true
    try {
        const response = await api.get(`/services/${service.name}/logs`, {
            params: { lines: 100 },
        })
        logs.value = response.data.logs
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to fetch logs')
    }
}

const getStatusClass = (service: Service) => {
    if (service.active_state === 'active') return 'badge-success'
    if (service.active_state === 'inactive') return 'badge-warning'
    if (service.active_state === 'failed') return 'badge-error'
    return 'badge-info'
}

onMounted(() => {
    fetchServices()
})
</script>
