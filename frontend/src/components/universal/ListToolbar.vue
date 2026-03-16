<template>
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-4">
        <BaseInput
            v-model="searchQuery"
            @input="$emit('search')"
            variant="search"
        />

        <div class="flex items-center gap-2">
            <select v-if="filterOptions" v-model="filterValue" class="input">
                <option
                    v-for="opt in filterOptions"
                    :key="opt.value"
                    :value="opt.value"
                >
                    {{ opt.label }}
                </option>
            </select>

            <span
                v-if="totalCount !== undefined"
                class="text-sm text-text-muted whitespace-nowrap"
            >
                {{ filteredCount }} of {{ totalCount }} {{ itemLabel }}
            </span>
        </div>

        <div v-if="$slots.actions" class="flex items-center gap-1 ml-auto">
            <slot name="actions" />
        </div>
    </div>
</template>

<script setup lang="ts">
import BaseInput from '@/components/universal/BaseInput.vue'

interface FilterOption {
    value: string
    label: string
}

const searchQuery = defineModel<string>('searchQuery', { required: true })
const filterValue = defineModel<string>('filterValue')

withDefaults(
    defineProps<{
        filterOptions?: FilterOption[]
        filteredCount?: number
        totalCount?: number
        itemLabel?: string
    }>(),
    { itemLabel: 'items' },
)

defineEmits<{ search: [] }>()
</script>
