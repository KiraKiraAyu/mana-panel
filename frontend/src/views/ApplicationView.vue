<template>
    <div class="p-6 space-y-6 animate-in">
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-text-primary">
                    Application
                </h1>
                <p class="text-text-muted mt-1">
                    Install and manage Docker-based applications (proxy
                    templates included)
                </p>
            </div>
            <span
                class="flex items-center gap-2 text-sm"
                :class="dockerAvailable ? 'text-success' : 'text-error'"
            >
                <span
                    class="w-2 h-2 rounded-full"
                    :class="
                        dockerAvailable
                            ? 'bg-success animate-pulse'
                            : 'bg-error'
                    "
                ></span>
                {{
                    dockerAvailable ? 'Docker Connected' : 'Docker Unavailable'
                }}
            </span>
        </div>

        <div class="card p-5 space-y-4">
            <div class="flex items-center justify-between gap-3">
                <div>
                    <h2 class="text-lg font-semibold text-text-primary">
                        Install Application
                    </h2>
                    <p class="text-sm text-text-muted mt-1">
                        Pick a template (e.g. Nginx-Standard, Caddy-AutoSSL) and
                        install as a managed instance.
                    </p>
                </div>
                <button
                    class="btn btn-ghost"
                    @click="refreshAll"
                    :disabled="loading || submitting"
                >
                    Refresh
                </button>
            </div>

            <form
                class="grid grid-cols-1 md:grid-cols-2 gap-3"
                @submit.prevent="installApplication"
            >
                <div class="space-y-1 md:col-span-2">
                    <label class="text-sm text-text-secondary">Template</label>
                    <select v-model="form.template_id" class="input">
                        <option value="" disabled>Select template</option>
                        <option
                            v-for="tpl in templates"
                            :key="tpl.id"
                            :value="tpl.id"
                        >
                            {{ tpl.name }} ({{ tpl.image }})
                        </option>
                    </select>
                </div>

                <div class="space-y-1">
                    <label class="text-sm text-text-secondary">
                        Instance Name
                        <span class="text-text-muted">(optional)</span>
                    </label>
                    <input
                        v-model="form.name"
                        class="input"
                        type="text"
                        placeholder="mana-nginx-standard"
                    />
                </div>

                <div class="space-y-1">
                    <label class="text-sm text-text-secondary">
                        Image Override
                        <span class="text-text-muted">(optional)</span>
                    </label>
                    <input
                        v-model="form.image_override"
                        class="input"
                        type="text"
                        placeholder="nginx:stable-alpine"
                    />
                </div>

                <div class="space-y-1">
                    <label class="text-sm text-text-secondary">
                        Listen HTTP Port
                    </label>
                    <input
                        v-model.number="form.listen_http_port"
                        class="input"
                        type="number"
                        min="1"
                        max="65535"
                        placeholder="80"
                    />
                </div>

                <div class="space-y-1">
                    <label class="text-sm text-text-secondary">
                        Listen HTTPS Port
                    </label>
                    <input
                        v-model.number="form.listen_https_port"
                        class="input"
                        type="number"
                        min="1"
                        max="65535"
                        placeholder="443"
                    />
                </div>

                <template v-if="selectedTemplate?.requires_upstream">
                    <div class="space-y-1">
                        <label class="text-sm text-text-secondary">
                            Upstream Host
                        </label>
                        <input
                            v-model="form.upstream_host"
                            class="input"
                            type="text"
                            placeholder="host.docker.internal"
                        />
                    </div>

                    <div class="space-y-1">
                        <label class="text-sm text-text-secondary">
                            Upstream Port
                        </label>
                        <input
                            v-model.number="form.upstream_port"
                            class="input"
                            type="number"
                            min="1"
                            max="65535"
                            placeholder="3000"
                        />
                    </div>
                </template>

                <div
                    v-if="selectedTemplate?.requires_domain"
                    class="space-y-1 md:col-span-2"
                >
                    <label class="text-sm text-text-secondary">Domain</label>
                    <input
                        v-model="form.domain"
                        class="input"
                        type="text"
                        placeholder="example.com"
                    />
                </div>

                <label
                    class="md:col-span-2 flex items-center gap-2 text-sm text-text-secondary cursor-pointer select-none"
                >
                    <input
                        v-model="form.replace_existing"
                        type="checkbox"
                        class="accent-reisa-lilac-500 w-4 h-4"
                    />
                    Replace existing running proxy applications before install
                </label>

                <div
                    v-if="validationErrors.length"
                    class="md:col-span-2 p-3 rounded-lg bg-warning/20 border border-warning/30 text-warning text-sm"
                >
                    <p class="font-medium mb-1">Please fix the following:</p>
                    <ul class="list-disc pl-4 space-y-0.5">
                        <li v-for="item in validationErrors" :key="item">
                            {{ item }}
                        </li>
                    </ul>
                </div>

                <div class="md:col-span-2 flex items-center gap-3">
                    <button
                        type="submit"
                        class="btn btn-primary"
                        :disabled="!canSubmit"
                    >
                        {{
                            pullingImage
                                ? 'Pulling image...'
                                : submitting
                                  ? 'Installing...'
                                  : 'Install Application'
                        }}
                    </button>
                </div>
            </form>

            <div
                v-if="error"
                class="p-3 rounded-lg bg-error/20 border border-error/30 text-error text-sm"
            >
                {{ error }}
            </div>

            <div
                v-if="success"
                class="p-3 rounded-lg bg-success/20 border border-success/30 text-success text-sm"
            >
                {{ success }}
            </div>

            <div
                v-if="installStatusMessage || pullProgressLines.length"
                class="p-3 rounded-lg bg-surface border border-border space-y-2"
            >
                <p class="text-sm font-medium text-text-primary">
                    {{ installStatusMessage || 'Install progress' }}
                </p>
                <ul
                    v-if="pullProgressLines.length"
                    class="max-h-44 overflow-auto space-y-1 text-xs text-text-secondary font-mono"
                >
                    <li
                        v-for="(line, idx) in pullProgressLines"
                        :key="`${idx}-${line}`"
                    >
                        {{ line }}
                    </li>
                </ul>
            </div>
        </div>

        <div class="card p-5 space-y-3">
            <h3 class="text-sm font-medium text-text-secondary">
                Installed Applications
            </h3>

            <div v-if="loading" class="text-sm text-text-muted">Loading...</div>

            <div
                v-else-if="instances.length === 0"
                class="text-sm text-text-muted"
            >
                No installed applications found.
            </div>

            <div
                v-for="app in instances"
                :key="app.id"
                class="p-3 rounded-lg border border-border bg-surface flex items-center justify-between gap-3"
            >
                <div class="min-w-0">
                    <p class="text-sm font-medium text-text-primary truncate">
                        {{ app.name || app.id.substring(0, 12) }}
                    </p>
                    <p class="text-xs text-text-muted truncate">
                        {{ app.template_id }} · {{ app.image }}
                    </p>
                    <p class="text-xs text-text-muted truncate">
                        {{ app.status }}
                    </p>
                    <p class="text-xs text-text-muted truncate">
                        ports:
                        {{
                            app.ports.length
                                ? app.ports
                                      .map((p) =>
                                          p.public_port
                                              ? `${p.public_port}:${p.private_port}`
                                              : `${p.private_port}`,
                                      )
                                      .join(', ')
                                : 'n/a'
                        }}
                    </p>
                </div>

                <div class="flex items-center gap-2">
                    <button
                        v-if="app.state !== 'running'"
                        class="btn btn-ghost"
                        @click="startApplication(app.id)"
                    >
                        Start
                    </button>
                    <button
                        v-else
                        class="btn btn-ghost"
                        @click="stopApplication(app.id)"
                    >
                        Stop
                    </button>
                    <button
                        class="btn btn-ghost text-error hover:bg-error/10"
                        @click="removeApplication(app.id, app.name)"
                    >
                        Remove
                    </button>
                </div>
            </div>
        </div>

        <div
            v-if="preflightConflicts.length > 0"
            class="card p-5 space-y-3 border border-warning/40"
        >
            <h3 class="text-sm font-semibold text-warning">
                Port conflict detected
            </h3>
            <p class="text-sm text-text-secondary">
                Starting this application would conflict with existing port
                bindings:
            </p>
            <ul class="list-disc pl-5 space-y-1 text-sm text-text-secondary">
                <li
                    v-for="c in preflightConflicts"
                    :key="`${c.port}-${c.occupied_by}`"
                >
                    Port {{ c.port }} is occupied by {{ c.occupied_by }}
                    <span v-if="c.occupant_image">({{ c.occupant_image }})</span
                    >.
                    {{ c.suggestion }}
                </li>
            </ul>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
    applicationsApi,
    type ApplicationInstance,
    type ApplicationTemplate,
    type InstallApplicationRequest,
    type PortConflict,
    type InstallPullProgress,
} from '@/api/applications'

