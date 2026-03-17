<template>
    <div class="p-6 space-y-6 animate-in">
        <div class="flex flex-row-reverse items-center justify-between">
            <BaseButton @click="showCreate = true" variant="emphasis">
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
                        d="M12 4v16m8-8H4"
                    />
                </svg>
                New Website
            </BaseButton>
        </div>

        <!-- Website List -->
        <div
            v-if="websites.length > 0"
            class="grid grid-cols-1 lg:grid-cols-2 gap-4"
        >
            <div v-for="site in websites" :key="site.id">
                <div class="flex items-start justify-between mb-3">
                    <div class="flex-1 min-w-0">
                        <h3 class="font-semibold text-text-primary truncate">
                            {{ site.name }}
                        </h3>
                        <p class="text-sm text-reisa-lilac-400 truncate mt-1">
                            {{ site.primary_domain }}
                        </p>
                    </div>
                    <div class="flex flex-col items-end gap-2">
                        <div class="flex items-center gap-2">
                            <span
                                v-if="site.has_ssl"
                                class="badge badge-success flex items-center gap-1"
                                title="HTTPS enabled"
                            >
                                <svg
                                    class="w-3 h-3"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                                    />
                                </svg>
                                HTTPS
                            </span>
                            <span
                                :class="['badge', statusClass(site.status)]"
                                :title="site.error || undefined"
                                >{{ site.status }}</span
                            >
                        </div>
                    </div>
                </div>

                <div
                    v-if="site.status === 'error' && site.error"
                    class="mb-4 text-xs text-error bg-error/10 p-2 rounded border border-error/20"
                >
                    <p class="font-semibold mb-0.5">Deployment Failed:</p>
                    <p class="wrap-break-word font-mono">{{ site.error }}</p>
                </div>

                <div
                    class="flex flex-wrap items-center gap-2 text-xs text-text-muted mb-4"
                >
                    <span class="px-2 py-0.5 rounded bg-surface-overlay">{{
                        serverLabel(site.server_type)
                    }}</span>
                    <span class="px-2 py-0.5 rounded bg-surface-overlay">{{
                        serverInstanceLabel(site)
                    }}</span>
                    <span class="flex items-center gap-1">
                        <template
                            v-for="(type, index) in site.site_types"
                            :key="type"
                        >
                            <span>{{ siteTypeLabel(type) }}</span>
                            <span
                                v-if="index < site.site_types.length - 1"
                                class="text-text-muted/50"
                                >+</span
                            >
                        </template>
                    </span>
                    <template v-if="site.site_types.includes('reverse_proxy')">
                        <span>→</span>
                        <span class="text-reisa-pink-400 truncate max-w-48">{{
                            proxyTargetLabel(site)
                        }}</span>
                    </template>
                    <template v-if="site.site_types.includes('static')">
                        <span>→</span>
                        <span class="text-reisa-gold-400 truncate max-w-48">{{
                            site.root_dir
                        }}</span>
                    </template>
                </div>

                <div
                    v-if="site.aliases.length > 0"
                    class="flex flex-wrap gap-1 mb-4"
                >
                    <span
                        v-for="alias in site.aliases"
                        :key="alias"
                        class="text-xs px-2 py-0.5 rounded bg-surface text-text-muted border border-border-subtle"
                    >
                        {{ alias }}
                    </span>
                </div>

                <div class="flex items-center gap-2">
                    <BaseButton
                        @click="editWebsite(site)"
                        variant="outline"
                        class="flex-1 text-reisa-lilac-400 hover:bg-reisa-lilac-500/20 hover:text-reisa-lilac-400"
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
                                d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                            />
                        </svg>
                        Edit
                    </BaseButton>
                    <BaseButton
                        @click="deleteWebsite(site)"
                        variant="outline"
                        class="text-error hover:bg-error/20 hover:text-error"
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
                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                            />
                        </svg>
                    </BaseButton>
                </div>
            </div>
        </div>

        <!-- Empty State -->
        <div v-else-if="!loading" class="text-center py-12">
            <svg
                class="w-16 h-16 mx-auto text-text-muted mb-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
                />
            </svg>
            <h3 class="text-lg font-semibold text-text-primary mb-2">
                No Websites Yet
            </h3>
            <p class="text-text-muted mb-4">
                Create your first website to get started with reverse proxy and
                HTTPS.
            </p>
            <BaseButton @click="showCreate = true" variant="emphasis">
                Create Website
            </BaseButton>
        </div>

        <!-- Loading Skeletons -->
        <div
            v-if="loading && websites.length === 0"
            class="grid grid-cols-1 lg:grid-cols-2 gap-4"
        >
            <div v-for="i in 4" :key="i">
                <div class="shimmer h-6 w-40 rounded mb-2"></div>
                <div class="shimmer h-4 w-56 rounded mb-4"></div>
                <div class="shimmer h-8 w-full rounded"></div>
            </div>
        </div>

        <!-- Create / Edit Modal -->
        <div
            v-if="showCreate || editingSite"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
            @click.self="closeModal"
        >
            <div
                class="w-full max-w-lg bg-surface-elevated rounded-xl shadow-2xl animate-in"
            >
                <div
                    class="flex items-center justify-between p-5 border-b border-border-subtle"
                >
                    <h3 class="font-semibold text-text-primary text-lg">
                        {{ editingSite ? 'Edit Website' : 'Create Website' }}
                    </h3>
                    <BaseButton
                        @click="closeModal"
                        class="p-1.5 rounded-lg hover:bg-surface-overlay text-text-muted hover:text-text-primary transition-colors"
                    >
                        <svg
                            class="w-5 h-5"
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
                    </BaseButton>
                </div>

                <div class="p-5 space-y-4 max-h-[70vh] overflow-y-auto">
                    <!-- Name -->
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-1"
                            >Name</label
                        >
                        <input
                            v-model="form.name"
                            class="input"
                            placeholder="My Website"
                        />
                    </div>

                    <!-- Primary Domain -->
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-1"
                            >Primary Domain</label
                        >
                        <input
                            v-model="form.primary_domain"
                            class="input"
                            placeholder="example.com"
                        />
                    </div>

                    <!-- Aliases -->
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-1"
                            >Aliases (comma separated)</label
                        >
                        <input
                            v-model="aliasesInput"
                            class="input"
                            placeholder="www.example.com, blog.example.com"
                        />
                    </div>

                    <!-- Server Instance -->
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-1"
                            >Server Instance</label
                        >
                        <select v-model="form.server_instance_id" class="input">
                            <option value="">
                                Select a proxy server instance...
                            </option>
                            <option
                                v-for="instance in proxyServerInstances"
                                :key="instance.id"
                                :value="instance.id"
                            >
                                {{ instance.name }} ({{
                                    serverLabel(
                                        serverTypeFromTemplate(
                                            instance.template_id,
                                        ) || 'unknown',
                                    )
                                }})
                            </option>
                        </select>
                        <p
                            v-if="proxyServerInstances.length === 0"
                            class="text-xs text-warning mt-1"
                        >
                            No proxy server instance found. Install
                            Caddy/Nginx/OpenResty app first.
                        </p>
                    </div>

                    <!-- Site Types (Checkboxes) -->
                    <div>
                        <label
                            class="block text-sm font-medium text-text-secondary mb-2"
                            >Site Modes</label
                        >
                        <div class="space-y-2">
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    value="reverse_proxy"
                                    v-model="form.site_types"
                                    class="checkbox checkbox-primary"
                                    :disabled="!!editingSite"
                                />
                                <span class="text-sm text-text-primary"
                                    >Reverse Proxy</span
                                >
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    value="static"
                                    v-model="form.site_types"
                                    class="checkbox checkbox-primary"
                                    :disabled="!!editingSite"
                                />
                                <span class="text-sm text-text-primary"
                                    >Static Site</span
                                >
                            </label>
                        </div>
                    </div>

                    <!-- SSL / HTTPS -->
                    <div>
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input
                                type="checkbox"
                                v-model="form.has_ssl"
                                class="checkbox checkbox-primary"
                                :disabled="isCaddySelected"
                            />
                            <span class="text-sm font-medium text-text-primary"
                                >Enable HTTPS</span
                            >
                        </label>
                        <p
                            v-if="isCaddySelected"
                            class="text-xs text-reisa-lilac-400 mt-1.5 ml-6"
                        >
                            Caddy automatically provisions HTTPS via Let's
                            Encrypt. SSL is always enabled.
                        </p>
                        <p
                            v-else-if="form.has_ssl"
                            class="text-xs text-reisa-lilac-400 mt-1.5 ml-6"
                        >
                            A Let's Encrypt certificate will be automatically
                            issued. Ensure your domain's DNS points to this
                            server.
                        </p>
                    </div>

                    <!-- Reverse Proxy Config -->
                    <template v-if="form.site_types.includes('reverse_proxy')">
                        <!-- Target Type -->
                        <div>
                            <label
                                class="block text-sm font-medium text-text-secondary mb-1"
                                >Proxy Target</label
                            >
                            <select
                                v-model="form.proxy_target_type"
                                class="input"
                            >
                                <option value="url">Custom URL</option>
                                <option value="application">
                                    Application Instance
                                </option>
                            </select>
                        </div>

                        <!-- Custom URL -->
                        <div v-if="form.proxy_target_type === 'url'">
                            <label
                                class="block text-sm font-medium text-text-secondary mb-1"
                                >Target URL</label
                            >
                            <input
                                v-model="form.proxy_target_url"
                                class="input"
                                placeholder="http://127.0.0.1:8080"
                            />
                        </div>

                        <!-- Application Instance -->
                        <template
                            v-if="form.proxy_target_type === 'application'"
                        >
                            <div>
                                <label
                                    class="block text-sm font-medium text-text-secondary mb-1"
                                    >Application</label
                                >
                                <select
                                    v-model="form.proxy_target_app_id"
                                    class="input"
                                >
                                    <option value="">
                                        Select an application...
                                    </option>
                                    <option
                                        v-for="app in appTargetInstances"
                                        :key="app.id"
                                        :value="app.id"
                                    >
                                        {{ app.name }} ({{ app.state }})
                                    </option>
                                </select>
                            </div>
                            <div>
                                <label
                                    class="block text-sm font-medium text-text-secondary mb-1"
                                    >Application Port</label
                                >
                                <select
                                    v-model.number="form.proxy_target_app_port"
                                    class="input"
                                >
                                    <option :value="0">Select a port...</option>
                                    <option
                                        v-for="port in selectedAppPorts"
                                        :key="`${port.private_port}/${port.protocol}`"
                                        :value="port.private_port"
                                    >
                                        {{ port.private_port }}/{{
                                            port.protocol
                                        }}{{
                                            port.public_port
                                                ? ` → :${port.public_port}`
                                                : ''
                                        }}
                                    </option>
                                </select>
                            </div>
                        </template>
                    </template>

                    <!-- Static Site Config -->
                    <div v-if="form.site_types.includes('static')">
                        <label
                            class="block text-sm font-medium text-text-secondary mb-1"
                            >Root Directory</label
                        >
                        <input
                            v-model="form.root_dir"
                            class="input"
                            placeholder="/opt/mana-panel/www/my-site"
                        />
                    </div>
                </div>

                <div
                    class="flex flex-row-reverse gap-3 p-5 border-t border-border-subtle"
                >
                    <BaseButton
                        @click="submitForm"
                        variant="emphasis"
                        :disabled="submitting"
                        :text="
                            submitting
                                ? 'Saving...'
                                : editingSite
                                  ? 'Update'
                                  : 'Create'
                        "
                    ></BaseButton>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
    websitesApi,
    type WebsiteInfo,
    type CreateWebsiteRequest,
    type UpdateWebsiteRequest,
    type ServerType,
    type SiteType,
    type ProxyTargetType,
} from '@/api/websites'
import BaseButton from '@/components/universal/BaseButton.vue'
import {
    applicationsApi,
    type ApplicationInstance,
    type ApplicationPort,
} from '@/api/applications'

