import { api } from '@/api'

export interface ApplicationTemplateParam {
    key: string
    label: string
    description: string
    required: boolean
    default_value: string | null
    input: string // text | password | textarea | number | boolean | select
    placeholder: string | null
    env: string | null
    env_key?: string
    rule?: string
    options?: string[]
}

export interface ApplicationTemplatePort {
    key: string
    container_port: number
    protocol: string
    default_host_port: number | null
    required: boolean
}

export interface ApplicationTemplateEnv {
    key: string
    value: string | null
    from: string | null
    required: boolean
}

export interface ApplicationTemplateConfigFile {
    path: string
}

export interface ApplicationTemplateService {
    name: string
    image: string | null
}

export interface ApplicationTemplate {
    id: string
    name: string
    version: string
    category: string
    description: string
    icon_path: string | null
    readme_path: string | null
    compose_file: string
    services: ApplicationTemplateService[]
    params: ApplicationTemplateParam[]
    ports: ApplicationTemplatePort[]
    env: ApplicationTemplateEnv[]
    config_files: ApplicationTemplateConfigFile[]
    has_conf_templates: boolean
    app_dir: string
}

export interface ApplicationPort {
    ip: string
    private_port: number
    public_port: number | null
    protocol: string
}

export interface ApplicationInstanceService {
    name: string
    container_name: string
    state: string
    health: string | null
    ports: ApplicationPort[]
}

export interface ApplicationInstance {
    id: string
    name: string
    template_id: string
    category: string
    state: string
    ports: ApplicationPort[]
    services: ApplicationInstanceService[]
}

export type ApplicationInstallValue = string | number | boolean

export interface InstallApplicationRequest {
    template_id: string
    name?: string

    typed_values?: Record<string, ApplicationInstallValue>

    // Host bindings by app.toml [[port]].key => host port
    port_bindings?: Record<string, number>

    env?: string[]
}

export interface DockerActionResponse {
    success: boolean
    message: string
}

export type InstallApplicationResponse = ApplicationTask

export type ApplicationTaskStatus =
    | 'pending'
    | 'running'
    | 'succeeded'
    | 'failed'
export type ApplicationTaskKind = 'install'
export type ApplicationTaskLogLevel = 'info' | 'warn' | 'error'

export interface ApplicationTaskLogEntry {
    at: string
    level: ApplicationTaskLogLevel
    message: string
}

export interface ApplicationTask {
    id: string
    kind: ApplicationTaskKind
    status: ApplicationTaskStatus
    template_id: string
    requested_name?: string | null
    instance_id?: string | null
    created_at: string
    started_at?: string | null
    finished_at?: string | null
    summary?: string | null
    logs: ApplicationTaskLogEntry[]
}

const TASKS_STREAM_URL = '/api/applications/tasks/stream'

export const applicationsApi = {
    async listTemplates(): Promise<ApplicationTemplate[]> {
        const response = await api.get('/applications/templates')
        return response.data
    },

    async listInstances(): Promise<ApplicationInstance[]> {
        const response = await api.get('/applications/instances')
        return response.data
    },

    async install(
        payload: InstallApplicationRequest,
    ): Promise<InstallApplicationResponse> {
        const response = await api.post('/applications/install', payload)
        return response.data
    },

    async listTasks(): Promise<ApplicationTask[]> {
        const response = await api.get('/applications/tasks')
        return response.data
    },

    async getTask(taskId: string): Promise<ApplicationTask> {
        const response = await api.get(`/applications/tasks/${taskId}`)
        return response.data
    },

    getTasksStreamUrl(): string {
        return TASKS_STREAM_URL
    },

    getTaskStreamUrl(taskId: string): string {
        const safeTaskId = encodeURIComponent(taskId)
        return `/api/applications/tasks/${safeTaskId}/stream`
    },

    async start(instanceId: string): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/start`)
        return response.data
    },

    async stop(instanceId: string): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/stop`)
        return response.data
    },

    async update(instanceId: string): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/update`)
        return response.data
    },

    async remove(
        instanceId: string,
        force: boolean = true,
    ): Promise<DockerActionResponse> {
        const response = await api.delete(`/applications/${instanceId}`, {
            params: { force },
        })
        return response.data
    },

    async logs(
        instanceId: string,
        tail: number = 500,
    ): Promise<DockerActionResponse> {
        const response = await api.get(`/applications/${instanceId}/logs`, {
            params: { tail },
        })
        return response.data
    },

    async getEnv(instanceId: string): Promise<Record<string, string>> {
        const response = await api.get(`/applications/${instanceId}/env`)
        return response.data
    },

    async updateEnv(
        instanceId: string,
        envMap: Record<string, string>,
    ): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/env`, envMap)
        return response.data
    },
}
