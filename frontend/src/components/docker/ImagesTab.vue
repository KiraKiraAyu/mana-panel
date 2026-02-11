<template>
    <div class="space-y-4">
        <!-- Toolbar -->
        <div class="flex items-center gap-4">
            <div class="flex-1 relative">
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
                    placeholder="Search images..."
                />
            </div>
            <button
                @click="$emit('pull')"
                class="inline-flex items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)] disabled:cursor-not-allowed disabled:opacity-50"
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
                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                    />
                </svg>
                Pull Image
            </button>
            <button
                @click="$emit('refresh')"
                class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-5 py-2.5 text-sm font-medium text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
                :disabled="loading"
            >
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

        <!-- Images Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
                v-for="img in filteredImages"
                :key="img.id"
                class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
            >
                <div class="flex items-start justify-between mb-3">
                    <div class="flex items-center gap-3 flex-1 min-w-0">
                        <div
                            class="w-10 h-10 rounded-xl bg-reisa-stripe-500/20 flex items-center justify-center shrink-0"
                        >
                            <svg
                                class="w-5 h-5 text-reisa-stripe-400"
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
                        </div>
                        <div class="min-w-0 flex-1">
                            <p
                                class="font-semibold text-text-primary truncate"
                                :title="displayName(img)"
                            >
                                {{ displayName(img) }}
                            </p>
                            <p
                                class="text-xs text-text-muted font-mono truncate"
                                :title="img.id"
                            >
                                {{ shortId(img.id) }}
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Tags -->
                <div class="flex flex-wrap gap-1 mb-3">
                    <span
                        v-for="tag in img.repo_tags.slice(0, 4)"
                        :key="tag"
                        class="text-xs font-mono bg-surface px-2 py-0.5 rounded text-reisa-lilac-400"
                    >
                        {{ tag }}
                    </span>
                    <span
                        v-if="img.repo_tags.length > 4"
                        class="text-xs text-text-muted"
                    >
                        +{{ img.repo_tags.length - 4 }}
                    </span>
                    <span
                        v-if="img.repo_tags.length === 0"
                        class="text-xs text-text-muted italic"
                    >
                        &lt;untagged&gt;
                    </span>
                </div>

                <!-- Meta -->
                <div
                    class="flex items-center gap-4 text-xs text-text-muted mb-4"
                >
                    <span>{{ formatBytes(img.size) }}</span>
                    <span>&bull;</span>
                    <span>{{ formatTimestamp(img.created) }}</span>
                    <template v-if="img.containers > 0">
                        <span>&bull;</span>
                        <span class="text-reisa-gold-400">
                            {{ img.containers }} container{{
                                img.containers > 1 ? 's' : ''
                            }}
                        </span>
                    </template>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-2">
                    <button
                        @click="$emit('createFromImage', img)"
                        class="inline-flex flex-1 items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs font-medium text-success transition-all duration-200 hover:border-success/40 hover:bg-success/20"
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
                                d="M12 4v16m8-8H4"
                            />
                        </svg>
                        Create
                    </button>
                    <button
                        @click="$emit('remove', img)"
                        class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs font-medium text-error transition-all duration-200 hover:border-error/40 hover:bg-error/20"
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
                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                            />
                        </svg>
                        Remove
                    </button>
                </div>
            </div>
        </div>

        <!-- Loading Skeleton -->
        <div
            v-if="loading && images.length === 0"
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
        >
            <div v-for="i in 6" :key="i" class="card">
                <div class="shimmer h-6 w-32 rounded mb-2"></div>
                <div class="shimmer h-4 w-48 rounded mb-4"></div>
                <div class="shimmer h-8 w-full rounded"></div>
            </div>
        </div>

        <!-- Empty State -->
        <div
            v-if="!loading && filteredImages.length === 0"
            class="p-8 text-center text-text-muted"
        >
            {{
                images.length === 0
                    ? 'No images found'
                    : 'No images match your search'
            }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

export interface ImageInfo {
    id: string
    repo_tags: string[]
    repo_digests: string[]
    created: number
    size: number
    shared_size: number
    containers: number
    labels: Record<string, string>
}

const props = defineProps<{
    images: ImageInfo[]
    loading: boolean
}>()

defineEmits<{
    (e: 'refresh'): void
    (e: 'pull'): void
    (e: 'createFromImage', img: ImageInfo): void
    (e: 'remove', img: ImageInfo): void
}>()

const searchQuery = ref('')

const filteredImages = computed(() => {
    if (!searchQuery.value) return props.images
    const q = searchQuery.value.toLowerCase()
    return props.images.filter(
        (img) =>
            img.repo_tags.some((t) => t.toLowerCase().includes(q)) ||
            img.id.toLowerCase().includes(q),
    )
})

const shortId = (id: string): string => {
    return id.replace('sha256:', '').substring(0, 12)
}

const displayName = (img: ImageInfo): string => {
    if (
        img.repo_tags.length > 0 &&
        img.repo_tags[0] &&
        img.repo_tags[0] !== '<none>:<none>'
    ) {
        return img.repo_tags[0]
    }
    return shortId(img.id)
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const formatTimestamp = (ts: number): string => {
    if (!ts) return '—'
    return new Date(ts * 1000).toLocaleString()
}
</script>