const websites = ref<WebsiteInfo[]>([])
const appInstances = ref<ApplicationInstance[]>([])
const loading = ref(false)
const submitting = ref(false)
const showCreate = ref(false)
const editingSite = ref<WebsiteInfo | null>(null)
const aliasesInput = ref('')

const form = ref<{
    name: string
    primary_domain: string
    server_instance_id: string
    site_types: SiteType[]
    proxy_target_type: ProxyTargetType
    proxy_target_url: string
    proxy_target_app_id: string
    proxy_target_app_port: number
    root_dir: string
    has_ssl: boolean
}>({
    name: '',
    primary_domain: '',
    server_instance_id: '',
    site_types: ['reverse_proxy'],
    proxy_target_type: 'url',
    proxy_target_url: '',
    proxy_target_app_id: '',
    proxy_target_app_port: 0,
    root_dir: '',
    has_ssl: true,
})

const selectedServerType = computed<ServerType | null>(() => {
    if (!form.value.server_instance_id) return null
    const instance = proxyServerInstances.value.find(
        (i) => i.id === form.value.server_instance_id,
    )
    return instance ? serverTypeFromTemplate(instance.template_id) : null
})

const isCaddySelected = computed(() => selectedServerType.value === 'caddy')

const PROXY_TEMPLATE_TO_SERVER_TYPE: Record<string, ServerType> = {
    'caddy-autossl': 'caddy',
    'nginx-standard': 'nginx',
    'openresty-waf': 'openresty',
}

