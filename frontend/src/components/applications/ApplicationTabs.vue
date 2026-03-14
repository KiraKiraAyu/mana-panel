<template>
    <div class="border-b border-border">
        <nav class="flex gap-6 relative" aria-label="Tabs">
            <button
                v-for="tab in tabs"
                :key="tab"
                @click="setTab(tab)"
                class="relative pb-3 px-1 h-8 text-sm font-medium transition-colors duration-300 capitalize flex items-center gap-2 cursor-pointer"
                :class="
                    activeTab === tab
                        ? 'text-reisa-lilac-500'
                        : 'text-text-secondary hover:text-text-primary'
                "
            >
                {{ tab }}
                <span
                    class="px-1.5 py-0.5 rounded-md text-xs font-semibold transition-colors"
                    :class="
                        activeTab === tab
                            ? 'bg-reisa-lilac-500/10 text-reisa-lilac-600'
                            : 'bg-surface text-text-muted'
                    "
                >
                    {{ countFor(tab) }}
                </span>
                <span
                    v-if="activeTab === tab"
                    class="absolute bottom-0 left-0 w-full h-0.5 bg-reisa-lilac-500 rounded-t-full transition-all duration-300"
                ></span>
            </button>
            <RouterLink
                class="flex justify-end absolute right-1 h-8 pb-3 text-text-secondary hover:text-text-primary transition-colors duration-300"
                to="/applications/tasks"
            >
                View Task Output
            </RouterLink>
        </nav>
    </div>
</template>

<script setup lang="ts">
type ApplicationTab = 'installed' | 'available'

const { activeTab, installedCount, availableCount } = defineProps<{
    activeTab: ApplicationTab
    installedCount: number
    availableCount: number
}>()

const emit = defineEmits<{
    (e: 'update:activeTab', value: ApplicationTab): void
}>()

const tabs: ApplicationTab[] = ['installed', 'available']

const countFor = (tab: ApplicationTab): number =>
    tab === 'installed' ? installedCount : availableCount

const setTab = (tab: ApplicationTab) => {
    emit('update:activeTab', tab)
}
</script>
