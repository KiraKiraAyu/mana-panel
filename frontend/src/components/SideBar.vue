<template>
    <aside>
        <!-- Logo -->
        <div class="p-6 border-b border-border-subtle">
            <div class="flex items-center gap-3">
                <div
                    class="w-10 h-10 rounded-xl bg-linear-to-br from-reisa-lilac-500 to-reisa-pink-500 flex items-center justify-center"
                >
                    <svg
                        class="w-6 h-6 text-white"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
                        />
                    </svg>
                </div>
                <div class="flex-1 min-w-0">
                    <h1 class="text-lg font-bold gradient-text">Mana Panel</h1>
                    <p class="text-xs text-text-muted">{{ hostname }}</p>
                </div>
            </div>
        </div>

        <!-- Navigation -->
        <nav class="flex-1 p-4 relative">
            <div
                class="absolute h-12 inset-0 m-4 rounded-lg bg-linear-to-br from-reisa-lilac-500/20 to-reisa-pink-500/10 z-0 transition-transform duration-300"
                :style="{ transform: `translateY(${offset})` }"
            ></div>
            <RouterLink
                v-for="item in navItems"
                :key="item.path"
                :to="item.path"
                class="relative flex items-center h-12 gap-3 z-1 py-3 px-4 text-sm text-text-secondary rounded-lg no-underline bg-clip-text transition-color duration-300 hover:text-text-primary"
                :class="{ 'text-reisa-lilac-400': isActive(item.path) }"
                @pointerdown.left.exact.prevent="navigateImmediate(item.path)"
            >
                <Icon class="text-lg" :icon="item.icon"></Icon>
                <span>{{ item.name }}</span>
            </RouterLink>
        </nav>

        <!-- Connectivity Status -->
        <Transition name="conn">
            <div
                v-if="connectionStore.status !== null"
                class="p-4 border-t border-border-subtle"
            >
                <div class="flex items-center gap-3">
                    <div
                        class="relative flex items-center justify-center w-8 h-8"
                    >
                        <!-- Ping ring for connected state -->
                        <span
                            v-if="connectionStore.status === 'connected'"
                            class="absolute inline-flex w-3 h-3 rounded-full bg-success opacity-60 animate-ping"
                        ></span>
                        <!-- Core dot -->
                        <span
                            class="relative inline-flex w-2.5 h-2.5 rounded-full transition-colors duration-300"
                            :class="{
                                'bg-success':
                                    connectionStore.status === 'connected',
                                'bg-warning animate-pulse':
                                    connectionStore.status === 'connecting',
                                'bg-error':
                                    connectionStore.status === 'disconnected',
                            }"
                        ></span>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm font-medium text-text-primary">
                            {{
                                connectionStore.status === 'connected'
                                    ? 'Connected'
                                    : connectionStore.status === 'connecting'
                                      ? 'Connecting…'
                                      : 'Disconnected'
                            }}
                        </p>
                        <p class="text-xs text-text-muted">Live stream</p>
                    </div>
                </div>
            </div>
        </Transition>
    </aside>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useConnectionStore } from '@/stores/connection'
import { useRoute, useRouter } from 'vue-router'
import { useSystemStore } from '@/stores/system'

const connectionStore = useConnectionStore()
const hostname = computed(() => systemStore.info?.hostname || 'Server')

const navigateImmediate = (path: string) => {
    if (route.path === path) return
    router.push(path)
}

const route = useRoute()
const router = useRouter()
const systemStore = useSystemStore()

const navItems = [
    { path: '/', name: 'Dashboard', icon: 'mdi:view-dashboard' },
    { path: '/processes', name: 'Processes', icon: 'mdi:apps' },
    { path: '/files', name: 'Files', icon: 'mdi:folder' },
    { path: '/services', name: 'Services', icon: 'mdi:cogs' },
    { path: '/docker', name: 'Docker', icon: 'mdi:docker' },
    { path: '/applications', name: 'Application', icon: 'mdi:cube' },
    { path: '/websites', name: 'Websites', icon: 'mdi:earth' },
    { path: '/terminal', name: 'Terminal', icon: 'mdi:application-brackets' },
    { path: '/settings', name: 'Settings', icon: 'mdi:cog' },
]

const offset = computed(() => {
    const index = navItems.findIndex((element) => isActive(element.path))
    return `${index * 48}px`
})

const isActive = (path: string) => {
    if (path === '/') return route.path === '/'
    return route.path.startsWith(path)
}

onMounted(() => {
    systemStore.fetchInfo()
    systemStore.startStreaming()
})

onUnmounted(() => {
    systemStore.stopStreaming()
})
</script>

<style scoped>
.conn-enter-active,
.conn-leave-active {
    transition:
        opacity 0.25s ease,
        transform 0.25s ease;
}
.conn-enter-from,
.conn-leave-to {
    opacity: 0;
    transform: translateY(8px);
}
</style>
