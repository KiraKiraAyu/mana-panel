<template>
    <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="$emit('close')"
    >
        <div
            class="w-full max-w-5xl bg-surface-elevated rounded-xl shadow-2xl animate-in"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-border-subtle"
            >
                <div>
                    <h3 class="font-semibold text-text-primary">
                        Logs: {{ containerName }}
                    </h3>
                    <p class="text-xs text-text-muted font-mono mt-0.5">
                        {{ containerId }}
                    </p>
                </div>
                <div class="flex items-center gap-2">
                    <select
                        v-model="tailLines"
                        @change="refresh"
                        class="w-24 rounded-lg border border-border bg-surface px-2 py-1 text-xs text-text-primary transition-all duration-200 focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                    >
                        <option :value="100">100 lines</option>
                        <option :value="200">200 lines</option>
                        <option :value="500">500 lines</option>
                        <option :value="1000">1000 lines</option>
                        <option :value="3000">3000 lines</option>
                    </select>
                    <button
                        @click="refresh"
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
                        @click="scrollToBottom"
                        class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
                    >
                        <svg
                            class="w-3.5 h-3.5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 14l-7 7m0 0l-7-7m7 7V3"
                            />
                        </svg>
                        Bottom
                    </button>
                    <button
                        @click="$emit('close')"
                        class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
                    >
                        Close
                    </button>
                </div>
            </div>
            <div
                ref="logsEl"
                class="h-[60vh] overflow-auto p-4 bg-surface font-mono text-xs leading-5"
            >
                <p
                    v-for="(line, index) in logs"
                    :key="index"
                    class="text-text-secondary whitespace-pre-wrap break-all"
                >
                    {{ line }}
                </p>
                <p v-if="logs.length === 0 && !loading" class="text-text-muted">
                    No logs available
                </p>
                <div
                    v-if="loading && logs.length === 0"
                    class="flex items-center gap-2 text-text-muted"
                >
                    <svg
                        class="w-4 h-4 animate-spin"
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
                    Loading logs...
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { api } from '@/api'

const props = defineProps<{
    containerId: string
    containerName: string
}>()

defineEmits<{
    (e: 'close'): void
}>()

const logs = ref<string[]>([])
const loading = ref(false)
const tailLines = ref(200)
const logsEl = ref<HTMLElement | null>(null)

const fetchLogs = async () => {
    loading.value = true
    try {
        const response = await api.get(
            `/docker/containers/${props.containerId}/logs`,
            {
                params: { tail: tailLines.value },
            },
        )
        logs.value = response.data
        await nextTick()
        scrollToBottom()
    } catch (e: any) {
        logs.value = [
            `Error fetching logs: ${e.response?.data?.error?.message || e.message}`,
        ]
    } finally {
        loading.value = false
    }
}

const refresh = () => {
    fetchLogs()
}

const scrollToBottom = () => {
    if (logsEl.value) {
        logsEl.value.scrollTop = logsEl.value.scrollHeight
    }
}

onMounted(() => {
    fetchLogs()
})
</script>
