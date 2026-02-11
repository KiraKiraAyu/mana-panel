<template>
  <div class="p-6 space-y-6 animate-in">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-text-primary">Services</h1>
        <p class="text-text-muted mt-1">Manage systemd services</p>
      </div>
      <button @click="fetchServices" class="btn btn-ghost" :disabled="loading">
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

    <!-- Search -->
    <div class="relative">
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
        type="text"
        class="input pl-10"
        placeholder="Search services..."
      />
    </div>

    <!-- Services Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div
        v-for="service in filteredServices()"
        :key="service.name"
        class="card"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex-1 min-w-0">
            <h3 class="font-semibold text-text-primary truncate">
              {{ service.name }}
            </h3>
            <p class="text-sm text-text-muted truncate mt-1">
              {{ service.description }}
            </p>
          </div>
          <span :class="['badge', getStatusClass(service)]">
            {{ service.active_state }}
          </span>
        </div>

        <div class="flex items-center gap-2 text-xs text-text-muted mb-4">
          <span>{{ service.load_state }}</span>
          <span>•</span>
          <span>{{ service.sub_state }}</span>
        </div>

        <div class="flex items-center gap-2">
          <button
            v-if="service.active_state !== 'active'"
            @click="startService(service.name)"
            class="btn btn-ghost flex-1 text-success hover:bg-success/20"
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
            Start
          </button>
          <button
            v-else
            @click="stopService(service.name)"
            class="btn btn-ghost flex-1 text-warning hover:bg-warning/20"
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
            Stop
          </button>
          <button
            @click="restartService(service.name)"
            class="btn btn-ghost text-reisa-lilac-400 hover:bg-reisa-lilac-500/20"
            title="Restart"
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
            class="btn btn-ghost text-reisa-stripe-400 hover:bg-reisa-stripe-500/20"
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
      </div>
    </div>

    <div
      v-if="loading && services.length === 0"
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
    >
      <div v-for="i in 6" :key="i" class="card">
        <div class="shimmer h-6 w-32 rounded mb-2"></div>
        <div class="shimmer h-4 w-48 rounded mb-4"></div>
        <div class="shimmer h-8 w-full rounded"></div>
      </div>
    </div>

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
          <button @click="showLogs = false" class="btn btn-ghost">Close</button>
        </div>
        <div class="h-[60vh] overflow-auto p-4 bg-surface font-mono text-xs">
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
import { ref, onMounted } from 'vue'
import { api } from '@/api'

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
const selectedService = ref<Service | null>(null)
const logs = ref<string[]>([])
const showLogs = ref(false)

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

const filteredServices = () => {
  if (!searchQuery.value) return services.value
  const query = searchQuery.value.toLowerCase()
  return services.value.filter(
    (s) =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query),
  )
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
