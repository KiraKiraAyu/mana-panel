<template>
    <div class="p-6 space-y-6 animate-in">
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-text-primary">Docker</h1>
                <p class="text-text-muted mt-1">Manage containers and images</p>
            </div>
            <div class="flex items-center gap-2">
                <span
                    class="flex items-center gap-2 text-sm"
                    :class="dockerAvailable ? 'text-success' : 'text-error'"
                >
                    <span
                        class="w-2 h-2 rounded-full"
                        :class="
                            dockerAvailable
                                ? 'bg-success animate-pulse'
                                : 'bg-error'
                        "
                    ></span>
                    {{
                        dockerAvailable
                            ? 'Docker Connected'
                            : 'Docker Unavailable'
                    }}
                </span>
            </div>
        </div>

        <!-- Tabs -->
        <div class="flex items-center gap-1 p-1 bg-surface rounded-xl w-fit">
            <button
                @click="activeTab = 'containers'"
                class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
                :class="
                    activeTab === 'containers'
                        ? 'bg-surface-elevated text-reisa-lilac-400 shadow-sm'
                        : 'text-text-muted hover:text-text-secondary'
                "
            >
                <span class="flex items-center gap-2">
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
                            d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                        />
                    </svg>
                    Containers
                    <span
                        v-if="containers.length"
                        class="badge badge-info text-xs"
                    >
                        {{ containers.length }}
                    </span>
                </span>
            </button>
            <button
                @click="activeTab = 'images'"
                class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
                :class="
                    activeTab === 'images'
                        ? 'bg-surface-elevated text-reisa-lilac-400 shadow-sm'
                        : 'text-text-muted hover:text-text-secondary'
                "
            >
                <span class="flex items-center gap-2">
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
                            d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
                        />
                    </svg>
                    Images
                    <span v-if="images.length" class="badge badge-info text-xs">
                        {{ images.length }}
                    </span>
                </span>
            </button>
        </div>

        <!-- Containers Tab -->
        <ContainersTab
            v-if="activeTab === 'containers'"
            :containers="containers"
            :loading="containersLoading"
            @refresh="fetchContainers"
            @create="showCreateModal = true"
            @start="startContainer"
            @stop="stopContainer"
            @restart="restartContainer"
            @logs="openLogs"
            @stats="openStats"
            @remove="removeContainer"
        />

        <!-- Images Tab -->
        <ImagesTab
            v-if="activeTab === 'images'"
            :images="images"
            :loading="imagesLoading"
            @refresh="fetchImages"
            @pull="showPullModal = true"
            @createFromImage="createFromImage"
            @remove="removeImage"
        />

        <!-- Modals -->
        <ContainerLogsModal
            v-if="logsTarget"
            :container-id="logsTarget.id"
            :container-name="logsTarget.name"
            @close="logsTarget = null"
        />

        <ContainerStatsModal
            v-if="statsTarget"
            :container-id="statsTarget.id"
            :container-name="statsTarget.name"
            @close="statsTarget = null"
        />

        <PullImageModal
            v-if="showPullModal"
            @close="showPullModal = false"
            @pulled="onImagePulled"
        />

        <CreateContainerModal
            v-if="showCreateModal"
            :default-image="createDefaultImage"
            @close="closeCreateModal"
            @created="onContainerCreated"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import ContainersTab from '@/components/docker/ContainersTab.vue'
import ImagesTab from '@/components/docker/ImagesTab.vue'
import ContainerLogsModal from '@/components/docker/ContainerLogsModal.vue'
import ContainerStatsModal from '@/components/docker/ContainerStatsModal.vue'
import PullImageModal from '@/components/docker/PullImageModal.vue'
import CreateContainerModal from '@/components/docker/CreateContainerModal.vue'
import type { ContainerInfo } from '@/components/docker/ContainersTab.vue'
import type { ImageInfo } from '@/components/docker/ImagesTab.vue'