const loading = ref(false)
const submitting = ref(false)
const pullingImage = ref(false)
const dockerAvailable = ref(true)
const error = ref('')
const success = ref('')
const installStatusMessage = ref('')
const pullProgressLines = ref<string[]>([])
const pullProgressIndexByKey = ref<Record<string, number>>({})

const templates = ref<ApplicationTemplate[]>([])
const instances = ref<ApplicationInstance[]>([])
const preflightConflicts = ref<PortConflict[]>([])

const form = ref({
    template_id: '',
    name: '',
    image_override: '',
    upstream_host: 'host.docker.internal',
    upstream_port: 3000,
    domain: '',
    listen_http_port: 80,
    listen_https_port: 443,
    replace_existing: false,
})

const selectedTemplate = computed(() =>
    templates.value.find((t) => t.id === form.value.template_id),
)

const validationErrors = computed(() => {
    const errors: string[] = []
    if (!form.value.template_id) errors.push('Template is required')

    const hp = Number(form.value.listen_http_port)
    const sp = Number(form.value.listen_https_port)
    if (!Number.isInteger(hp) || hp < 1 || hp > 65535) {
        errors.push('Listen HTTP Port must be between 1 and 65535')
    }
    if (!Number.isInteger(sp) || sp < 1 || sp > 65535) {
        errors.push('Listen HTTPS Port must be between 1 and 65535')
    }

    if (selectedTemplate.value?.requires_upstream) {
        if (!form.value.upstream_host.trim()) {
            errors.push('Upstream Host is required for this template')
        }
        const up = Number(form.value.upstream_port)
        if (!Number.isInteger(up) || up < 1 || up > 65535) {
            errors.push('Upstream Port must be between 1 and 65535')
        }
    }

    if (selectedTemplate.value?.requires_domain && !form.value.domain.trim()) {
        errors.push('Domain is required for this template')
    }

    return errors
})

