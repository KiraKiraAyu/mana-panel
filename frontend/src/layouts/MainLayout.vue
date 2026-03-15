<template>
    <div class="flex min-h-screen">
        <SideBar
            class="fixed h-screen w-64 bg-surface border-r border-border-subtle flex flex-col"
        ></SideBar>

        <main class="ml-64 flex-1 bg-magic overflow-auto">
            <RouterView />
        </main>
    </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import SideBar from '@/components/SideBar.vue'
</script>
