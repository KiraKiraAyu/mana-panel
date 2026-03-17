<template>
    <button
        :type="type"
        class="cursor-pointer overflow-hidden relative inline-flex items-center gap-2 h-10 group disabled:cursor-not-allowed disabled:opacity-50 transition-colors duration-300"
        :class="classes"
    >
        <template v-if="text">
            <div class="text-lg" v-if="$slots.default">
                <slot></slot>
            </div>
            <div class="relative overflow-hidden inline-flex">
                <span
                    class="inline-block transition-transform duration-400 ease-[cubic-bezier(.17,.67,.39,1.35)] group-hover:-translate-y-8"
                    >{{ text }}</span
                >
                <span
                    class="inline-block absolute inset-0 transition-transform duration-400 ease-[cubic-bezier(.17,.67,.39,1.35)] translate-y-8 group-hover:translate-y-0"
                    >{{ text }}</span
                >
            </div>
        </template>
        <slot v-else></slot>
    </button>
</template>

<script lang="ts" setup>
import { computed } from 'vue'

interface Props {
    variant?: 'outline' | 'emphasis' | 'square'
    text?: string
    type?: 'button' | 'submit' | 'reset'
}

const props = withDefaults(defineProps<Props>(), {
    variant: 'outline',
    text: '',
    type: 'button',
})

const candidate = {
    outline:
        'rounded-full py-2 px-4 text-sm bg-transparent border border-border text-text-secondary hover:bg-surface-elevated hover:text-text-primary',
    emphasis:
        'rounded-full py-2 px-4 text-sm border-0 bg-linear-to-br from-reisa-lilac-500 to-reisa-lilac-600 text-white hover:from-reisa-lilac-400 hover:to-reisa-lilac-500',
    square: 'rounded-lg  aspect-square justify-center',
}

const classes = computed(() => {
    return candidate[props.variant]
})
</script>
