<template>
    <div class="p-6 space-y-6 animate-in">
        <!-- Header -->
        <div>
            <h1 class="text-2xl font-bold text-text-primary">Settings</h1>
            <p class="text-text-muted mt-1">
                Manage your account and preferences
            </p>
        </div>

        <!-- Account Section -->
        <div class="card">
            <h2 class="text-lg font-semibold text-text-primary mb-4">
                Account
            </h2>

            <div class="space-y-4">
                <div
                    class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
                >
                    <h2 class="text-lg font-semibold text-text-primary mb-4">
                        Account
                    </h2>

                    <div class="space-y-4">
                        <div
                            class="flex items-center justify-between py-3 border-b border-border-subtle"
                        >
                            <div>
                                <p class="text-text-primary font-medium">
                                    Username
                                </p>
                                <p class="text-text-muted text-sm">
                                    Your account username
                                </p>
                            </div>
                            <span class="text-text-secondary font-mono">{{
                                authStore.user?.username || 'admin'
                            }}</span>
                        </div>

                        <div class="flex items-center justify-between py-3">
                            <div>
                                <p class="text-text-primary font-medium">
                                    User ID
                                </p>
                                <p class="text-text-muted text-sm">
                                    Your unique identifier
                                </p>
                            </div>
                            <span class="text-text-secondary font-mono">{{
                                authStore.user?.id || '1'
                            }}</span>
                        </div>
                    </div>
                </div>

                <div class="flex items-center justify-between py-3">
                    <div>
                        <p class="text-text-primary font-medium">User ID</p>
                        <p class="text-text-muted text-sm">
                            Your unique identifier
                        </p>
                    </div>
                    <span class="text-text-secondary font-mono">{{
                        authStore.user?.id || '1'
                    }}</span>
                </div>
            </div>
        </div>

        <!-- Change Password Section -->
        <div class="card">
            <h2 class="text-lg font-semibold text-text-primary mb-4">
                Change Password
            </h2>

            <form @submit.prevent="changePassword" class="space-y-4 max-w-md">
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Current Password
                    </label>
                    <input
                        v-model="currentPassword"
                        type="password"
                        class="input"
                        placeholder="Enter current password"
                        required
                    />
                </div>

                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        New Password
                    </label>
                    <input
                        v-model="newPassword"
                        type="password"
                        class="input"
                        placeholder="Enter new password"
                        required
                    />
                </div>

                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Confirm New Password
                    </label>
                    <input
                        v-model="confirmPassword"
                        type="password"
                        class="input"
                        placeholder="Confirm new password"
                        required
                    />
                </div>

                <div
                    class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
                >
                    <h2 class="text-lg font-semibold text-text-primary mb-4">
                        Change Password
                    </h2>

                    <form
                        @submit.prevent="changePassword"
                        class="space-y-4 max-w-md"
                    >
                        <div>
                            <label
                                class="block text-sm font-medium text-text-secondary mb-1"
                            >
                                Current Password
                            </label>
                            <input
                                v-model="currentPassword"
                                type="password"
                                class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Enter current password"
                                required
                            />
                        </div>

                        <div>
                            <label
                                class="block text-sm font-medium text-text-secondary mb-1"
                            >
                                New Password
                            </label>
                            <input
                                v-model="newPassword"
                                type="password"
                                class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Enter new password"
                                required
                            />
                        </div>

                        <div>
                            <label
                                class="block text-sm font-medium text-text-secondary mb-1"
                            >
                                Confirm New Password
                            </label>
                            <input
                                v-model="confirmPassword"
                                type="password"
                                class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Confirm new password"
                                required
                            />
                        </div>

                        <div
                            v-if="passwordError"
                            class="p-3 rounded-lg bg-error/20 text-error text-sm"
                        >
                            {{ passwordError }}
                        </div>

                        <div
                            v-if="passwordSuccess"
                            class="p-3 rounded-lg bg-success/20 text-success text-sm"
                        >
                            {{ passwordSuccess }}
                        </div>

                        <button
                            type="submit"
                            class="inline-flex items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)] disabled:cursor-not-allowed disabled:opacity-50"
                            :disabled="passwordLoading"
                        >
                            <svg
                                v-if="passwordLoading"
                                class="w-4 h-4 animate-spin"
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
                            Change Password
                        </button>
                    </form>
                </div>

                <!-- About Section -->
                <div
                    class="rounded-xl border border-border-subtle bg-surface-elevated p-6 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
                >
                    <h2 class="text-lg font-semibold text-text-primary mb-4">
                        About
                    </h2>

                    <div class="space-y-3">
                        <div class="flex items-center justify-between py-2">
                            <span class="text-text-secondary">Version</span>
                            <span class="text-text-primary font-mono">{{
                                systemVersion
                            }}</span>
                        </div>

                        <div class="flex items-center justify-between py-2">
                            <span class="text-text-secondary">Project</span>
                            <span class="text-text-primary">Mana Panel</span>
                        </div>

                        <div class="flex items-center justify-between py-2">
                            <span class="text-text-secondary">License</span>
                            <span class="text-text-primary">MIT</span>
                        </div>
                    </div>

                    <div class="mt-6 pt-4 border-t border-border-subtle">
                        <p class="text-text-muted text-sm">
                            A lightweight, high-performance Linux operations
                            panel built with Rust and Vue 3.
                        </p>
                    </div>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary"
                    :disabled="passwordLoading"
                >
                    <svg
                        v-if="passwordLoading"
                        class="w-4 h-4 animate-spin"
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
                    Change Password
                </button>
            </form>
        </div>

        <!-- About Section -->
        <div class="card">
            <h2 class="text-lg font-semibold text-text-primary mb-4">About</h2>

            <div class="space-y-3">
                <div class="flex items-center justify-between py-2">
                    <span class="text-text-secondary">Version</span>
                    <span class="text-text-primary font-mono">{{
                        systemVersion
                    }}</span>
                </div>

                <div class="flex items-center justify-between py-2">
                    <span class="text-text-secondary">Project</span>
                    <span class="text-text-primary">Mana Panel</span>
                </div>

                <div class="flex items-center justify-between py-2">
                    <span class="text-text-secondary">License</span>
                    <span class="text-text-primary">MIT</span>
                </div>
            </div>

            <div class="mt-6 pt-4 border-t border-border-subtle">
                <p class="text-text-muted text-sm">
                    A lightweight, high-performance Linux operations panel built
                    with Rust and Vue 3.
                </p>
            </div>
        </div>

        <!-- Danger Zone -->
        <div class="card border border-error/30">
            <h2 class="text-lg font-semibold text-error mb-4">Danger Zone</h2>

            <div class="flex items-center justify-between">
                <div>
                    <p class="text-text-primary font-medium">Logout</p>
                    <p class="text-text-muted text-sm">
                        Sign out of your account
                    </p>
                </div>
                <button @click="authStore.logout()" class="btn btn-danger">
                    Logout
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/api'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

// Password change form
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordLoading = ref(false)
const passwordError = ref('')
const passwordSuccess = ref('')

const changePassword = async () => {
    passwordError.value = ''
    passwordSuccess.value = ''

    if (newPassword.value !== confirmPassword.value) {
        passwordError.value = 'New passwords do not match'
        return
    }

    if (newPassword.value.length < 6) {
        passwordError.value = 'Password must be at least 6 characters'
        return
    }

    passwordLoading.value = true
    try {
        await api.post('/auth/password', {
            current_password: currentPassword.value,
            new_password: newPassword.value,
        })
        passwordSuccess.value = 'Password changed successfully'
        currentPassword.value = ''
        newPassword.value = ''
        confirmPassword.value = ''
    } catch (e: any) {
        passwordError.value =
            e.response?.data?.error?.message || 'Failed to change password'
    } finally {
        passwordLoading.value = false
    }
}

// System info
const systemVersion = ref('0.1.0')
</script>
