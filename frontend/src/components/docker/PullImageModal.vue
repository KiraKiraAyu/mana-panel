<template>
    <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="$emit('close')"
    >
        <div
            class="w-full max-w-lg bg-surface-elevated rounded-xl shadow-2xl animate-in"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-border-subtle"
            >
                <h3 class="font-semibold text-text-primary">Pull Image</h3>
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Image Name
                    </label>
                    <BaseInput
                        v-model="imageName"
                        type="text"
                        placeholder="e.g. nginx:latest, redis:7-alpine, ubuntu:22.04"
                        @keydown.enter="pullImage"
                        :disabled="pulling"
                    />
                    <p class="text-xs text-text-muted mt-1.5">
                        Format:
                        <span class="font-mono text-reisa-lilac-400"
                            >repository:tag</span
                        >. If no tag is specified,
                        <span class="font-mono text-reisa-lilac-400"
                            >latest</span
                        >
                        will be used.
                    </p>
                </div>

                <!-- Error -->
                <div
                    v-if="error"
                    class="p-3 rounded-lg bg-error/20 border border-error/30 text-error text-sm"
                >
                    {{ error }}
                </div>

                <!-- Success -->
                <div
                    v-if="success"
                    class="p-3 rounded-lg bg-success/20 border border-success/30 text-success text-sm"
                >
                    {{ success }}
                </div>

                <!-- Progress Log -->
                <div
                    v-if="progressLines.length > 0"
                    class="rounded-lg bg-surface border border-border-subtle overflow-hidden"
                >
                    <div
                        class="flex items-center justify-between px-3 py-2 border-b border-border-subtle"
                    >
                        <span class="text-xs font-medium text-text-muted"
                            >Pull Progress</span
                        >
                        <span class="text-xs text-text-muted">
                            {{ progressLines.length }} messages
                        </span>
                    </div>
                    <div
                        ref="progressEl"
                        class="max-h-48 overflow-auto p-3 font-mono text-xs leading-5"
                    >
                        <p
                            v-for="(line, index) in progressLines"
                            :key="index"
                            class="text-text-secondary"
                        >
                            <span v-if="line.id" class="text-reisa-lilac-400"
                                >{{ line.id }}:
                            </span>
                            <span>{{ line.status }}</span>
                            <span v-if="line.progress" class="text-text-muted">
                                {{ line.progress }}</span
                            >
                        </p>
                    </div>
                </div>

                <BaseButton
                    @click="pullImage"
                    variant="emphasis"
                    class="w-full"
                    :disabled="pulling || !imageName.trim()"
                >
                    <template v-if="pulling">
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
                        Pulling...
                    </template>
                    <template v-else>
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
                                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                            />
                        </svg>
                        Pull Image
                    </template>
                </BaseButton>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { api } from '@/api'
import BaseInput from '@/components/universal/BaseInput.vue'
import BaseButton from '@/components/universal/BaseButton.vue'

interface PullProgressLine {
    status: string
    id: string | null
    progress: string | null
}

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'pulled'): void
}>()

const imageName = ref('')
const pulling = ref(false)
const error = ref('')
const success = ref('')
const progressLines = ref<PullProgressLine[]>([])
const progressEl = ref<HTMLElement | null>(null)

const pullImage = async () => {
    const name = imageName.value.trim()
    if (!name) return

    pulling.value = true
    error.value = ''
    success.value = ''
    progressLines.value = []

    try {
        const response = await api.post('/docker/images/pull', {
            image: name,
        })

        // The API returns collected progress messages
        if (Array.isArray(response.data)) {
            progressLines.value = response.data
            await nextTick()
            scrollProgressToBottom()
        }

        success.value = `Image "${name}" pulled successfully!`
        emit('pulled')
    } catch (e: any) {
        error.value = e.response?.data?.error?.message || 'Failed to pull image'
    } finally {
        pulling.value = false
    }
}

const scrollProgressToBottom = () => {
    if (progressEl.value) {
        progressEl.value.scrollTop = progressEl.value.scrollHeight
    }
}
</script>