const serverTypeFromTemplate = (templateId: string): ServerType | null => {
    return PROXY_TEMPLATE_TO_SERVER_TYPE[templateId] || null
}

const proxyServerInstances = computed<ApplicationInstance[]>(() => {
    return appInstances.value.filter((instance) =>
        Boolean(serverTypeFromTemplate(instance.template_id)),
    )
})

const appTargetInstances = computed<ApplicationInstance[]>(() => {
    return appInstances.value.filter(
        (instance) => !serverTypeFromTemplate(instance.template_id),
    )
})

const selectedAppPorts = computed<ApplicationPort[]>(() => {
    if (!form.value.proxy_target_app_id) return []
    const app = appTargetInstances.value.find(
        (a) => a.id === form.value.proxy_target_app_id,
    )
    return app?.ports ?? []
})

// Force SSL on for Caddy
watch(isCaddySelected, (isCaddy) => {
    if (isCaddy) {
        form.value.has_ssl = true
    }
})

// Auto-populate root_dir for static sites
watch(
    [() => form.value.name, () => form.value.site_types],
    ([newName, newTypes]) => {
        if (newTypes.includes('static') && !editingSite.value) {
            const sanitized = newName
                .toLowerCase()
                .trim()
                .replace(/[^a-z0-9]/g, '-')
                .replace(/-+/g, '-')
                .replace(/^-|-$/g, '')

            // Only update if current root_dir is empty or was previously auto-generated (starts with the prefix)
            const currentDir = form.value.root_dir
            if (!currentDir || currentDir.startsWith('/opt/mana-panel/www/')) {
                form.value.root_dir = sanitized
                    ? `/opt/mana-panel/www/${sanitized}`
                    : ''
            }
        }
    },
)

