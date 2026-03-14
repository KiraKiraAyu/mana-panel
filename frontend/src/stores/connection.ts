import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ConnectionStatus = 'connected' | 'connecting' | 'disconnected'

export const useConnectionStore = defineStore('connection', () => {
    const status = ref<ConnectionStatus | null>(null)

    const set = (s: ConnectionStatus) => {
        status.value = s
    }

    const clear = () => {
        status.value = null
    }

    return { status, set, clear }
})
