<template>
    <div
        v-if="variant === 'search'"
        class="relative w-full h-10 text-xl flex items-center"
    >
        <input
            ref="inputRef"
            type="text"
            class="peer absolute rounded-full bg-reisa-lilac-900 w-full h-full px-4 outline-none transition-all"
            :value="inputValue"
            @input="handleInput"
            v-bind="forwardedAttrs"
        />

        <Icon
            icon="mdi:magnify"
            class="absolute text-reisa-lilac-700 z-10 text-3xl ml-4 transition-opacity duration-300 pointer-events-none peer-focus:opacity-0"
            :class="{ 'opacity-0': hasValue }"
        />

        <div
            class="absolute flex items-center justify-center right-4 cursor-pointer transition-opacity duration-300 w-10 h-10"
            :class="{ 'opacity-0': !hasValue }"
            @click="clearValue"
        >
            <span
                class="absolute w-6 h-0.5 bg-reisa-lilac-700 rotate-45"
            ></span>
            <span
                class="absolute w-6 h-0.5 bg-reisa-lilac-700 -rotate-45"
            ></span>
        </div>
    </div>
    <input
        v-else
        ref="inputRef"
        class="input"
        :value="inputValue"
        @input="handleInput"
        v-bind="forwardedAttrs"
    />
</template>

<script lang="ts" setup>
import { computed, getCurrentInstance, ref, useAttrs } from 'vue'
import { Icon } from '@iconify/vue'

defineOptions({ inheritAttrs: false })

const props = defineProps<{
    variant?: 'search' | 'form'
    modelValue?: string | number
    modelModifiers?: {
        number?: boolean
    }
}>()

const emit = defineEmits<{
    (event: 'update:modelValue', value: string | number): void
}>()

const attrs = useAttrs()
const inputRef = ref<HTMLInputElement | null>(null)
const instance = getCurrentInstance()

const hasModelBinding = computed(() => {
    const vnodeProps = instance?.vnode.props
    return Boolean(
        vnodeProps &&
        Object.prototype.hasOwnProperty.call(vnodeProps, 'modelValue'),
    )
})

const inputValue = computed(() => {
    if (hasModelBinding.value) {
        return props.modelValue ?? ''
    }

    const externalValue = attrs.value
    return externalValue == null ? '' : (externalValue as string | number)
})

const forwardedAttrs = computed(() => {
    const { value, ...rest } = attrs
    return rest
})

const hasValue = computed(() => String(inputValue.value).trim().length > 0)

const castValue = (raw: string): string | number => {
    if (!props.modelModifiers?.number) return raw

    const parsed = Number.parseFloat(raw)
    return Number.isNaN(parsed) ? raw : parsed
}

const handleInput = (event: Event) => {
    if (!hasModelBinding.value) return
    const target = event.target as HTMLInputElement
    emit('update:modelValue', castValue(target.value))
}

const clearValue = () => {
    if (hasModelBinding.value) {
        emit('update:modelValue', '')
    }

    const input = inputRef.value
    if (!input) return

    input.value = ''
    input.dispatchEvent(new Event('input', { bubbles: true }))
}
</script>
