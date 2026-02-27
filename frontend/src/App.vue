<template>
  <template v-if="route.meta.public">
    <RouterView />
  </template>
  <template v-else>
    <AppLayout>
      <RouterView />
    </AppLayout>
  </template>
  <Toast position="bottom-right" />
</template>

<script setup lang="ts">
import { useRoute, RouterView } from 'vue-router'
import Toast from 'primevue/toast'
import AppLayout from '@/components/layout/AppLayout.vue'

const route = useRoute()
</script>

<style>
body {
  font-family: 'Inter', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>
