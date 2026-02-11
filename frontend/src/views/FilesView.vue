<template>
    <div
        class="p-6 space-y-6 animate-[fade-in_0.3s_ease-out,slide-up_0.3s_ease-out]"
    >
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-text-primary">Files</h1>
                <p class="text-text-muted mt-1">
                    Browse and manage server files
                </p>
            </div>
            <div class="flex items-center gap-2">
                <input
                    ref="uploadInput"
                    type="file"
                    class="hidden"
                    @change="handleUpload"
                />
                <button
                    @click="triggerUpload"
                    class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-5 py-2.5 text-sm font-medium text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
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
                            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
                        />
                    </svg>
                    Upload
                </button>
                <button
                    @click="createFolder"
                    class="inline-flex items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)]"
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
                            d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"
                        />
                    </svg>
                    New Folder
                </button>
            </div>
        </div>

        <!-- Breadcrumb -->
        <div class="flex items-center gap-2 text-sm">
            <template v-for="(part, index) in pathParts()" :key="part.path">
                <button
                    @click="navigateTo(part.path)"
                    class="text-text-secondary hover:text-reisa-lilac-400 transition-colors"
                    :class="{
                        'text-text-primary font-medium':
                            index === pathParts().length - 1,
                    }"
                >
                    {{ part.name }}
                </button>
                <span
                    v-if="index < pathParts().length - 1"
                    class="text-text-muted"
                    >/</span
                >
            </template>
        </div>

        <!-- File List -->
        <div
            class="overflow-hidden rounded-xl border border-border-subtle bg-surface-elevated p-0 transition-all duration-300 hover:border-reisa-lilac-600 hover:shadow-[0_8px_32px_oklch(0_0_0/0.3),0_0_0_1px_oklch(0.66_0.058_301/0.1)]"
        >
            <table
                class="w-full border-collapse [&_th]:border-b [&_th]:border-border-subtle [&_th]:bg-surface [&_th]:px-4 [&_th]:py-3.5 [&_th]:text-left [&_th]:text-xs [&_th]:font-semibold [&_th]:uppercase [&_th]:tracking-wider [&_th]:text-text-muted [&_td]:border-b [&_td]:border-border-subtle [&_td]:px-4 [&_td]:py-3.5 [&_td]:text-left [&_tbody_tr]:transition-colors [&_tbody_tr:hover]:bg-surface-elevated"
            >
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Size</th>
                        <th>Modified</th>
                        <th>Permissions</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <!-- Parent Directory -->
                    <tr
                        v-if="currentPath !== '/'"
                        @click="navigateUp"
                        class="cursor-pointer"
                    >
                        <td class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg bg-reisa-stripe-500/20 flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-reisa-stripe-400"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M11 17l-5-5m0 0l5-5m-5 5h12"
                                    />
                                </svg>
                            </div>
                            <span class="font-medium">..</span>
                        </td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                    </tr>

                    <!-- Files -->
                    <tr
                        v-for="file in files"
                        :key="file.path"
                        @dblclick="openFile(file)"
                        class="cursor-pointer"
                    >
                        <td class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg flex items-center justify-center"
                                :class="
                                    file.is_dir
                                        ? 'bg-reisa-gold-500/20'
                                        : 'bg-reisa-lilac-500/20'
                                "
                            >
                                <!-- Folder Icon -->
                                <svg
                                    v-if="file.is_dir"
                                    class="w-4 h-4 text-reisa-gold-400"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                                    />
                                </svg>
                                <!-- File Icon -->
                                <svg
                                    v-else
                                    class="w-4 h-4 text-reisa-lilac-400"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                                    />
                                </svg>
                            </div>
                            <span class="font-medium text-text-primary">{{
                                file.name
                            }}</span>
                        </td>
                        <td class="text-text-secondary">
                            {{ file.is_dir ? '-' : formatBytes(file.size) }}
                        </td>
                        <td class="text-text-secondary text-sm">
                            {{ formatDate(file.modified) }}
                        </td>
                        <td class="font-mono text-sm text-text-muted">
                            {{ file.permissions }}
                        </td>
                        <td>
                            <div class="flex items-center gap-1">
                                <button
                                    v-if="!file.is_dir"
                                    @click.stop="downloadFile(file)"
                                    class="p-1.5 rounded-lg hover:bg-reisa-lilac-500/20 text-reisa-lilac-400 transition-colors"
                                    title="Download"
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
                                            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                                        />
                                    </svg>
                                </button>
                                <button
                                    @click.stop="deleteFile(file)"
                                    class="p-1.5 rounded-lg hover:bg-error/20 text-error transition-colors"
                                    title="Delete"
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
                                </button>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>

            <div v-if="loading" class="p-8 text-center">
                <div
                    class="h-8 w-48 mx-auto rounded bg-[linear-gradient(90deg,var(--color-surface)_0%,var(--color-surface-elevated)_50%,var(--color-surface)_100%)] bg-size-[200%_100%] animate-[shimmer_1.5s_infinite]"
                ></div>
            </div>

            <div
                v-if="!loading && files.length === 0"
                class="p-8 text-center text-text-muted"
            >
                This directory is empty
            </div>
        </div>

        <!-- File Editor Modal -->
        <div
            v-if="showEditor"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
            @click.self="showEditor = false"
        >
            <div
                class="w-full max-w-4xl bg-surface-elevated rounded-xl shadow-2xl animate-[fade-in_0.3s_ease-out,slide-up_0.3s_ease-out]"
            >
                <div
                    class="flex items-center justify-between p-4 border-b border-border-subtle"
                >
                    <h3 class="font-semibold text-text-primary">
                        {{ selectedFile?.name }}
                    </h3>
                    <div class="flex items-center gap-2">
                        <button
                            @click="saveFile"
                            class="inline-flex items-center justify-center gap-2 rounded-lg border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 px-5 py-2.5 text-sm font-medium text-white transition-all duration-200 hover:-translate-y-px hover:from-reisa-lilac-400 hover:to-reisa-lilac-500 hover:shadow-[0_4px_16px_oklch(0.66_0.058_301/0.4)]"
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
                                    d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"
                                />
                            </svg>
                            Save
                        </button>
                        <button
                            @click="showEditor = false"
                            class="inline-flex items-center justify-center gap-2 rounded-lg border border-border bg-transparent px-5 py-2.5 text-sm font-medium text-text-secondary transition-all duration-200 hover:border-reisa-lilac-500 hover:bg-surface-elevated hover:text-text-primary"
                        >
                            Close
                        </button>
                    </div>
                </div>
                <textarea
                    v-model="editingContent"
                    class="w-full h-[60vh] p-4 bg-surface font-mono text-sm text-text-primary resize-none focus:outline-none"
                    spellcheck="false"
                ></textarea>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'