// --- State ---
const activeTab = ref<'containers' | 'images'>('containers')
const dockerAvailable = ref(true)

// Containers
const containers = ref<ContainerInfo[]>([])
const containersLoading = ref(false)

// Images
const images = ref<ImageInfo[]>([])
const imagesLoading = ref(false)

// Modals
const logsTarget = ref<{ id: string; name: string } | null>(null)
const statsTarget = ref<{ id: string; name: string } | null>(null)
const showPullModal = ref(false)
const showCreateModal = ref(false)
const createDefaultImage = ref('')
const closeCreateModal = () => {
    showCreateModal.value = false
    createDefaultImage.value = ''
}

// --- Fetch ---
const fetchContainers = async (all: boolean = true) => {
    containersLoading.value = true
    try {
        const response = await api.get('/docker/containers', {
            params: { all },
        })
        containers.value = response.data
        dockerAvailable.value = true
    } catch (e: any) {
        if (
            e.response?.data?.error?.message
                ?.toLowerCase()
                .includes('not available')
        ) {
            dockerAvailable.value = false
        }
        console.error('Failed to fetch containers:', e)
    } finally {
        containersLoading.value = false
    }
}

const fetchImages = async () => {
    imagesLoading.value = true
    try {
        const response = await api.get('/docker/images')
        images.value = response.data
        dockerAvailable.value = true
    } catch (e: any) {
        if (
            e.response?.data?.error?.message
                ?.toLowerCase()
                .includes('not available')
        ) {
            dockerAvailable.value = false
        }
        console.error('Failed to fetch images:', e)
    } finally {
        imagesLoading.value = false
    }
}

// --- Container Actions ---
const startContainer = async (id: string) => {
    try {
        await api.post(`/docker/containers/${id}/start`)
        await fetchContainers()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to start container')
    }
}

const stopContainer = async (id: string) => {
    try {
        await api.post(`/docker/containers/${id}/stop`)
        await fetchContainers()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to stop container')
    }
}

const restartContainer = async (id: string) => {
    try {
        await api.post(`/docker/containers/${id}/restart`)
        await fetchContainers()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to restart container')
    }
}

const removeContainer = async (container: ContainerInfo) => {
    const name = container.names[0] || container.id.substring(0, 12)
    const isRunning = container.state === 'running'
    const msg = isRunning
        ? `Container "${name}" is running. Force remove it?`
        : `Remove container "${name}"?`

    if (!confirm(msg)) return

    try {
        await api.delete(`/docker/containers/${container.id}`, {
            params: { force: isRunning },
        })
        await fetchContainers()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to remove container')
    }
}

const openLogs = (container: ContainerInfo) => {
    logsTarget.value = {
        id: container.id,
        name: container.names[0] || container.id.substring(0, 12),
    }
}

const openStats = (container: ContainerInfo) => {
    statsTarget.value = {
        id: container.id,
        name: container.names[0] || container.id.substring(0, 12),
    }
}

// --- Image Actions ---
const removeImage = async (img: ImageInfo) => {
    const name =
        img.repo_tags.length > 0 && img.repo_tags[0] !== '<none>:<none>'
            ? img.repo_tags[0]
            : img.id.replace('sha256:', '').substring(0, 12)

    if (!confirm(`Remove image "${name}"?`)) return

    try {
        await api.delete(`/docker/images/${encodeURIComponent(img.id)}`, {
            params: { force: false },
        })
        await fetchImages()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to remove image')
    }
}

const createFromImage = (img: ImageInfo) => {
    createDefaultImage.value =
        img.repo_tags.length > 0 && img.repo_tags[0] !== '<none>:<none>'
            ? img.repo_tags[0]!
            : img.id
    showCreateModal.value = true
}

const onImagePulled = () => {
    fetchImages()
}

const onContainerCreated = () => {
    fetchContainers()
}

// --- Lifecycle ---
onMounted(() => {
    fetchContainers()
    fetchImages()
})
</script>
