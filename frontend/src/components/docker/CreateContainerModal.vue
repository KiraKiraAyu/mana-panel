<template>
    <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="$emit('close')"
    >
        -1 font-mono text-sm" ut">
        <div
            class="w-full max-w-2xl bg-surface-elevated rounded-xl shadow-2xl animate-in max-h-[85vh] overflow-y-auto"
        >
            <div
                class="flex items-center justify-between p-4 border-b border-border-subtle sticky top-0 bg-surface-elevated z-10"
            >
                <h3 class="font-semibold text-text-primary">
                    Create Container
                </h3>
                <button
                    @click="$emit('close')"
                    class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="creating"
                >
                    Close
                </button>
            </div>

            <form @submit.prevent="submit" class="p-6 space-y-5">
                <!-- Name -->
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Container Name
                        <span class="text-text-muted font-normal"
                            >(optional)</span
                        >
                    </label>
                    <input
                        v-model="form.name"
                        type="text"
                        class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                        placeholder="e.g. my-nginx"
                    />
                </div>

                <!-- Image -->
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Image
                        <span class="text-error">*</span>
                    </label>
                    <input
                        v-model="form.image"
                        type="text"
                        class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                        placeholder="e.g. nginx:latest"
                        required
                    />
                </div>

                <!-- Command -->
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Command
                        <span class="text-text-muted font-normal"
                            >(optional, space-separated)</span
                        >
                    </label>
                    <input
                        v-model="cmdRaw"
                        type="text"
                        class="w-full px-4 py-3 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                        placeholder="e.g. /bin/sh -c 'echo hello'"
                    />
                </div>

                <!-- Environment Variables -->
                <div>
                    <div class="flex items-center justify-between mb-1">
                        <label
                            class="block text-sm font-medium text-text-secondary"
                        >
                            Environment Variables
                        </label>
                        <button
                            type="button"
                            @click="addEnv"
                            class="text-xs text-reisa-lilac-400 hover:text-reisa-lilac-300 transition-colors"
                        >
                            + Add
                        </button>
                    </div>
                    <div class="space-y-2">
                        <div
                            v-for="(env, idx) in envPairs"
                            :key="idx"
                            class="flex items-center gap-2"
                        >
                            <input
                                v-model="env.key"
                                type="text"
                                class="flex-1 px-4 py-3 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="KEY"
                            />
                            <span class="text-text-muted">=</span>
                            <input
                                v-model="env.value"
                                type="text"
                                class="flex-1 px-4 py-3 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="value"
                            />
                            <button
                                type="button"
                                @click="envPairs.splice(idx, 1)"
                                class="p-1.5 rounded-lg hover:bg-error/20 text-error transition-colors shrink-0"
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
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                            </button>
                        </div>
                        <p
                            v-if="envPairs.length === 0"
                            class="text-xs text-text-muted"
                        >
                            No environment variables configured.
                        </p>
                    </div>
                </div>

                <!-- Port Mappings -->
                <div>
                    <div class="flex items-center justify-between mb-1">
                        <label
                            class="block text-sm font-medium text-text-secondary"
                        >
                            Port Mappings
                        </label>
                        <button
                            type="button"
                            @click="addPort"
                            class="text-xs text-reisa-lilac-400 hover:text-reisa-lilac-300 transition-colors"
                        >
                            + Add
                        </button>
                    </div>
                    <div class="space-y-2">
                        <div
                            v-for="(pm, idx) in portMappings"
                            :key="idx"
                            class="flex items-center gap-2"
                        >
                            <input
                                v-model="pm.hostPort"
                                type="text"
                                class="w-24 px-3 py-2 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Host"
                            />
                            <span class="text-text-muted">:</span>
                            <input
                                v-model="pm.containerPort"
                                type="text"
                                class="w-24 px-3 py-2 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Container"
                            />
                            <span class="text-text-muted">/</span>
                            <select
                                v-model="pm.protocol"
                                class="w-20 px-2 py-2 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                            >
                                <option value="tcp">tcp</option>
                                <option value="udp">udp</option>
                            </select>
                            <button
                                type="button"
                                @click="portMappings.splice(idx, 1)"
                                class="p-1.5 rounded-lg hover:bg-error/20 text-error transition-colors shrink-0"
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
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                            </button>
                        </div>
                        <p
                            v-if="portMappings.length === 0"
                            class="text-xs text-text-muted"
                        >
                            No port mappings configured.
                        </p>
                    </div>
                </div>

                <!-- Volume Mounts -->
                <div>
                    <div class="flex items-center justify-between mb-1">
                        <label
                            class="block text-sm font-medium text-text-secondary"
                        >
                            Volume Mounts
                        </label>
                        <button
                            type="button"
                            @click="addVolume"
                            class="text-xs text-reisa-lilac-400 hover:text-reisa-lilac-300 transition-colors"
                        >
                            + Add
                        </button>
                    </div>
                    <div class="space-y-2">
                        <div
                            v-for="(vol, idx) in volumeMounts"
                            :key="idx"
                            class="flex items-center gap-2"
                        >
                            <input
                                v-model="vol.hostPath"
                                type="text"
                                class="flex-1 px-4 py-3 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Host path"
                            />
                            <span class="text-text-muted">:</span>
                            <input
                                v-model="vol.containerPath"
                                type="text"
                                class="flex-1 px-4 py-3 text-sm font-mono bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                                placeholder="Container path"
                            />
                            <button
                                type="button"
                                @click="volumeMounts.splice(idx, 1)"
                                class="p-1.5 rounded-lg hover:bg-error/20 text-error transition-colors shrink-0"
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
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                            </button>
                        </div>
                        <p
                            v-if="volumeMounts.length === 0"
                            class="text-xs text-text-muted"
                        >
                            No volumes configured.
                        </p>
                    </div>
                </div>

                <!-- Restart Policy -->
                <div>
                    <label
                        class="block text-sm font-medium text-text-secondary mb-1"
                    >
                        Restart Policy
                    </label>
                    <select
                        v-model="form.restartPolicy"
                        class="w-full px-4 py-3 text-sm bg-surface border border-border rounded-lg text-text-primary transition-all duration-200 focus:outline-none focus:border-reisa-lilac-500 focus:shadow-[0_0_0_3px_oklch(0.66_0.058_301/0.2)]"
                    >
                        <option value="">None (no)</option>
                        <option value="always">Always</option>
                        <option value="unless-stopped">Unless Stopped</option>
                        <option value="on-failure">On Failure</option>
                    </select>
                </div>

                <!-- Error -->
                <div
                    v-if="error"
                    class="p-3 rounded-lg bg-error/20 border border-error/30 text-error text-sm"
                >
                    {{ error }}
                </div>

                <!-- Success -->
                <div
                    v-if="successMsg"
                    class="p-3 rounded-lg bg-success/20 border border-success/30 text-success text-sm"
                >
                    {{ successMsg }}
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-3 pt-2">
                    <button
                        type="submit"
                        class="inline-flex items-center justify-center gap-2 flex-1 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)] disabled:cursor-not-allowed disabled:opacity-50"
                        :disabled="creating || !form.image.trim()"
                    >
                        <template v-if="creating">
                            <svg
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
                            Creating...
                        </template>
                        <template v-else> Create Container </template>
                    </button>
                    <button
                        type="submit"
                        class="inline-flex items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-pink-500 to-reisa-pink-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-pink-400 hover:to-reisa-pink-500 hover:shadow-[0_4px_16px_oklch(0.65_0.21_356/0.4)] disabled:cursor-not-allowed disabled:opacity-50"
                        :disabled="creating || !form.image.trim()"
                        @click.prevent="submitAndStart"
                    >
                        Create &amp; Start
                    </button>
                </div>
            </form>
            n-primary flex-1" n-accent"
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { api } from '@/api'

