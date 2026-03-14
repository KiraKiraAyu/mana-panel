<template>
    <div class="h-full flex flex-col p-6 animate-in">
        <!-- Terminal -->
        <div
            class="flex-1 rounded-xl overflow-hidden border border-border-subtle bg-[#0f0f14]"
        >
            <div ref="terminalContainer" class="h-full w-full p-2"></div>
        </div>

        <!-- Keyboard Shortcuts -->
        <div class="mt-4 flex items-center gap-6 text-xs text-text-muted">
            <span
                ><kbd
                    class="px-1.5 py-0.5 rounded bg-surface border border-border-subtle"
                    >Ctrl+C</kbd
                >
                Interrupt</span
            >
            <span
                ><kbd
                    class="px-1.5 py-0.5 rounded bg-surface border border-border-subtle"
                    >Ctrl+D</kbd
                >
                Exit</span
            >
            <span
                ><kbd
                    class="px-1.5 py-0.5 rounded bg-surface border border-border-subtle"
                    >Ctrl+L</kbd
                >
                Clear</span
            >
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    ref,
    watch,
    onMounted,
    onBeforeUnmount,
    onUnmounted,
    nextTick,
} from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { useConnectionStore } from '@/stores/connection'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

const terminalContainer = ref<HTMLDivElement | null>(null)
const terminal = ref<Terminal | null>(null)
const fitAddon = ref<FitAddon | null>(null)
const socket = ref<WebSocket | null>(null)
const resizeObserver = ref<ResizeObserver | null>(null)
const connectionStore = useConnectionStore()
const terminalDataHandler = ref<{ dispose: () => void } | null>(null)
const isConnected = ref(false)
const isConnecting = ref(false)
const isUnmounted = ref(false)
const isTearingDown = ref(false)

const handleGlobalPointerDown = (event: PointerEvent) => {
    const container = terminalContainer.value
    if (!container) return

    const target = event.target as Node | null
    if (target && container.contains(target)) return

    const activeElement = document.activeElement as HTMLElement | null
    if (activeElement && container.contains(activeElement)) {
        activeElement.blur()
    }
}

const connect = () => {
    if (
        socket.value?.readyState === WebSocket.OPEN ||
        socket.value?.readyState === WebSocket.CONNECTING
    ) {
        return
    }

    isConnecting.value = true

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//${window.location.host}/api/terminal/ws`
    const ws = new WebSocket(wsUrl)
    socket.value = ws

    ws.onopen = () => {
        if (socket.value !== ws || isUnmounted.value) return
        isConnected.value = true
        isConnecting.value = false
        terminal.value?.focus()
    }

    ws.onmessage = async (event) => {
        if (socket.value !== ws || isUnmounted.value) return

        if (event.data instanceof Blob) {
            const text = await event.data.text()
            if (socket.value !== ws || isUnmounted.value) return
            terminal.value?.write(text)
        } else {
            terminal.value?.write(event.data)
        }
    }

    ws.onclose = () => {
        if (socket.value === ws) {
            socket.value = null
        }
        isConnected.value = false
        isConnecting.value = false

        if (!isUnmounted.value) {
            terminal.value?.write('\r\n\x1b[31mConnection closed\x1b[0m\r\n')
        }
    }

    ws.onerror = () => {
        if (socket.value !== ws && socket.value !== null) return
        isConnected.value = false
        isConnecting.value = false

        if (!isUnmounted.value) {
            terminal.value?.write('\r\n\x1b[31mConnection error\x1b[0m\r\n')
        }
    }
}

const disconnect = () => {
    const ws = socket.value
    if (ws) {
        ws.onopen = null
        ws.onmessage = null
        ws.onclose = null
        ws.onerror = null
        if (
            ws.readyState === WebSocket.OPEN ||
            ws.readyState === WebSocket.CONNECTING
        ) {
            ws.close()
        }
    }

    socket.value = null
    isConnected.value = false
    isConnecting.value = false
}

const initTerminal = () => {
    if (!terminalContainer.value) return

    terminal.value = new Terminal({
        theme: {
            background: '#0f0f14',
            foreground: '#e8e6f0',
            cursor: '#c9a5ff',
            cursorAccent: '#0f0f14',
            selectionBackground: '#c9a5ff33',
            black: '#1a1a24',
            red: '#ff6b6b',
            green: '#69db7c',
            yellow: '#ffd43b',
            blue: '#748ffc',
            magenta: '#c9a5ff',
            cyan: '#66d9e8',
            white: '#e8e6f0',
            brightBlack: '#2d2d3d',
            brightRed: '#ff8787',
            brightGreen: '#8ce99a',
            brightYellow: '#ffe066',
            brightBlue: '#91a7ff',
            brightMagenta: '#d9b8ff',
            brightCyan: '#99e9f2',
            brightWhite: '#f8f9fa',
        },
        fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace',
        fontSize: 14,
        lineHeight: 1.2,
        cursorBlink: true,
        cursorStyle: 'block',
        scrollback: 10000,
        allowProposedApi: true,
    })

    fitAddon.value = new FitAddon()
    terminal.value.loadAddon(fitAddon.value)

    terminal.value.open(terminalContainer.value)
    fitAddon.value.fit()

    // Handle input
    terminalDataHandler.value = terminal.value.onData((data) => {
        if (socket.value?.readyState === WebSocket.OPEN) {
            socket.value.send(data)
        }
    })

    // Handle resize
    resizeObserver.value = new ResizeObserver(() => {
        fitAddon.value?.fit()
    })
    resizeObserver.value.observe(terminalContainer.value)
}

const teardownTerminal = () => {
    document.removeEventListener('pointerdown', handleGlobalPointerDown, true)

    if (isTearingDown.value) return
    isTearingDown.value = true

    disconnect()

    terminalDataHandler.value?.dispose()
    terminalDataHandler.value = null

    resizeObserver.value?.disconnect()
    resizeObserver.value = null

    terminal.value?.dispose()
    terminal.value = null
    fitAddon.value = null

    const activeElement = document.activeElement as HTMLElement | null
    if (
        activeElement &&
        terminalContainer.value &&
        terminalContainer.value.contains(activeElement)
    ) {
        activeElement.blur()
    }

    if (terminalContainer.value) {
        terminalContainer.value.replaceChildren()
    }
}

onBeforeRouteLeave((_to, _from, next) => {
    teardownTerminal()
    next()
})

watch(
    [isConnected, isConnecting],
    ([connected, connecting]) => {
        if (connected) connectionStore.set('connected')
        else if (connecting) connectionStore.set('connecting')
        else connectionStore.set('disconnected')
    },
    { immediate: true },
)

onBeforeUnmount(() => {
    isUnmounted.value = true
    teardownTerminal()
})

onMounted(async () => {
    isUnmounted.value = false
    isTearingDown.value = false
    await nextTick()
    document.addEventListener('pointerdown', handleGlobalPointerDown, true)
    initTerminal()
    connect()
})

onUnmounted(() => {
    connectionStore.clear()
    teardownTerminal()
})
</script>

<style scoped>
:deep(.xterm) {
    padding: 8px;
}

:deep(.xterm-viewport) {
    overflow-y: auto !important;
}

:deep(.xterm-viewport::-webkit-scrollbar) {
    width: 8px;
}

:deep(.xterm-viewport::-webkit-scrollbar-track) {
    background: transparent;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb) {
    background: oklch(0.46 0.044 301);
    border-radius: 4px;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb:hover) {
    background: oklch(0.56 0.052 301);
}
</style>
