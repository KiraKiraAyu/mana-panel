<template>
    <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-sm"
    >
        <div
            class="bg-surface border border-border rounded-xl shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-200"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-border bg-surface-secondary/50"
            >
                <div class="flex items-center gap-3">
                    <h2 class="text-lg font-semibold text-text-primary">
                        Logs: {{ appName || appId }}
                    </h2>
                    <div
                        v-if="loading"
                        class="w-4 h-4 border-2 border-reisa-lilac-500/30 border-t-reisa-lilac-500 rounded-full animate-spin"
                    ></div>
                </div>
                <div class="flex items-center gap-2">
                    <BaseButton
                        @click="fetchLogs"
                        class="p-2 text-text-secondary hover:text-text-primary transition-colors rounded-lg hover:bg-surface"
                        title="Refresh Logs"
                        :disabled="loading"
                    >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                    </BaseButton>
                    <BaseButton
                        @click="$emit('close')"
                        class="p-2 text-text-muted hover:text-text-primary transition-colors rounded-lg hover:bg-surface"
                    >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </BaseButton>
                </div>
            </div>

            <div
                class="flex-1 overflow-auto bg-[#1e1e1e] p-4 text-sm font-mono text-gray-300 min-h-[400px]"
                ref="logsContainer"
            >
                <div v-if="error" class="text-error mb-4">
                    {{ error }}
                </div>
                
                <template v-if="logs">
                    <pre class="whitespace-pre-wrap break-all">{{ logs }}</pre>
                </template>
                <div v-else-if="!loading && !error" class="text-gray-500 italic">
                    No logs available.
                </div>
            </div>
            
            <div class="p-4 border-t border-border bg-surface-secondary/50 flex justify-between items-center text-xs text-text-muted">
                <span>Showing last 500 lines</span>
                <BaseButton
                    v-if="logs"
                    @click="scrollToBottom"
                    class="hover:text-text-primary transition-colors"
                >
                    Scroll to Bottom
                </BaseButton>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { applicationsApi } from '@/api/applications'
import BaseButton from '@/components/universal/BaseButton.vue'

const props = defineProps<{
    open: boolean
    appId: string | null
    appName: string | null
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const logs = ref<string>('')
const loading = ref(false)
const error = ref('')
const logsContainer = ref<HTMLElement | null>(null)

const scrollToBottom = () => {
    if (logsContainer.value) {
        logsContainer.value.scrollTop = logsContainer.value.scrollHeight
    }
}

const fetchLogs = async () => {
    if (!props.appId) return
    
    loading.value = true
    error.value = ''
    try {
        const response = await applicationsApi.logs(props.appId, 500)
        logs.value = response.message
        
        nextTick(() => {
            scrollToBottom()
        })
    } catch (e: any) {
        error.value = e?.message || e.response?.data?.error?.message || 'Failed to fetch logs'
    } finally {
        loading.value = false
    }
}

watch(
    () => props.open,
    (isOpen) => {
        if (isOpen && props.appId) {
            logs.value = ''
            fetchLogs()
        }
    }
)
</script>
