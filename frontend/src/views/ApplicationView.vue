<template>
    <div
        class="max-w-7xl mx-auto p-4 md:p-6 space-y-8 animate-in fade-in duration-500"
    >
        <ApplicationHeader
            :loading="loading"
            :submitting="submitting"
            :docker-available="dockerAvailable"
            @refresh="refreshAll"
        />

        <div class="flex justify-end">
            <RouterLink
                to="/applications/tasks"
                class="btn btn-sm btn-ghost border border-border hover:border-reisa-lilac-500/40 text-text-secondary hover:text-text-primary"
            >
                View Task Output
            </RouterLink>
        </div>

        <ApplicationTabs
            :active-tab="activeTab"
            :installed-count="instances.length"
            :available-count="templates.length"
            @update:active-tab="activeTab = $event"
        />

        <TransitionGroup name="list" tag="div" class="space-y-2">
            <div
                v-if="error && !installModalOpen"
                key="error"
                class="p-4 rounded-xl bg-error/10 border border-error/20 text-error text-sm flex items-center gap-3"
            >
                <svg
                    class="w-5 h-5 shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>
                {{ error }}
            </div>

            <div
                v-if="success"
                key="success"
                class="p-4 rounded-xl bg-success/10 border border-success/20 text-success text-sm flex items-center gap-3"
            >
                <svg
                    class="w-5 h-5 shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M5 13l4 4L19 7"
                    />
                </svg>
                {{ success }}
            </div>
        </TransitionGroup>

        <Transition name="slide-fade" mode="out-in">
            <div
                v-if="loading && !templates.length && !instances.length"
                class="h-64 flex flex-col items-center justify-center text-text-muted space-y-4"
            >
                <div
                    class="w-8 h-8 border-4 border-reisa-lilac-500/30 border-t-reisa-lilac-500 rounded-full animate-spin"
                ></div>
                <p>Loading...</p>
            </div>

            <AvailableTemplatesTab
                v-else-if="activeTab === 'available'"
                key="available"
                :templates="templates"
                :loading="loading && !templates.length"
                @install="openInstallModal"
            />

            <InstalledApplicationsTab
                v-else
                key="installed"
                :instances="instances"
                :loading="loading || submitting"
                @start="startApplication"
                @stop="stopApplication"
                @update="updateApplication"
                @remove="removeApplication"
                @logs="openLogsModal"
                @env="openEnvModal"
                @switch-to-available="activeTab = 'available'"
            />
        </Transition>

        <InstallApplicationModal
            :open="installModalOpen"
            :template="selectedTemplate || null"
            :form="form"
            :param-fields="paramFields"
            :port-fields="portFields"
            :submitting="submitting"
            :error="error"
            :validation-errors="validationErrors"
            :status-message="installStatusMessage"
            :can-submit="canSubmit"
            @close="closeInstallModal"
            @submit="installApplication"
        />

        <ApplicationLogsModal
            :open="logsModalOpen"
            :app-id="logsAppId"
            :app-name="logsAppName"
            @close="logsModalOpen = false"
        />

        <ApplicationEnvModal
            :open="envModalOpen"
            :app-id="envAppId"
            :app-name="envAppName"
            @close="envModalOpen = false"
            @updated="refreshAll"
        />
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'

import {
    applicationsApi,
    type InstallApplicationRequest,
} from '@/api/applications'
import ApplicationHeader from '@/components/applications/ApplicationHeader.vue'
import ApplicationTabs from '@/components/applications/ApplicationTabs.vue'
import AvailableTemplatesTab from '@/components/applications/AvailableTemplatesTab.vue'
import InstallApplicationModal from '@/components/applications/InstallApplicationModal.vue'
import ApplicationLogsModal from '@/components/applications/ApplicationLogsModal.vue'
import ApplicationEnvModal from '@/components/applications/ApplicationEnvModal.vue'
import InstalledApplicationsTab from '@/components/applications/InstalledApplicationsTab.vue'
import { useApplicationInstallForm } from '@/composables/useApplicationInstallForm'
import { useApplicationsData } from '@/composables/useApplicationsData'

type ApplicationTab = 'installed' | 'available'

const activeTab = ref<ApplicationTab>('available')
const submitting = ref(false)
const installStatusMessage = ref('')

const logsModalOpen = ref(false)
const logsAppId = ref<string | null>(null)
const logsAppName = ref<string | null>(null)

const openLogsModal = (id: string, name: string) => {
    logsAppId.value = id
    logsAppName.value = name
    logsModalOpen.value = true
}

const envModalOpen = ref(false)
const envAppId = ref<string | null>(null)
const envAppName = ref<string | null>(null)

const openEnvModal = (id: string, name: string) => {
    envAppId.value = id
    envAppName.value = name
    envModalOpen.value = true
}

const {
    loading,
    dockerAvailable,
    error,
    success,
    templates,
    instances,
    refreshAll,
    startApplication,
    stopApplication,
    updateApplication,
    removeApplication,
} = useApplicationsData()

const {
    installModalOpen,
    form,
    selectedTemplate,
    paramFields,
    portFields,
    validationErrors,
    canSubmit,
    openInstallModal,
    closeInstallModal,
    buildTypedValues,
    buildPortBindings,
    buildEnvOverrides,
} = useApplicationInstallForm({
    templates,
    submitting,
    error,
    success,
    installStatusMessage,
})

const installApplication = async () => {
    submitting.value = true
    error.value = ''
    success.value = ''
    installStatusMessage.value = ''

    try {
        if (validationErrors.value.length > 0) {
            error.value = validationErrors.value[0] || 'Invalid form input'
            return
        }

        const payload: InstallApplicationRequest = {
            template_id: form.value.template_id,
            name: form.value.name.trim() || undefined,
            typed_values: buildTypedValues(),
            port_bindings: buildPortBindings(),
            env: buildEnvOverrides(),
        }

        // Close immediately after submit; task output is tracked in dedicated task view.
        installModalOpen.value = false
        installStatusMessage.value = ''

        const task = await applicationsApi.install(payload)
        success.value = `Install task ${task.id.slice(0, 8)} created. Open "View Task Output" to monitor.`
    } catch (e: any) {
        error.value =
            e?.message ||
            e.response?.data?.error?.message ||
            'Failed to install application'
    } finally {
        submitting.value = false
    }
}
</script>

<style scoped>
.slide-fade-enter-active,
.slide-fade-leave-active {
    transition: all 0.25s ease-out;
}

.slide-fade-enter-from {
    opacity: 0;
    transform: translateY(10px);
}

.slide-fade-leave-to {
    opacity: 0;
    transform: translateY(-10px);
}

.list-enter-active,
.list-leave-active {
    transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(-10px);
}
</style>
