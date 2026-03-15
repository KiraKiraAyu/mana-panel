<template>
    <div class="p-0 overflow-hidden">
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th
                            v-for="col in columns"
                            :key="col.key"
                            :class="[
                                col.align === 'right' ? 'text-right' : '',
                                col.sortable
                                    ? 'cursor-pointer hover:text-text-primary select-none'
                                    : '',
                            ]"
                            @click="col.sortable && $emit('sort', col.key)"
                        >
                            <div
                                :class="[
                                    'flex items-center gap-1',
                                    col.align === 'right'
                                        ? 'justify-end'
                                        : '',
                                ]"
                            >
                                {{ col.label }}
                                <svg
                                    v-if="col.sortable && sortBy === col.key"
                                    class="w-4 h-4"
                                    :class="{
                                        'rotate-180': sortOrder === 'desc',
                                    }"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M5 15l7-7 7 7"
                                    />
                                </svg>
                            </div>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <!-- Skeleton rows shown during initial load -->
                    <template v-if="loading && empty">
                        <tr v-for="i in skeletonRows" :key="'sk-' + i">
                            <td
                                v-for="col in columns"
                                :key="col.key"
                            >
                                <div
                                    class="shimmer h-4 rounded"
                                    :class="col.align === 'right' ? 'ml-auto w-20' : 'w-32'"
                                ></div>
                            </td>
                        </tr>
                    </template>

                    <slot name="rows" />
                </tbody>
            </table>

            <!-- Empty state -->
            <div
                v-if="!loading && empty"
                class="p-8 text-center text-text-muted"
            >
                <slot name="empty">
                    <svg
                        class="w-12 h-12 mx-auto mb-2 opacity-50"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                    </svg>
                    <p>{{ emptyText }}</p>
                </slot>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
export interface Column {
    key: string
    label: string
    align?: 'left' | 'right'
    sortable?: boolean
}

withDefaults(
    defineProps<{
        columns: Column[]
        loading?: boolean
        empty?: boolean
        emptyText?: string
        sortBy?: string
        sortOrder?: 'asc' | 'desc'
        skeletonRows?: number
    }>(),
    {
        loading: false,
        empty: false,
        emptyText: 'No items found',
        skeletonRows: 5,
    },
)

defineEmits<{
    sort: [key: string]
}>()
</script>
