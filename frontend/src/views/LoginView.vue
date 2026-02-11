<template>
    <div
        class="min-h-screen flex items-center justify-center p-4 bg-[radial-gradient(at_27%_37%,oklch(0.94_0.04_301/0.15)_0px,transparent_50%),radial-gradient(at_97%_21%,oklch(0.94_0.06_356/0.12)_0px,transparent_50%),radial-gradient(at_52%_99%,oklch(0.96_0.08_95/0.1)_0px,transparent_50%),radial-gradient(at_10%_29%,oklch(0.92_0.06_240/0.08)_0px,transparent_50%),radial-gradient(at_97%_96%,oklch(0.94_0.04_301/0.1)_0px,transparent_50%),radial-gradient(at_33%_50%,oklch(0.96_0.06_356/0.08)_0px,transparent_50%),radial-gradient(at_79%_53%,oklch(0.97_0.08_95/0.06)_0px,transparent_50%),var(--color-surface)]"
    >
        <!-- Decorative Background Elements -->
        <div class="fixed inset-0 overflow-hidden pointer-events-none">
            <div
                class="absolute top-1/4 left-1/4 w-96 h-96 bg-reisa-lilac-500/10 rounded-full blur-3xl"
            ></div>
            <div
                class="absolute bottom-1/4 right-1/4 w-96 h-96 bg-reisa-pink-500/10 rounded-full blur-3xl"
            ></div>
            <div
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-150 h-150 bg-reisa-gold-500/5 rounded-full blur-3xl"
            ></div>
        </div>

        <!-- Login Card -->
        <div class="relative w-full max-w-md">
            <div
                class="rounded-2xl border border-[oklch(0.3_0.025_280/0.5)] bg-[oklch(0.18_0.02_280/0.8)] p-8 backdrop-blur-md animate-[fade-in_0.3s_ease-out,slide-up_0.3s_ease-out]"
            >
                <!-- Logo -->
                <div class="text-center mb-8">
                    <div
                        class="mb-4 inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-linear-to-br from-reisa-lilac-500 to-reisa-pink-500 shadow-[0_0_20px_oklch(0.66_0.058_301/0.4)]"
                    >
                        <svg
                            class="w-8 h-8 text-white"
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
                    <h1
                        class="bg-linear-to-r from-reisa-lilac-400 via-reisa-pink-400 to-reisa-gold-400 bg-clip-text text-2xl font-bold text-transparent"
                    >
                        Mana Panel
                    </h1>
                    <p class="text-text-muted mt-2">Sign in to your server</p>
                </div>

                <!-- Error Message -->
                <div
                    v-if="error"
                    class="mb-6 p-4 rounded-lg bg-error/10 border border-error/20 text-error text-sm"
                >
                    {{ error }}
                </div>

                <!-- Login Form -->
                <form @submit.prevent="handleLogin" class="space-y-5">
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-2"
                        >
                            Username
                        </label>
                        <input
                            v-model="username"
                            type="text"
                            class="w-full rounded-lg border border-border bg-surface px-4 py-3 text-sm text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                            placeholder="Enter your username"
                            required
                            autocomplete="username"
                        />
                    </div>

                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-2"
                        >
                            Password
                        </label>
                        <input
                            v-model="password"
                            type="password"
                            class="w-full rounded-lg border border-border bg-surface px-4 py-3 text-sm text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                            placeholder="Enter your password"
                            required
                            autocomplete="current-password"
                        />
                    </div>

                    <button
                        type="submit"
                        class="inline-flex h-12 w-full items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-base font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)] disabled:cursor-not-allowed disabled:opacity-50"
                        :disabled="loading"
                    >
                        <svg
                            v-if="loading"
                            class="w-5 h-5 animate-spin"
                            fill="none"
                            viewBox="0 0 24 24"
                        >
                            <circle
                                class="opacity-25"
                                cx="12"
                                cy="12"
                                r="10"
                                stroke="currentColor"
                                stroke-width="4"
                            ></circle>
                            <path
                                class="opacity-75"
                                fill="currentColor"
                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            ></path>
                        </svg>
                        <span v-else>Sign In</span>
                    </button>
                </form>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

const handleLogin = async () => {
    error.value = ''
    loading.value = true

    try {
        await authStore.login(username.value, password.value)
        router.push('/')
    } catch (e: any) {
        error.value = e.response?.data?.error?.message || 'Login failed'
    } finally {
        loading.value = false
    }
}
</script>