const canSubmit = computed(
    () =>
        !submitting.value &&
        !pullingImage.value &&
        validationErrors.value.length === 0,
)

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
        if (
            e.response?.data?.error?.message
                ?.toLowerCase()
                .includes('docker is not available')
        ) {
            dockerAvailable.value = false
        }
        error.value =
            e.response?.data?.error?.message || 'Failed to load applications'
    } finally {
        loading.value = false
    }
}

const formatPullProgress = (item: InstallPullProgress): string => {
    const parts = [item.status, item.id, item.progress].filter(Boolean)
    return parts.length ? parts.join(' | ') : 'pulling...'
}

const pullProgressKey = (item: InstallPullProgress): string => {
    const status = (item.status || '').trim()
    const id = (item.id || '').trim()
    if (id) return `${status}|${id}`
    return status || 'generic'
}

const upsertPullProgressLine = (item: InstallPullProgress) => {
    const key = pullProgressKey(item)
    const line = formatPullProgress(item)
    const existingIndex = pullProgressIndexByKey.value[key]

    if (existingIndex === undefined) {
        pullProgressIndexByKey.value[key] = pullProgressLines.value.length
        pullProgressLines.value.push(line)
        return
    }

    if (pullProgressLines.value[existingIndex] !== line) {
        pullProgressLines.value[existingIndex] = line
    }
}

