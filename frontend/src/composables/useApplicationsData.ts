import { onMounted, ref } from 'vue'

import {
    applicationsApi,
    type ApplicationInstance,
    type ApplicationTemplate,
} from '@/api/applications'

export interface UseApplicationsDataOptions {
    autoLoad?: boolean
}

export const useApplicationsData = (
    options: UseApplicationsDataOptions = {},
) => {
    const { autoLoad = true } = options

    const loading = ref(false)
    const dockerAvailable = ref(true)
    const error = ref('')
    const success = ref('')
    const templates = ref<ApplicationTemplate[]>([])
    const instances = ref<ApplicationInstance[]>([])

    const clearFeedback = () => {
        error.value = ''
        success.value = ''
    }

    const refreshAll = async () => {
        loading.value = true
        error.value = ''

        try {
            const [tpls, insts] = await Promise.all([
                applicationsApi.listTemplates(),
                applicationsApi.listInstances(),
            ])

            templates.value = tpls
            instances.value = insts
            dockerAvailable.value = true
        } catch (e: any) {
            const message: string =
                e?.response?.data?.error?.message || 'Failed to load applications'

            if (message.toLowerCase().includes('docker is not available')) {
                dockerAvailable.value = false
            }

            error.value = message
        } finally {
            loading.value = false
        }
    }

    const startApplication = async (id: string) => {
        clearFeedback()

        try {
            const result = await applicationsApi.start(id)
            success.value = result.message || 'Application started'
            await refreshAll()
            return true
        } catch (e: any) {
            error.value =
                e?.response?.data?.error?.message || 'Failed to start application'
            return false
        }
    }

    const stopApplication = async (id: string) => {
        clearFeedback()

        try {
            const result = await applicationsApi.stop(id)
            success.value = result.message || 'Application stopped'
            await refreshAll()
            return true
        } catch (e: any) {
            error.value =
                e?.response?.data?.error?.message || 'Failed to stop application'
            return false
        }
    }

    const removeApplication = async (
        id: string,
        name: string,
        requireConfirmation: boolean = true,
    ) => {
        if (
            requireConfirmation &&
            typeof window !== 'undefined' &&
            !window.confirm(`Remove application "${name || id.substring(0, 12)}"?`)
        ) {
            return false
        }

        clearFeedback()

        try {
            const result = await applicationsApi.remove(id, true)
            success.value = result.message || 'Application removed'
            await refreshAll()
            return true
        } catch (e: any) {
            error.value =
                e?.response?.data?.error?.message || 'Failed to remove application'
            return false
        }
    }

    if (autoLoad) {
        onMounted(() => {
            refreshAll()
        })
    }

    return {
        loading,
        dockerAvailable,
        error,
        success,
        templates,
        instances,
        clearFeedback,
        refreshAll,
        startApplication,
        stopApplication,
        removeApplication,
    }
}
