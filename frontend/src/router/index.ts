import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/login',
            name: 'login',
            component: () => import('@/views/LoginView.vue'),
            meta: { guest: true },
        },
        {
            path: '/',
            component: () => import('@/layouts/MainLayout.vue'),
            meta: { requiresAuth: true },
            children: [
                {
                    path: '',
                    name: 'dashboard',
                    component: () => import('@/views/DashboardView.vue'),
                },
                {
                    path: 'processes',
                    name: 'processes',
                    component: () => import('@/views/ProcessesView.vue'),
                },
                {
                    path: 'files',
                    name: 'files',
                    component: () => import('@/views/FilesView.vue'),
                },
                {
                    path: 'services',
                    name: 'services',
                    component: () => import('@/views/ServicesView.vue'),
                },
                {
                    path: 'applications',
                    name: 'applications',
                    component: () => import('@/views/ApplicationView.vue'),
                },
                {
                    path: 'docker',
                    name: 'docker',
                    component: () => import('@/views/DockerView.vue'),
                },
                {
                    path: 'terminal',
                    name: 'terminal',
                    component: () => import('@/views/TerminalView.vue'),
                },
                {
                    path: 'settings',
                    name: 'settings',
                    component: () => import('@/views/SettingsView.vue'),
                },
            ],
        },
    ],
})

router.beforeEach(async (to, _from, next) => {
    const authStore = useAuthStore()

    if (!authStore.isInitialized) {
        await authStore.init()
    }

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next({ name: 'login' })
    } else if (to.meta.guest && authStore.isAuthenticated) {
        next({ name: 'dashboard' })
    } else {
        next()
    }
})

export default router
