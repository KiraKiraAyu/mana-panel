<template>
    <button
        type="button"
        class="cursor-pointer rounded-full overflow-hidden relative flex gap-2 items-center group"
        :class="classes"
    >
        <div class="text-lg">
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
    </button>
</template>

<script lang="ts" setup>
import { computed } from 'vue'

interface Props {
    variant?: 'outline' | 'emphasis'
    text: string
}

const props = withDefaults(defineProps<Props>(), { variant: 'outline' })

const candidate = {
    outline:
        'h-10 py-2 px-4 text-sm text-text-primary border-1 border-reisa-lilac-700',
    emphasis:
        'h-10 py-2 px-4 text-sm bg-reisa-lilac-500 hover:bg-reisa-lilac-400',
}

const classes = computed(() => {
    return candidate[props.variant]
})
</script>
