import { computed, ref, watch, type Ref } from 'vue'

import type {
    ApplicationInstallValue,
    ApplicationTemplate,
    ApplicationTemplateParam,
} from '@/api/applications'

export interface InstallFormState {
    template_id: string
    name: string
    values: Record<string, string>
    port_bindings: Record<string, number | undefined>
    extra_port_bindings: Record<string, number | undefined>
    env_overrides: Record<string, string>
}

export interface InstallPortField {
    key: string
    endpoint: string
    required: boolean
    default_host_port: number | null
}

interface UseApplicationInstallFormOptions {
    templates: Ref<ApplicationTemplate[]>
    submitting: Ref<boolean>
    error?: Ref<string>
    success?: Ref<string>
    installStatusMessage?: Ref<string>
}

const createInitialFormState = (): InstallFormState => ({
    template_id: '',
    name: '',
    values: {},
    port_bindings: {},
    extra_port_bindings: {},
    env_overrides: {},
})

export function useApplicationInstallForm(
    options: UseApplicationInstallFormOptions,
) {
    const installModalOpen = ref(false)
    const form = ref<InstallFormState>(createInitialFormState())

    const selectedTemplate = computed(() =>
        options.templates.value.find((t) => t.id === form.value.template_id),
    )

    const paramFields = computed<ApplicationTemplateParam[]>(
        () => selectedTemplate.value?.params || [],
    )

    const portFields = computed<InstallPortField[]>(() =>
        (selectedTemplate.value?.ports || []).map((p) => {
            const protocol =
                (p.protocol || 'tcp').toLowerCase() === 'udp' ? 'udp' : 'tcp'

            return {
                key: p.key,
                endpoint: `${p.container_port}/${protocol}`,
                required: !!p.required,
                default_host_port: p.default_host_port ?? null,
            }
        }),
    )

    watch(
        selectedTemplate,
        (tpl) => {
            if (!tpl) return

            const nextValues = { ...form.value.values }
            for (const p of tpl.params || []) {
                if (nextValues[p.key] !== undefined) continue

                if (p.default_value != null && p.default_value !== '') {
                    nextValues[p.key] = String(p.default_value)
                } else if (p.input === 'boolean') {
                    nextValues[p.key] = 'false'
                } else if (p.input === 'select' && p.options?.length) {
                    nextValues[p.key] = p.options![0] ?? ''
                } else {
                    nextValues[p.key] = ''
                }
            }
            form.value.values = nextValues

            const nextBindings = { ...form.value.port_bindings }
            for (const p of tpl.ports || []) {
                if (nextBindings[p.key] !== undefined) continue
                if (p.default_host_port)
                    nextBindings[p.key] = p.default_host_port
            }
            form.value.port_bindings = nextBindings
        },
        { immediate: true },
    )

    const validationErrors = computed(() => {
        const errors: string[] = []

        if (!form.value.template_id) errors.push('Template is required')

        for (const p of paramFields.value) {
            if (!p.required) continue
            if (p.input === 'boolean') continue

            const raw = form.value.values[p.key]
            if (!String(raw ?? '').trim()) {
                errors.push(`${p.label || p.key} is required`)
            }
        }

        for (const port of portFields.value) {
            const raw = form.value.port_bindings[port.key]
            const configured = Number(raw)
            const hasConfigured =
                Number.isInteger(configured) &&
                configured >= 1 &&
                configured <= 65535

            if (
                raw !== undefined &&
                raw !== null &&
                raw !== ('' as unknown as number) &&
                !hasConfigured
            ) {
                errors.push(
                    `Host port for ${port.key} must be between 1 and 65535`,
                )
            }

            if (!hasConfigured && port.required && !port.default_host_port) {
                errors.push(
                    `Host port for ${port.key} (${port.endpoint}) is required`,
                )
            }
        }

        for (const [endpoint, raw] of Object.entries(
            form.value.extra_port_bindings,
        )) {
            const endpointNormalized = String(endpoint).trim().toLowerCase()
            if (!/^\d+\/(tcp|udp)$/.test(endpointNormalized)) {
                errors.push(
                    `Custom port key "${endpoint}" must match "<containerPort>/tcp|udp"`,
                )
                continue
            }

            const configured = Number(raw)
            const hasConfigured =
                Number.isInteger(configured) &&
                configured >= 1 &&
                configured <= 65535

            if (
                raw !== undefined &&
                raw !== null &&
                raw !== ('' as unknown as number) &&
                !hasConfigured
            ) {
                errors.push(
                    `Custom host port for ${endpoint} must be between 1 and 65535`,
                )
            }
        }

        for (const [key, value] of Object.entries(form.value.env_overrides)) {
            const envKey = String(key).trim()
            if (!envKey) continue
            if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(envKey)) {
                errors.push(`Environment variable name "${key}" is invalid`)
            }
            if (String(value ?? '').includes('\n')) {
                errors.push(
                    `Environment variable "${key}" cannot contain new lines`,
                )
            }
        }

        return errors
    })

    const canSubmit = computed(
        () => !options.submitting.value && validationErrors.value.length === 0,
    )

    const clearMessages = () => {
        if (options.error) options.error.value = ''
        if (options.success) options.success.value = ''
        if (options.installStatusMessage)
            options.installStatusMessage.value = ''
    }

    const openInstallModal = (tpl: ApplicationTemplate) => {
        clearMessages()
        form.value = {
            ...createInitialFormState(),
            template_id: tpl.id,
        }
        installModalOpen.value = true
    }

    const closeInstallModal = () => {
        if (options.submitting.value) return
        installModalOpen.value = false
        if (options.installStatusMessage)
            options.installStatusMessage.value = ''
    }

    const buildTypedValues = ():
        | Record<string, ApplicationInstallValue>
        | undefined => {
        const out: Record<string, ApplicationInstallValue> = {}

        for (const p of paramFields.value) {
            const raw = form.value.values[p.key]
            if (raw === undefined || raw === null) continue

            const rawString = String(raw).trim()
            if (!rawString && p.input !== 'boolean') continue

            if (p.input === 'boolean') {
                out[p.key] = rawString.toLowerCase() === 'true'
                continue
            }

            if (p.input === 'number') {
                const n = Number(rawString)
                if (Number.isFinite(n)) out[p.key] = n
                continue
            }

            out[p.key] = rawString
        }

        return Object.keys(out).length ? out : undefined
    }

    const buildPortBindings = (): Record<string, number> | undefined => {
        const out: Record<string, number> = {}

        for (const p of portFields.value) {
            const configured = Number(form.value.port_bindings[p.key])
            if (
                Number.isInteger(configured) &&
                configured >= 1 &&
                configured <= 65535
            ) {
                out[p.key] = configured
                continue
            }

            if (p.default_host_port) out[p.key] = p.default_host_port
        }

        for (const [endpoint, raw] of Object.entries(
            form.value.extra_port_bindings,
        )) {
            const endpointNormalized = String(endpoint).trim().toLowerCase()
            const configured = Number(raw)
            if (
                /^\d+\/(tcp|udp)$/.test(endpointNormalized) &&
                Number.isInteger(configured) &&
                configured >= 1 &&
                configured <= 65535
            ) {
                out[endpointNormalized] = configured
            }
        }

        return Object.keys(out).length ? out : undefined
    }

    const buildEnvOverrides = (): string[] | undefined => {
        const out: string[] = []

        for (const [key, value] of Object.entries(form.value.env_overrides)) {
            const k = String(key).trim()
            if (!k) continue
            if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(k)) continue

            const v = String(value ?? '').trim()
            out.push(`${k}=${v}`)
        }

        return out.length ? out : undefined
    }

    return {
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
    }
}
