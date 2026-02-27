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
:root {
  --tk-bg: #0d0f1a;
  --tk-sidebar-bg: #09090f;
  --tk-header-bg: rgba(13, 15, 26, 0.88);
  --tk-surface: #13162a;
  --tk-surface-hover: #1a1e35;
  --tk-border: rgba(255, 255, 255, 0.07);
  --tk-border-hover: rgba(255, 255, 255, 0.14);
  --tk-text: #e2e4f0;
  --tk-text-muted: #5a6080;
  --tk-accent: #00e5b0;
  --tk-accent-hover: #00ffc8;
  --tk-accent-dim: rgba(0, 229, 176, 0.12);
  --tk-accent-glow: rgba(0, 229, 176, 0.28);
  --tk-font-display: 'Syne', ui-sans-serif, system-ui, sans-serif;
  --tk-font-body: 'Outfit', ui-sans-serif, system-ui, sans-serif;
}

*, *::before, *::after {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  background: var(--tk-bg);
}

body {
  font-family: var(--tk-font-body);
  font-size: 18px;
  color: var(--tk-text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

h1, h2, h3, h4 {
  font-family: var(--tk-font-display);
  letter-spacing: -0.025em;
  margin: 0;
}

::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 229, 176, 0.25);
}
</style>
