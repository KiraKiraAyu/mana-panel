<template>
    <div class="space-y-6">
        <div
            v-if="loading"
            class="h-64 flex flex-col items-center justify-center text-text-muted space-y-4"
        >
            <div
                class="w-8 h-8 border-4 border-reisa-lilac-500/30 border-t-reisa-lilac-500 rounded-full animate-spin"
            ></div>
            <p>Loading templates...</p>
        </div>

        <div
            v-else-if="templates.length === 0"
            class="flex flex-col items-center justify-center py-20 text-text-muted border-2 border-dashed border-border rounded-2xl bg-surface/30"
        >
            <svg
                class="w-12 h-12 mb-3 opacity-20"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                />
            </svg>
            <p>No templates available</p>
        </div>

        <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-5"
        >
            <div
                v-for="tpl in templates"
                :key="tpl.id"
                class="group relative flex flex-col bg-surface border border-border rounded-2xl p-5"
            >
                <div class="flex items-start justify-between mb-4">
                    <div class="flex items-center gap-3">
                        <div
                            class="w-10 h-10 rounded-lg bg-linear-to-br from-reisa-lilac-500/10 to-blue-500/10 flex items-center justify-center text-reisa-lilac-600 border border-reisa-lilac-500/20"
                        >
                            <span class="text-lg font-bold">{{
                                tpl.name.charAt(0).toUpperCase()
                            }}</span>
                        </div>
                        <div>
                            <h3
                                class="font-semibold text-text-primary leading-tight"
                            >
                                {{ tpl.name }}
                            </h3>
                            <div class="flex items-center gap-2 mt-1">
                                <span
                                    class="text-[10px] uppercase tracking-wider font-medium text-text-secondary bg-surface-secondary px-1.5 py-0.5 rounded border border-border"
                                >
                                    {{ tpl.category || 'App' }}
                                </span>
                                <span class="text-xs text-text-muted"
                                    >v{{ tpl.version }}</span
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <p class="text-sm text-text-secondary line-clamp-2 mb-4 grow">
                    {{ tpl.description || 'No description provided.' }}
                </p>

                <div
                    class="pt-4 border-t border-border mt-auto flex items-center justify-end"
                >
                    <BaseButton
                        variant="emphasis"
                        @click="$emit('install', tpl)"
                    >
                        Install
                    </BaseButton>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { ApplicationTemplate } from '@/api/applications'
import BaseButton from '@/components/universal/BaseButton.vue'

withDefaults(
    defineProps<{
        templates: ApplicationTemplate[]
        loading?: boolean
    }>(),
    {
        loading: false,
    },
)

defineEmits<{
    (e: 'install', template: ApplicationTemplate): void
}>()
</script>
