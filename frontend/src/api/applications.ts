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
    template: string
    target: string
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

export interface ApplicationInstance {
    id: string
    name: string
    template_id: string
    category: string
    state: string
    ports: ApplicationPort[]
}

export interface InstallApplicationRequest {
    template_id: string
    name?: string

    // Generic app values (app.toml params)
    values?: Record<string, string>

    // Host bindings by app.toml [[port]].key => host port
    port_bindings?: Record<string, number>

    env?: string[]
}

export interface DockerActionResponse {
    success: boolean
    message: string
}

export interface InstallApplicationResponse {
    action: DockerActionResponse
}

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

    async start(instanceId: string): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/start`)
        return response.data
    },

    async stop(instanceId: string): Promise<DockerActionResponse> {
        const response = await api.post(`/applications/${instanceId}/stop`)
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
}