interface FileEntry {
    name: string
    path: string
    is_dir: boolean
    size: number
    modified: number
    permissions: string
    owner: string
    group: string
}

const currentPath = ref('/')
const files = ref<FileEntry[]>([])
const loading = ref(false)
const selectedFile = ref<FileEntry | null>(null)
const editingContent = ref('')
const showEditor = ref(false)
const uploadInput = ref<HTMLInputElement | null>(null)

const fetchFiles = async (path: string = currentPath.value) => {
    loading.value = true
    try {
        const response = await api.get('/files', { params: { path } })
        files.value = response.data
        currentPath.value = path
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to load files')
    } finally {
        loading.value = false
    }
}

const navigateTo = (path: string) => {
    fetchFiles(path)
}

const navigateUp = () => {
    const parts = currentPath.value.split('/').filter(Boolean)
    parts.pop()
    const newPath = '/' + parts.join('/')
    fetchFiles(newPath || '/')
}

const openFile = async (file: FileEntry) => {
    if (file.is_dir) {
        navigateTo(file.path)
    } else {
        // Open in editor
        try {
            const response = await api.get('/files/content', {
                params: { path: file.path },
            })
            selectedFile.value = file
            editingContent.value = response.data.content
            showEditor.value = true
        } catch (e: any) {
            alert(e.response?.data?.error?.message || 'Failed to open file')
        }
    }
}

const saveFile = async () => {
    if (!selectedFile.value) return
    try {
        await api.put('/files/content', {
            path: selectedFile.value.path,
            content: editingContent.value,
        })
        showEditor.value = false
        alert('File saved successfully')
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to save file')
    }
}

const downloadFile = (file: FileEntry) => {
    window.open(
        `/api/files/download?path=${encodeURIComponent(file.path)}`,
        '_blank',
    )
}

const deleteFile = async (file: FileEntry) => {
    if (!confirm(`Are you sure you want to delete ${file.name}?`)) return
    try {
        await api.delete('/files', { params: { path: file.path } })
        await fetchFiles()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to delete')
    }
}

const createFolder = async () => {
    const name = prompt('Enter folder name:')
    if (!name) return
    try {
        await api.post('/files/mkdir', {
            path: `${currentPath.value}/${name}`,
        })
        await fetchFiles()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to create folder')
    }
}

const triggerUpload = () => {
    uploadInput.value?.click()
}

const handleUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    if (!file) return

    const formData = new FormData()
    formData.append('path', `${currentPath.value}/${file.name}`)
    formData.append('file', file)

    try {
        await api.post('/files/upload', formData, {
            headers: { 'Content-Type': 'multipart/form-data' },
        })
        await fetchFiles()
    } catch (e: any) {
        alert(e.response?.data?.error?.message || 'Failed to upload')
    }

    input.value = ''
}

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '-'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const formatDate = (timestamp: number) => {
    if (!timestamp) return '-'
    return new Date(timestamp * 1000).toLocaleString()
}

onMounted(() => {
    fetchFiles()
})

const pathParts = () => {
    const parts = currentPath.value.split('/').filter(Boolean)
    const result: { name: string; path: string }[] = [
        { name: 'Root', path: '/' },
    ]
    let fullPath = ''
    for (const part of parts) {
        fullPath += '/' + part
        result.push({ name: part, path: fullPath })
    }
    return result
}
</script>