const props = defineProps<{
    defaultImage?: string
}>()

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'created'): void
}>()

interface EnvPair {
    key: string
    value: string
}

interface PortMapping {
    hostPort: string
    containerPort: string
    protocol: string
}

interface VolumeMount {
    hostPath: string
    containerPath: string
}

const form = reactive({
    name: '',
    image: props.defaultImage || '',
    restartPolicy: '',
})

const cmdRaw = ref('')
const envPairs = ref<EnvPair[]>([])
const portMappings = ref<PortMapping[]>([])
const volumeMounts = ref<VolumeMount[]>([])
const creating = ref(false)
const error = ref('')
const successMsg = ref('')

const addEnv = () => {
    envPairs.value.push({ key: '', value: '' })
}

const addPort = () => {
    portMappings.value.push({
        hostPort: '',
        containerPort: '',
        protocol: 'tcp',
    })
}

const addVolume = () => {
    volumeMounts.value.push({ hostPath: '', containerPath: '' })
}

const buildPayload = () => {
    const payload: Record<string, any> = {
        image: form.image.trim(),
    }

    if (form.name.trim()) {
        payload.name = form.name.trim()
    }

    const cmdStr = cmdRaw.value.trim()
    if (cmdStr) {
        payload.cmd = cmdStr.split(/\s+/)
    }

    const envList = envPairs.value
        .filter((p) => p.key.trim())
        .map((p) => `${p.key.trim()}=${p.value}`)
    if (envList.length > 0) {
        payload.env = envList
    }

    const ports: Record<
        string,
        Array<{ host_ip: string; host_port: string }>
    > = {}
    for (const pm of portMappings.value) {
        if (!pm.containerPort.trim()) continue
        const key = `${pm.containerPort.trim()}/${pm.protocol}`
        ports[key] = [
            {
                host_ip: '0.0.0.0',
                host_port: pm.hostPort.trim() || pm.containerPort.trim(),
            },
        ]
    }
    if (Object.keys(ports).length > 0) {
        payload.ports = ports
    }

    const volumes: Record<string, string> = {}
    for (const vol of volumeMounts.value) {
        if (!vol.hostPath.trim() || !vol.containerPath.trim()) continue
        volumes[vol.hostPath.trim()] = vol.containerPath.trim()
    }
    if (Object.keys(volumes).length > 0) {
        payload.volumes = volumes
    }

    if (form.restartPolicy) {
        payload.restart_policy = form.restartPolicy
    }

    return payload
}

const submit = async () => {
    creating.value = true
    error.value = ''
    successMsg.value = ''

    try {
        const payload = buildPayload()
        const response = await api.post('/docker/containers', payload)
        successMsg.value = response.data.message || 'Container created!'
        emit('created')
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message || 'Failed to create container'
    } finally {
        creating.value = false
    }
}

const submitAndStart = async () => {
    creating.value = true
    error.value = ''
    successMsg.value = ''

    try {
        const payload = buildPayload()
        const createResp = await api.post('/docker/containers', payload)
        const msg = createResp.data.message || ''

        // Extract the container id from the response message
        // message format: "Container created with id <id>"
        const idMatch = msg.match(/id\s+([a-f0-9]+)/i)
        if (idMatch) {
            await api.post(`/docker/containers/${idMatch[1]}/start`)
            successMsg.value = 'Container created and started!'
        } else {
            successMsg.value = 'Container created! (could not auto-start)'
        }
        emit('created')
    } catch (e: any) {
        error.value =
            e.response?.data?.error?.message ||
            'Failed to create/start container'
    } finally {
        creating.value = false
    }
}
</script>
