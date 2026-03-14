<template>
    <div class="flex min-h-screen">
        <!-- Sidebar -->
        <aside
            class="fixed h-screen w-64 bg-surface border-r border-border-subtle flex flex-col"
        >
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
                        <h1 class="text-lg font-bold gradient-text">
                            Mana Panel
                        </h1>
                        <p class="text-xs text-text-muted">{{ hostname }}</p>
                    </div>
                    <button
                        @click="handleLogout"
                        class="p-1.5 rounded-lg hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors"
                        title="Logout"
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
                                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                            />
                        </svg>
                    </button>
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
                    @pointerdown.left.exact.prevent="
                        navigateImmediate(item.path)
                    "
                >
                    <!-- Dashboard Icon -->
                    <svg
                        v-if="item.icon === 'dashboard'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
                        />
                    </svg>
                    <!-- Process Icon -->
                    <svg
                        v-else-if="item.icon === 'process'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                        />
                    </svg>
                    <!-- Folder Icon -->
                    <svg
                        v-else-if="item.icon === 'folder'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                        />
                    </svg>
                    <!-- Service Icon -->
                    <svg
                        v-else-if="item.icon === 'service'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        />
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                    </svg>
                    <!-- Docker Icon -->
                    <svg
                        v-else-if="item.icon === 'docker'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                        />
                    </svg>
                    <!-- Application Icon -->
                    <svg
                        v-else-if="item.icon === 'application'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M3.6 9h16.8M3.6 15h16.8M12 3a15 15 0 010 18M12 3a15 15 0 000 18"
                        />
                    </svg>
                    <!-- Website Icon -->
                    <svg
                        v-else-if="item.icon === 'website'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                    </svg>
                    <!-- Terminal Icon -->
                    <svg
                        v-else-if="item.icon === 'terminal'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                        />
                    </svg>
                    <!-- Settings Icon -->
                    <svg
                        v-else-if="item.icon === 'settings'"
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        />
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                    </svg>
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
                        <div class="relative flex items-center justify-center w-8 h-8">
                            <!-- Ping ring for connected state -->
                            <span
                                v-if="connectionStore.status === 'connected'"
                                class="absolute inline-flex w-3 h-3 rounded-full bg-success opacity-60 animate-ping"
                            ></span>
                            <!-- Core dot -->
                            <span
                                class="relative inline-flex w-2.5 h-2.5 rounded-full transition-colors duration-300"
                                :class="{
                                    'bg-success': connectionStore.status === 'connected',
                                    'bg-warning animate-pulse': connectionStore.status === 'connecting',
                                    'bg-error': connectionStore.status === 'disconnected',
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

        <!-- Main Content -->
        <main class="ml-64 flex-1 bg-magic overflow-auto">
            <RouterView />
        </main>
    </div>
</template>

<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSystemStore } from '@/stores/system'
import { useConnectionStore } from '@/stores/connection'
import { onMounted, onUnmounted, computed } from 'vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const systemStore = useSystemStore()
const connectionStore = useConnectionStore()

const navigateImmediate = (path: string) => {
    if (route.path === path) return
    router.push(path)
}

const navItems = [
    { path: '/', name: 'Dashboard', icon: 'dashboard' },
    { path: '/processes', name: 'Processes', icon: 'process' },
    { path: '/files', name: 'Files', icon: 'folder' },
    { path: '/services', name: 'Services', icon: 'service' },
    { path: '/docker', name: 'Docker', icon: 'docker' },
    { path: '/applications', name: 'Application', icon: 'application' },
    { path: '/websites', name: 'Websites', icon: 'website' },
    { path: '/terminal', name: 'Terminal', icon: 'terminal' },
    { path: '/settings', name: 'Settings', icon: 'settings' },
]

const offset = computed(() => {
    const index = navItems.findIndex((element) => isActive(element.path))
    return `${index * 48}px`
})

const isActive = (path: string) => {
    if (path === '/') return route.path === '/'
    return route.path.startsWith(path)
}

const handleLogout = () => {
    authStore.logout()
    router.push('/login')
}

onMounted(() => {
    systemStore.fetchInfo()
    systemStore.startStreaming()
})

onUnmounted(() => {
    systemStore.stopStreaming()
})

const hostname = computed(() => systemStore.info?.hostname || 'Server')
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