const fetchWebsites = async () => {
    loading.value = true
    try {
        websites.value = await websitesApi.list()
    } catch (e) {
        console.error('Failed to fetch websites:', e)
    } finally {
        loading.value = false
    }
}

const fetchAppInstances = async () => {
    try {
        appInstances.value = await applicationsApi.listInstances()
    } catch (e) {
        console.error('Failed to fetch app instances:', e)
    }
}

const resetForm = () => {
    form.value = {
        name: '',
        primary_domain: '',
        server_instance_id: '',
        site_types: ['reverse_proxy'],
        proxy_target_type: 'url',
        proxy_target_url: '',
        proxy_target_app_id: '',
        proxy_target_app_port: 0,
        root_dir: '',
        has_ssl: true,
    }
    aliasesInput.value = ''
}

const editWebsite = (site: WebsiteInfo) => {
    editingSite.value = site
    form.value = {
        name: site.name,
        primary_domain: site.primary_domain,
        server_instance_id: site.server_instance_id || '',
        site_types: [...site.site_types],
        proxy_target_type: site.proxy_target_type || 'url',
        proxy_target_url: site.proxy_target_url || '',
        proxy_target_app_id: site.proxy_target_app_id || '',
        proxy_target_app_port: site.proxy_target_app_port || 0,
        root_dir: site.root_dir || '',
        has_ssl: site.has_ssl,
    }
    aliasesInput.value = site.aliases.join(', ')
    fetchAppInstances()
}