const streamImagePullProgress = async (image: string): Promise<void> => {
    if (!image.trim()) return

    pullingImage.value = true
    installStatusMessage.value = `Pulling image ${image}...`
    pullProgressLines.value = [`Pulling image: ${image}`]
    pullProgressIndexByKey.value = {}

    await new Promise<void>((resolve, reject) => {
        const eventSource = new EventSource(
            `/api/applications/image-pull/stream?image=${encodeURIComponent(image)}`,
        )

        let settled = false
        const finalize = (fn: () => void) => {
            if (settled) return
            settled = true
            eventSource.close()
            fn()
        }

        eventSource.addEventListener('progress', (event) => {
            try {
                const payload = JSON.parse(
                    (event as MessageEvent).data,
                ) as InstallPullProgress
                upsertPullProgressLine(payload)
            } catch {
                const fallback =
                    (event as MessageEvent).data || 'progress update'
                const last =
                    pullProgressLines.value[pullProgressLines.value.length - 1]
                if (last !== fallback) {
                    pullProgressLines.value.push(fallback)
                }
            }
        })

        eventSource.addEventListener('done', () => {
            installStatusMessage.value = `Image pull completed: ${image}`
            finalize(resolve)
        })

        eventSource.addEventListener('error', (event) => {
            try {
                const payload = JSON.parse(
                    (event as MessageEvent).data || '{}',
                ) as {
                    message?: string
                }
                finalize(() =>
                    reject(
                        new Error(
                            payload.message || `Image pull failed for ${image}`,
                        ),
                    ),
                )
            } catch {
                finalize(() =>
                    reject(new Error(`Image pull failed for ${image}`)),
                )
            }
        })

        eventSource.onerror = () => {
            finalize(() =>
                reject(
                    new Error(`Image pull stream disconnected for ${image}`),
                ),
            )
        }
    })
}

const installApplication = async () => {
    submitting.value = true
    error.value = ''
    success.value = ''
    preflightConflicts.value = []
    installStatusMessage.value = ''
    pullProgressLines.value = []
    pullProgressIndexByKey.value = {}

    try {
        if (validationErrors.value.length > 0) {
            error.value = validationErrors.value[0] || 'Invalid form input'
            return
        }

        const payload: InstallApplicationRequest = {
            template_id: form.value.template_id,
            name: form.value.name.trim() || undefined,
            image_override: form.value.image_override.trim() || undefined,
            upstream_host: form.value.upstream_host.trim() || undefined,
            upstream_port: Number(form.value.upstream_port) || undefined,
            domain: form.value.domain.trim() || undefined,
            listen_http_port: Number(form.value.listen_http_port) || undefined,
            listen_https_port:
                Number(form.value.listen_https_port) || undefined,
            replace_existing: form.value.replace_existing || undefined,
        }

        const installImage =
            payload.image_override || selectedTemplate.value?.image || ''

        if (installImage.trim()) {
            const exists = await applicationsApi.imageExists(installImage)
            if (!exists) {
                await streamImagePullProgress(installImage)
            } else {
                installStatusMessage.value = `Image already exists locally: ${installImage}`
                pullProgressLines.value = [
                    `Skip pull (local image found): ${installImage}`,
                ]
            }
        }

        installStatusMessage.value = 'Creating container...'
        const result = await applicationsApi.install(payload)

        if (result.pull_progress?.length) {
            pullProgressLines.value.push(
                ...result.pull_progress.map((item) => formatPullProgress(item)),
            )
        }

        installStatusMessage.value = 'Install completed'
        success.value = result.action?.message || 'Application installed'
        await refreshAll()
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to install application'
    } finally {
        pullingImage.value = false
        submitting.value = false
    }
}

const startApplication = async (id: string) => {
    error.value = ''
    success.value = ''
    preflightConflicts.value = []

    try {
        const preflight = await applicationsApi.preflightStart(id)
        if (!preflight.ok) {
            preflightConflicts.value = preflight.conflicts || []
            error.value =
                'Start blocked: required port(s) are already in use. Stop the conflicting app or change listening ports.'
            return
        }

        const result = await applicationsApi.start(id)
        success.value = result.message || 'Application started'
        await refreshAll()
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to start application'
    }
}

const stopApplication = async (id: string) => {
    error.value = ''
    success.value = ''
    try {
        const result = await applicationsApi.stop(id)
        success.value = result.message || 'Application stopped'
        await refreshAll()
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to stop application'
    }
}

const removeApplication = async (id: string, name: string) => {
    if (!confirm(`Remove application "${name || id.substring(0, 12)}"?`)) return
    error.value = ''
    success.value = ''
    try {
        const result = await applicationsApi.remove(id, true)
        success.value = result.message || 'Application removed'
        await refreshAll()
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to remove application'
    }
}

onMounted(() => {
    refreshAll()
})
</script>
