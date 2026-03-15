<template>
    <Transition name="fade">
        <div
            v-if="open"
            class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6"
            style="backdrop-filter: blur(4px)"
        >
            <div
                class="absolute inset-0 bg-black/60 transition-opacity"
                @click="$emit('close')"
            ></div>

            <div
                class="relative w-full max-w-4xl max-h-[90vh] flex flex-col rounded-2xl border border-border bg-surface shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200"
            >
                <div
                    class="flex items-center justify-between p-5 border-b border-border bg-surface-secondary/30"
                >
                    <div>
                        <h2 class="text-xl font-bold text-text-primary">
                            Deploy Template
                        </h2>
                        <p class="text-sm text-text-muted">
                            {{ template?.name }}
                            <span class="opacity-50"
                                >v{{ template?.version }}</span
                            >
                        </p>
                    </div>
                    <button
                        @click="$emit('close')"
                        class="btn btn-ghost btn-sm btn-circle text-text-muted hover:text-text-primary"
                        :disabled="submitting"
                    >
                        <svg
                            class="w-5 h-5"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
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

                <div class="flex-1 overflow-y-auto p-6 scrollbar-thin">
                    <div
                        v-if="error"
                        class="mb-6 p-4 rounded-xl bg-error/10 border border-error/20 text-error text-sm flex gap-3"
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
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            />
                        </svg>
                        {{ error }}
                    </div>

                    <form
                        id="installForm"
                        @submit.prevent="$emit('submit')"
                        class="space-y-8"
                    >
                        <div class="space-y-4">
                            <h3
                                class="text-sm font-semibold uppercase tracking-wider text-text-secondary border-b border-border pb-2"
                            >
                                Configuration
                            </h3>
                            <div class="grid grid-cols-1 gap-5">
                                <div class="form-control space-y-1.5">
                                    <label
                                        class="text-sm font-medium text-text-primary"
                                        >Instance Name</label
                                    >
                                    <BaseInput
                                        v-model="form.name"
                                        type="text"
                                        :placeholder="form.name"
                                    />
                                </div>
                            </div>
                        </div>

                        <div v-if="portFields.length" class="space-y-4">
                            <h3
                                class="text-sm font-semibold uppercase tracking-wider text-text-secondary border-b border-border pb-2"
                            >
                                Ports
                            </h3>
                            <div
                                class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
                            >
                                <div
                                    v-for="port in portFields"
                                    :key="port.key"
                                    class="bg-surface-secondary/30 p-3 rounded-lg border border-border/50"
                                >
                                    <label
                                        class="block text-xs font-medium text-text-secondary mb-1.5"
                                    >
                                        {{ port.key }}
                                        <span class="opacity-50"
                                            >({{ port.endpoint }})</span
                                        >
                                        <span
                                            v-if="port.required"
                                            class="text-error ml-0.5"
                                            >*</span
                                        >
                                    </label>
                                    <div class="flex items-center gap-2">
                                        <span class="text-xs text-text-muted"
                                            >Host:</span
                                        >
                                        <BaseInput
                                            v-model.number="
                                                form.port_bindings[port.key]
                                            "
                                            type="number"
                                            min="1"
                                            max="65535"
                                            :placeholder="
                                                port.default_host_port
                                                    ? String(
                                                          port.default_host_port,
                                                      )
                                                    : 'Random'
                                            "
                                            class="h-8 px-2 font-mono"
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="space-y-4">
                            <h3
                                class="text-sm font-semibold uppercase tracking-wider text-text-secondary border-b border-border pb-2"
                            >
                                Custom Port Mappings
                            </h3>

                            <div class="space-y-3">
                                <div
                                    v-if="
                                        Object.keys(
                                            form.extra_port_bindings || {},
                                        ).length === 0
                                    "
                                    class="text-xs text-text-muted"
                                >
                                    No custom port mappings. Add endpoint-based
                                    mappings like
                                    <span class="font-mono"
                                        >8081/tcp → 18081</span
                                    >.
                                </div>

                                <div
                                    v-for="endpoint in Object.keys(
                                        form.extra_port_bindings || {},
                                    )"
                                    :key="`custom-port-${endpoint}`"
                                    class="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr_auto] gap-2 items-center"
                                >
                                    <BaseInput
                                        :value="endpoint"
                                        @input="
                                            renameCustomPortEndpoint(
                                                endpoint,
                                                $event,
                                            )
                                        "
                                        placeholder="8081/tcp"
                                        class="font-mono"
                                    />
                                    <span
                                        class="text-xs text-text-muted text-center"
                                        >→</span
                                    >
                                    <BaseInput
                                        v-model.number="
                                            form.extra_port_bindings[endpoint]
                                        "
                                        type="number"
                                        min="1"
                                        max="65535"
                                        placeholder="18081"
                                        class="font-mono"
                                    />
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-sm text-error hover:bg-error/10"
                                        @click="
                                            removeCustomPortEndpoint(endpoint)
                                        "
                                    >
                                        Remove
                                    </button>
                                </div>

                                <button
                                    type="button"
                                    class="btn btn-sm btn-ghost border border-border hover:border-reisa-lilac-500/40"
                                    @click="addCustomPortBinding"
                                >
                                    + Add Custom Port Mapping
                                </button>
                            </div>
                        </div>

                        <div class="space-y-4">
                            <h3
                                class="text-sm font-semibold uppercase tracking-wider text-text-secondary border-b border-border pb-2"
                            >
                                Environment Overrides
                            </h3>

                            <div class="space-y-3">
                                <div
                                    v-if="
                                        Object.keys(form.env_overrides || {})
                                            .length === 0
                                    "
                                    class="text-xs text-text-muted"
                                >
                                    No environment overrides. Add values like
                                    <span class="font-mono"
                                        >TZ=Asia/Shanghai</span
                                    >.
                                </div>

                                <div
                                    v-for="envKey in Object.keys(
                                        form.env_overrides || {},
                                    )"
                                    :key="`env-override-${envKey}`"
                                    class="grid grid-cols-1 md:grid-cols-[1fr_1fr_auto] gap-2 items-center"
                                >
                                    <BaseInput
                                        :value="envKey"
                                        @input="
                                            renameEnvOverrideKey(envKey, $event)
                                        "
                                        placeholder="ENV_NAME"
                                        class="font-mono"
                                    />
                                    <BaseInput
                                        v-model="form.env_overrides[envKey]"
                                        placeholder="value"
                                        class="font-mono"
                                    />
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-sm text-error hover:bg-error/10"
                                        @click="removeEnvOverride(envKey)"
                                    >
                                        Remove
                                    </button>
                                </div>

                                <button
                                    type="button"
                                    class="btn btn-sm btn-ghost border border-border hover:border-reisa-lilac-500/40"
                                    @click="addEnvOverride"
                                >
                                    + Add Environment Override
                                </button>
                            </div>
                        </div>

                        <div v-if="paramFields.length" class="space-y-4">
                            <h3
                                class="text-sm font-semibold uppercase tracking-wider text-text-secondary border-b border-border pb-2"
                            >
                                Template Parameters
                            </h3>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                                <div
                                    v-for="field in paramFields"
                                    :key="field.key"
                                    :class="
                                        field.input === 'textarea'
                                            ? 'md:col-span-2'
                                            : ''
                                    "
                                >
                                    <div
                                        v-if="field.input === 'boolean'"
                                        class="flex items-center justify-between p-3 rounded-lg border border-border bg-surface-secondary/20"
                                    >
                                        <div class="flex flex-col">
                                            <span
                                                class="text-sm font-medium text-text-primary"
                                                >{{ field.label }}</span
                                            >
                                            <span
                                                class="text-xs text-text-muted mt-0.5"
                                                >{{ field.description }}</span
                                            >
                                        </div>
                                        <input
                                            type="checkbox"
                                            :checked="
                                                readBooleanValue(field.key)
                                            "
                                            @change="
                                                setBooleanValue(
                                                    field.key,
                                                    $event,
                                                )
                                            "
                                            class="toggle toggle-sm accent-reisa-lilac-500"
                                        />
                                    </div>

                                    <div
                                        v-else-if="field.input === 'textarea'"
                                        class="space-y-1.5"
                                    >
                                        <div class="flex justify-between">
                                            <label
                                                class="text-sm font-medium text-text-primary"
                                            >
                                                {{ field.label }}
                                                <span
                                                    v-if="field.required"
                                                    class="text-error"
                                                    >*</span
                                                >
                                            </label>
                                        </div>
                                        <textarea
                                            v-model="form.values[field.key]"
                                            :placeholder="
                                                field.placeholder ?? undefined
                                            "
                                            class="textarea w-full min-h-25 bg-background border-border focus:border-reisa-lilac-500 rounded-lg text-sm font-mono leading-relaxed"
                                        ></textarea>
                                        <p class="text-xs text-text-muted">
                                            {{ field.description }}
                                        </p>
                                    </div>

                                    <div
                                        v-else-if="
                                            field.input === 'select' &&
                                            field.options?.length
                                        "
                                        class="space-y-1.5"
                                    >
                                        <label
                                            class="text-sm font-medium text-text-primary"
                                        >
                                            {{ field.label }}
                                            <span
                                                v-if="field.required"
                                                class="text-error"
                                                >*</span
                                            >
                                        </label>
                                        <select
                                            v-model="form.values[field.key]"
                                            class="select w-full bg-background border-border focus:border-reisa-lilac-500 rounded-lg"
                                        >
                                            <option
                                                v-for="opt in field.options"
                                                :key="opt"
                                                :value="opt"
                                            >
                                                {{ opt }}
                                            </option>
                                        </select>
                                        <p
                                            class="text-xs text-text-muted truncate"
                                            :title="field.description"
                                        >
                                            {{ field.description }}
                                        </p>
                                    </div>

                                    <div v-else class="space-y-1.5">
                                        <label
                                            class="text-sm font-medium text-text-primary"
                                        >
                                            {{ field.label }}
                                            <span
                                                v-if="field.required"
                                                class="text-error"
                                                >*</span
                                            >
                                        </label>
                                        <BaseInput
                                            v-model="form.values[field.key]"
                                            :type="
                                                resolveInputType(field.input)
                                            "
                                            :placeholder="
                                                field.placeholder ?? undefined
                                            "
                                        />
                                        <p
                                            class="text-xs text-text-muted truncate"
                                            :title="field.description"
                                        >
                                            {{ field.description }}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div
                            v-if="validationErrors.length"
                            class="p-4 rounded-xl bg-warning/10 border border-warning/20"
                        >
                            <div
                                class="flex items-center gap-2 text-warning font-medium mb-2 text-sm"
                            >
                                <svg
                                    class="w-4 h-4"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                    />
                                </svg>
                                配置检查
                            </div>
                            <ul
                                class="list-disc list-inside text-xs text-text-secondary space-y-1 pl-1"
                            >
                                <li
                                    v-for="item in validationErrors"
                                    :key="item"
                                >
                                    {{ item }}
                                </li>
                            </ul>
                        </div>
                    </form>
                </div>

                <div
                    class="p-5 border-t border-border bg-surface-secondary/30 flex items-center justify-between gap-4"
                >
                    <div
                        class="text-sm text-text-muted font-medium animate-pulse"
                        v-if="submitting"
                    >
                        {{ statusMessage || 'Processing...' }}
                    </div>
                    <div class="flex items-center gap-3 ml-auto">
                        <button
                            type="button"
                            class="btn btn-ghost hover:bg-surface-secondary text-text-secondary"
                            @click="$emit('close')"
                            :disabled="submitting"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            form="installForm"
                            class="btn bg-reisa-lilac-500 hover:bg-reisa-lilac-600 text-white border-none shadow-lg shadow-reisa-lilac-500/20 min-w-30"
                            :disabled="!canSubmit || submitting"
                        >
                            <span
                                v-if="submitting"
                                class="loading loading-spinner loading-sm mr-2"
                            ></span>
                            {{ submitting ? 'Deploying...' : 'Deploy' }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </Transition>
</template>

<script setup lang="ts">
import type {
    ApplicationTemplate,
    ApplicationTemplateParam,
} from '@/api/applications'
import BaseInput from '@/components/universal/BaseInput.vue'

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

const props = defineProps<{
    open: boolean
    template: ApplicationTemplate | null
    form: InstallFormState
    paramFields: ApplicationTemplateParam[]
    portFields: InstallPortField[]
    submitting: boolean
    error: string
    validationErrors: string[]
    statusMessage: string
    canSubmit: boolean
}>()

defineEmits<{
    (e: 'close'): void
    (e: 'submit'): void
}>()

const ensureCustomPortBindings = () => {
    if (!props.form.extra_port_bindings) {
        props.form.extra_port_bindings = {}
    }
}

const ensureEnvOverrides = () => {
    if (!props.form.env_overrides) {
        props.form.env_overrides = {}
    }
}

const addCustomPortBinding = () => {
    ensureCustomPortBindings()

    let i = 0
    let candidate = '8081/tcp'
    while (props.form.extra_port_bindings[candidate] !== undefined) {
        i += 1
        candidate = `${8081 + i}/tcp`
    }

    props.form.extra_port_bindings[candidate] = undefined
}

const removeCustomPortEndpoint = (endpoint: string) => {
    ensureCustomPortBindings()
    delete props.form.extra_port_bindings[endpoint]
}

const renameCustomPortEndpoint = (oldKey: string, event: Event) => {
    ensureCustomPortBindings()

    const target = event.target as HTMLInputElement
    const nextKey = target.value.trim().toLowerCase()
    if (nextKey === oldKey) return

    const currentValue = props.form.extra_port_bindings[oldKey]
    delete props.form.extra_port_bindings[oldKey]

    if (nextKey) {
        props.form.extra_port_bindings[nextKey] = currentValue
    }
}

const addEnvOverride = () => {
    ensureEnvOverrides()

    let i = 0
    let candidate = 'NEW_ENV'
    while (props.form.env_overrides[candidate] !== undefined) {
        i += 1
        candidate = `NEW_ENV_${i}`
    }

    props.form.env_overrides[candidate] = ''
}

const removeEnvOverride = (key: string) => {
    ensureEnvOverrides()
    delete props.form.env_overrides[key]
}

const renameEnvOverrideKey = (oldKey: string, event: Event) => {
    ensureEnvOverrides()

    const target = event.target as HTMLInputElement
    const nextKey = target.value.trim()
    if (nextKey === oldKey) return

    const currentValue = props.form.env_overrides[oldKey]
    delete props.form.env_overrides[oldKey]

    if (nextKey) {
        props.form.env_overrides[nextKey] = currentValue ?? ''
    }
}

const resolveInputType = (input: string): string => {
    if (input === 'password') return 'password'
    if (input === 'number') return 'number'
    return 'text'
}

const readBooleanValue = (key: string): boolean => {
    const raw = props.form.values[key] ?? ''
    return String(raw).trim().toLowerCase() === 'true'
}

const setBooleanValue = (key: string, event: Event) => {
    const target = event.target as HTMLInputElement
    props.form.values[key] = target.checked ? 'true' : 'false'
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
