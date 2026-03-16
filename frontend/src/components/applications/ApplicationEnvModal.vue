<template>
    <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-sm"
    >
        <div
            class="bg-surface border border-border rounded-xl shadow-xl w-full max-w-2xl max-h-[90vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-200"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-border bg-surface-secondary/50"
            >
                <div>
                    <h2 class="text-lg font-semibold text-text-primary">
                        Environment: {{ appName || appId }}
                    </h2>
                    <p class="text-xs text-text-muted mt-1">
                        Edit environment variables (key=value). Saving will restart the application.
                    </p>
                </div>
                <BaseButton
                    @click="$emit('close')"
                    class="p-2 text-text-muted hover:text-text-primary transition-colors rounded-lg hover:bg-surface"
                >
                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </BaseButton>
            </div>

            <div class="flex-1 overflow-auto p-4 space-y-4">
                <div v-if="loading" class="flex justify-center py-8">
                    <div class="w-8 h-8 border-4 border-reisa-lilac-500/30 border-t-reisa-lilac-500 rounded-full animate-spin"></div>
                </div>
                
                <div v-else class="space-y-4">
                    <div v-if="error" class="p-3 bg-error/10 border border-error/20 text-error rounded-lg text-sm">
                        {{ error }}
                    </div>
                    <div v-if="success" class="p-3 bg-success/10 border border-success/20 text-success rounded-lg text-sm">
                        {{ success }}
                    </div>
                    
                    <textarea
                        v-model="envText"
                        class="input w-full font-mono text-sm min-h-[300px] resize-y"
                        placeholder="KEY=VALUE"
                        spellcheck="false"
                    ></textarea>
                </div>
            </div>
            
            <div class="p-4 border-t border-border bg-surface-secondary/50 justify-end flex gap-3">
                <BaseButton
                    @click="$emit('close')"
                    variant="outline"
                    :disabled="submitting"
                >
                    Cancel
                </BaseButton>
                <BaseButton
                    @click="saveEnv"
                    variant="emphasis"
                    :disabled="loading || submitting"
                >
                    <span v-if="submitting">Saving...</span>
                    <span v-else>Save & Restart</span>
                </BaseButton>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { applicationsApi } from '@/api/applications'
import BaseButton from '@/components/universal/BaseButton.vue'

const props = defineProps<{
    open: boolean
    appId: string | null
    appName: string | null
}>()

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'updated'): void
}>()

const envText = ref<string>('')
const loading = ref(false)
const submitting = ref(false)
const error = ref('')
const success = ref('')

const fetchEnv = async () => {
    if (!props.appId) return
    
    loading.value = true
    error.value = ''
    success.value = ''
    try {
        const envMap = await applicationsApi.getEnv(props.appId)
        
        // Convert map to KEY=VALUE string
        const entries = Object.entries(envMap).sort(([a], [b]) => a.localeCompare(b))
        envText.value = entries.map(([k, v]) => `${k}=${v}`).join('\n')
    } catch (e: any) {
        error.value = e?.message || e.response?.data?.error?.message || 'Failed to fetch environment variables'
    } finally {
        loading.value = false
    }
}

const saveEnv = async () => {
    if (!props.appId) return
    
    submitting.value = true
    error.value = ''
    success.value = ''
    try {
        // Parse textarea back into Map
        const newEnvMap: Record<string, string> = {}
        const lines = envText.value.split('\n')
        for (const line of lines) {
            const trimmed = line.trim()
            if (!trimmed || trimmed.startsWith('#')) continue
            
            const matchIndex = trimmed.indexOf('=')
            if (matchIndex > 0) {
                const k = trimmed.substring(0, matchIndex).trim()
                const v = trimmed.substring(matchIndex + 1).trim()
                newEnvMap[k] = v
            }
        }
        
        await applicationsApi.updateEnv(props.appId, newEnvMap)
        success.value = 'Environment updated and application restarted.'
        emit('updated')
        setTimeout(() => emit('close'), 1500)
    } catch (e: any) {
        error.value = e?.message || e.response?.data?.error?.message || 'Failed to update environment'
    } finally {
        submitting.value = false
    }
}

watch(
    () => props.open,
    (isOpen) => {
        if (isOpen && props.appId) {
            envText.value = ''
            fetchEnv()
        }
    }
)
</script>
