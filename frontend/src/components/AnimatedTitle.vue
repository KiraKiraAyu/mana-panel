<template>
    <div>
        <h1
            class="text-2xl inline-block font-bold select-none relative overflow-hidden"
        >
            <span
                ref="overlay"
                class="absolute inset-0 bg-reisa-lilac-800 text-transparent opacity-0"
            >
                {{ text }}
            </span>
            <div ref="textContainer" class="inset-0">
                <span
                    v-for="(char, i) in letters"
                    :key="i"
                    ref="letterRefs"
                    class="inline-block bg-clip-text [text-shadow:0_0_6px_rgb(246,111,168)]"
                    :class="{ 'w-[0.3em]': char === ' ' }"
                    >{{ char === ' ' ? '\u00A0' : char }}</span
                >
            </div>
        </h1>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'

const text = 'ManaPanel'
const interval = 5

const letters = [...text]
const textContainer = ref<HTMLElement>()
const letterRefs = ref<HTMLSpanElement[]>([])
const overlay = ref<HTMLSpanElement>()

let timeline: gsap.core.Timeline | null = null
let intervalId: ReturnType<typeof setInterval> | null = null

function play() {
    if (!textContainer.value || !letterRefs.value.length || !overlay.value)
        return

    const width = textContainer.value.offsetWidth

    timeline = gsap.timeline()

    timeline.set(overlay.value, { x: -width })

    timeline.to(letterRefs.value, {
        y: 20,
        rotation: 60,
        opacity: 0,
        duration: 0.2,
        ease: 'power2.in',
        stagger: { each: 0.08, from: 'end' },
    })

    timeline.set(letterRefs.value, { y: 0, rotation: 0, opacity: 1 }, '+=0.4')
    timeline.set(textContainer.value, { x: -width })
    timeline.set(overlay.value, { opacity: 1 })

    timeline.to(overlay.value, { x: 0, duration: 0.6, ease: 'power4.out' })

    timeline.to(
        textContainer.value,
        {
            x: 0,
            duration: 0.8,
            ease: 'power4.out',
        },
        '-=0.4',
    )

    timeline.to(
        overlay.value,
        { x: width, duration: 0.6, ease: 'power4.out' },
        '-=0.5',
    )
}

onMounted(() => {
    intervalId = setInterval(play, interval * 1000)
})

onUnmounted(() => {
    if (intervalId) clearInterval(intervalId)
    if (timeline) timeline.kill()
})
</script>
