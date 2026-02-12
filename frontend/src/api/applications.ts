import { api } from '@/api'

export interface ApplicationTemplate {
    id: string
    name: string
    description: string
    category: string
    image: string
    default_ports: number[]
    requires_domain: boolean
    requires_upstream: boolean
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
    image: string
    state: string
    status: string
    ports: ApplicationPort[]
    labels: Record<string, string>
}

export interface InstallApplicationRequest {
    template_id: string
    name?: string
    image_override?: string
    upstream_host?: string
    upstream_port?: number
    domain?: string
    listen_http_port?: number
    listen_https_port?: number
    env?: string[]
    replace_existing?: boolean
}

export interface PortConflict {
    port: number
    occupied_by: string
    occupant_type: 'container' | 'host-process' | string
    occupant_container_id: string | null
    occupant_image: string | null
    suggestion: string
}

export interface PreflightResult {
    ok: boolean
    requested_ports: number[]
    conflicts: PortConflict[]
    warnings: string[]
}

export interface DockerActionResponse {
    success: boolean
    message: string
}

export interface InstallPullProgress {
    status: string
    id?: string
    progress?: string
}

export interface InstallApplicationResponse {
    action: DockerActionResponse
    image_pulled: boolean
    pull_progress: InstallPullProgress[]
}

interface ImageExistsResponse {
    image: string
    exists: boolean
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

    async imageExists(image: string): Promise<boolean> {
        const target = image.trim()
        if (!target) return false

        const response = await api.get('/applications/images/exists', {
            params: { image: target },
        })
        const data = response.data as ImageExistsResponse
        return !!data.exists
    },

    async install(
        payload: InstallApplicationRequest,
    ): Promise<InstallApplicationResponse> {
        const response = await api.post('/applications/install', payload)
        return response.data
    },

    async preflightStart(instanceId: string): Promise<PreflightResult> {
        const response = await api.post(`/applications/${instanceId}/preflight`)
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
