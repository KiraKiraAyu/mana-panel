import { api } from '@/api'

export type ServerType = 'nginx' | 'caddy' | 'openresty'
export type SiteType = 'reverse_proxy' | 'static'
export type ProxyTargetType = 'url' | 'application'
export type WebsiteStatus = 'running' | 'stopped' | 'error'

export interface WebsiteInfo {
    id: number
    name: string
    primary_domain: string
    aliases: string[]
    server_type: ServerType
    server_instance_id: string | null
    site_types: SiteType[]
    proxy_target_type: ProxyTargetType | null
    proxy_target_url: string | null
    proxy_target_app_id: string | null
    proxy_target_app_port: number | null
    root_dir?: string | null
    status: string
    error?: string | null
    has_ssl: boolean
    created_at: string
    updated_at: string
}

export interface CreateWebsiteRequest {
    name: string
    primary_domain: string
    aliases?: string[]
    server_instance_id: string
    site_types: SiteType[]
    proxy_target_type?: ProxyTargetType | null
    proxy_target_url?: string | null
    proxy_target_app_id?: string | null
    proxy_target_app_port?: number | null
    root_dir?: string | null
}

export interface UpdateWebsiteRequest {
    name?: string
    primary_domain?: string
    aliases?: string[]
    server_instance_id?: string
    site_types?: SiteType[]
    proxy_target_type?: ProxyTargetType | null
    proxy_target_url?: string | null
    proxy_target_app_id?: string | null
    proxy_target_app_port?: number | null
    root_dir?: string | null
}

export const websitesApi = {
    async list(): Promise<WebsiteInfo[]> {
        const response = await api.get('/websites')
        return response.data
    },

    async get(id: number): Promise<WebsiteInfo> {
        const response = await api.get(`/websites/${id}`)
        return response.data
    },

    async create(payload: CreateWebsiteRequest): Promise<WebsiteInfo> {
        const response = await api.post('/websites', payload)
        return response.data
    },

    async update(
        id: number,
        payload: UpdateWebsiteRequest,
    ): Promise<WebsiteInfo> {
        const response = await api.put(`/websites/${id}`, payload)
        return response.data
    },

    async remove(id: number): Promise<void> {
        await api.delete(`/websites/${id}`)
    },
}