const closeModal = () => {
    showCreate.value = false
    editingSite.value = null
    resetForm()
}

const submitForm = async () => {
    submitting.value = true
    const aliases = aliasesInput.value
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)

    try {
        if (!form.value.server_instance_id) {
            alert('Please select a server instance')
            return
        }

        if (editingSite.value) {
            const payload: UpdateWebsiteRequest = {
                name: form.value.name,
                primary_domain: form.value.primary_domain,
                aliases,
                server_instance_id: form.value.server_instance_id,
                has_ssl: form.value.has_ssl,
            }

            if (form.value.site_types.includes('reverse_proxy')) {
                payload.proxy_target_type = form.value.proxy_target_type
                if (form.value.proxy_target_type === 'url') {
                    payload.proxy_target_url =
                        form.value.proxy_target_url || null
                } else {
                    payload.proxy_target_app_id =
                        form.value.proxy_target_app_id || null
                    payload.proxy_target_app_port =
                        form.value.proxy_target_app_port || null
                }
            }
            if (form.value.site_types.includes('static')) {
                payload.root_dir = form.value.root_dir || null
            }

            await websitesApi.update(editingSite.value.id, payload)
        } else {
            const payload: CreateWebsiteRequest = {
                name: form.value.name,
                primary_domain: form.value.primary_domain,
                aliases,
                server_instance_id: form.value.server_instance_id,
                site_types: form.value.site_types,
                has_ssl: form.value.has_ssl,
            }

            if (form.value.site_types.includes('reverse_proxy')) {
                payload.proxy_target_type = form.value.proxy_target_type
                if (form.value.proxy_target_type === 'url') {
                    payload.proxy_target_url = form.value.proxy_target_url
                } else {
                    payload.proxy_target_app_id = form.value.proxy_target_app_id
                    payload.proxy_target_app_port =
                        form.value.proxy_target_app_port
                }
            }
            if (form.value.site_types.includes('static')) {
                payload.root_dir = form.value.root_dir
            }

            await websitesApi.create(payload)
        }
        closeModal()
        await fetchWebsites()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Operation failed')
    } finally {
        submitting.value = false
    }
}

const deleteWebsite = async (site: WebsiteInfo) => {
    if (!confirm(`Are you sure you want to delete "${site.name}"?`)) return
    try {
        await websitesApi.remove(site.id)
        await fetchWebsites()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to delete website')
    }
}

const statusClass = (status: string) => {
    if (status === 'running') return 'badge-success'
    if (status === 'stopped') return 'badge-warning'
    if (status === 'error') return 'badge-error'
    return 'badge-info'
}

const serverLabel = (type: string) => {
    const labels: Record<string, string> = {
        caddy: 'Caddy',
        nginx: 'Nginx',
        openresty: 'OpenResty',
    }
    return labels[type] || type
}

const serverInstanceLabel = (site: WebsiteInfo) => {
    if (!site.server_instance_id) return 'Unbound instance'
    const instance = appInstances.value.find(
        (a) => a.id === site.server_instance_id,
    )
    return instance?.name || site.server_instance_id
}

const siteTypeLabel = (type: string) => {
    return type === 'reverse_proxy' ? 'Reverse Proxy' : 'Static'
}

const proxyTargetLabel = (site: WebsiteInfo) => {
    if (site.proxy_target_type === 'application') {
        return `App: ${site.proxy_target_app_id}:${site.proxy_target_app_port}`
    }
    return site.proxy_target_url || '—'
}

onMounted(() => {
    fetchWebsites()
    fetchAppInstances()
})
</script>
