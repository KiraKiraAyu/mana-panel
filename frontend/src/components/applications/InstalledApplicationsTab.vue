<template>
    <div class="space-y-4">
        <div
            v-if="instances.length === 0"
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
                    d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                />
            </svg>
            <p>No applications installed</p>
            <button
                @click="$emit('switchToAvailable')"
                class="mt-4 text-reisa-lilac-500 hover:underline text-sm"
            >
                Go to App Store
            </button>
        </div>

        <div v-else class="space-y-3">
            <div
                v-for="app in instances"
                :key="app.id"
                class="group bg-surface hover:bg-surface/80 border border-border rounded-xl p-4 transition-all duration-200 flex flex-col md:flex-row md:items-center gap-4"
            >
                <div
                    class="md:hidden w-full h-1 rounded-full overflow-hidden mb-2 bg-surface-secondary"
                >
                    <div
                        class="h-full"
                        :class="
                            app.state === 'running'
                                ? 'bg-success w-full'
                                : 'bg-text-muted w-1/3'
                        "
                    ></div>
                </div>

                <div class="flex-1 min-w-0 flex items-center gap-4">
                    <div
                        class="hidden md:block w-2.5 h-2.5 rounded-full ring-2 ring-offset-2 ring-offset-surface"
                        :class="
                            app.state === 'running'
                                ? 'bg-success ring-success/20'
                                : 'bg-text-muted ring-text-muted/20'
                        "
                        :title="app.state"
                    ></div>

                    <div class="min-w-0">
                        <h3
                            class="text-base font-semibold text-text-primary truncate flex items-center gap-2"
                        >
                            {{ app.name || app.id }}
                            <span
                                class="md:hidden text-[10px] px-1.5 py-0.5 rounded border"
                                :class="
                                    app.state === 'running'
                                        ? 'bg-success/10 text-success border-success/20'
                                        : 'bg-surface-secondary text-text-muted border-border'
                                "
                            >
                                {{ app.state }}
                            </span>
                        </h3>
                        <p class="text-xs text-text-muted mt-0.5">
                            <span class="font-mono">{{ app.template_id }}</span>
                            <span class="mx-1.5 opacity-50">|</span>
                            <span
                                v-if="app.ports.length"
                                class="text-text-secondary"
                            >
                                {{
                                    app.ports
                                        .map((p) =>
                                            p.public_port
                                                ? `${p.public_port}→${p.private_port}`
                                                : `${p.private_port}`,
                                        )
                                        .join(', ')
                                }}
                            </span>
                            <span v-else>Internal Only</span>
                        </p>

                        <div
                            v-if="app.services?.length"
                            class="mt-2 space-y-1.5"
                        >
                            <div
                                v-for="svc in app.services"
                                :key="`${app.id}-${svc.name}`"
                                class="flex flex-wrap items-center gap-2 text-[11px]"
                            >
                                <span class="font-mono text-text-secondary">{{
                                    svc.name
                                }}</span>
                                <span
                                    class="px-1.5 py-0.5 rounded border"
                                    :class="
                                        svc.state === 'running'
                                            ? 'bg-success/10 text-success border-success/20'
                                            : 'bg-surface-secondary text-text-muted border-border'
                                    "
                                >
                                    {{ svc.state }}
                                </span>
                                <span class="text-text-muted">·</span>
                                <span class="text-text-muted">
                                    {{
                                        svc.ports.length
                                            ? svc.ports
                                                  .map((p) =>
                                                      p.public_port
                                                          ? `${p.public_port}→${p.private_port}`
                                                          : `${p.private_port}`,
                                                  )
                                                  .join(', ')
                                            : 'Internal Only'
                                    }}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                <div
                    class="flex items-center gap-1 md:opacity-0 md:group-hover:opacity-100 transition-opacity duration-200"
                >
                    <template v-if="app.state !== 'running'">
                        <button
                            @click="$emit('start', String(app.id))"
                            title="Start"
                            class="p-2 text-text-secondary hover:text-success hover:bg-success/10 rounded-lg transition-colors"
                            :disabled="loading"
                        >
                            <svg
                                class="w-5 h-5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                                />
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                            </svg>
                        </button>
                    </template>
                    <template v-else>
                        <button
                            @click="$emit('stop', String(app.id))"
                            title="Stop"
                            class="p-2 text-text-secondary hover:text-warning hover:bg-warning/10 rounded-lg transition-colors"
                            :disabled="loading"
                        >
                            <svg
                                class="w-5 h-5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                            </svg>
                        </button>
                    </template>

                    <div class="w-px h-4 bg-border mx-1"></div>

                    <button
                        @click="$emit('update', String(app.id))"
                        title="Update"
                        class="p-2 text-text-secondary hover:text-reisa-lilac-500 hover:bg-reisa-lilac-500/10 rounded-lg transition-colors"
                        :disabled="loading"
                    >
                        <svg
                            class="w-4 h-4"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M4 4v6h6M20 20v-6h-6M20 8a8 8 0 00-13.657-5.657L4 4m16 16l-2.343 2.343A8 8 0 014 16"
                            />
                        </svg>
                    </button>

                    <button
                        @click="$emit('remove', String(app.id), app.name)"
                        title="Remove"
                        class="p-2 text-text-secondary hover:text-error hover:bg-error/10 rounded-lg transition-colors"
                        :disabled="loading"
                    >
                        <svg
                            class="w-4 h-4"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                            />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { ApplicationInstance } from '@/api/applications'

defineProps<{
    instances: ApplicationInstance[]
    loading: boolean
}>()

defineEmits<{
    (e: 'start', id: string): void
    (e: 'stop', id: string): void
    (e: 'update', id: string): void
    (e: 'remove', id: string, name: string): void
    (e: 'switchToAvailable'): void
}>()
</script>
