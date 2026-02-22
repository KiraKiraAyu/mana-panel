<template>
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
        <div>
            <h1 class="text-3xl font-bold text-text-primary tracking-tight">
                {{ title }}
            </h1>
            <p class="text-text-muted mt-2 text-sm md:text-base max-w-2xl">
                {{ description }}
            </p>
        </div>

        <div class="flex items-center gap-3">
            <div
                class="px-4 py-2 rounded-full border bg-surface/50 backdrop-blur-sm transition-colors duration-300"
                :class="
                    dockerAvailable
                        ? 'border-success/20 text-success bg-success/5'
                        : 'border-error/20 text-error bg-error/5'
                "
            >
                <div class="flex items-center gap-2.5">
                    <span class="relative flex h-2.5 w-2.5">
                        <span
                            v-if="dockerAvailable"
                            class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"
                        ></span>
                        <span
                            class="relative inline-flex rounded-full h-2.5 w-2.5"
                            :class="dockerAvailable ? 'bg-success' : 'bg-error'"
                        ></span>
                    </span>
                    <span class="text-sm font-medium">
                        {{
                            dockerAvailable
                                ? 'Docker Engine Connected'
                                : 'Docker Service Unavailable'
                        }}
                    </span>
                </div>
            </div>

            <button
                class="btn btn-ghost btn-square p-2 text-text-secondary hover:text-text-primary hover:bg-surface border border-transparent hover:border-border rounded-xl transition-all"
                @click="$emit('refresh')"
                :disabled="loading || submitting"
                title="Refresh Data"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    :class="{ 'animate-spin': loading }"
                >
                    <path
                        d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
                    />
                    <path d="M3 3v5h5" />
                    <path
                        d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"
                    />
                    <path d="M16 21h5v-5" />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
withDefaults(
    defineProps<{
        loading: boolean
        submitting: boolean
        dockerAvailable: boolean
        title?: string
        description?: string
    }>(),
    {
        title: 'Applications',
        description:
            'Deploy and manage based on Docker Compose containerized applications.',
    },
)

defineEmits<{
    (e: 'refresh'): void
}>()
</script>
