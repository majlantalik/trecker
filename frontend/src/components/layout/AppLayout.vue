<template>
  <div class="app-layout">
    <AppSidebar />
    <div class="app-main">
      <header class="app-header">
        <QuickAddBar />
      </header>
      <main class="app-content">
        <slot />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppSidebar from './AppSidebar.vue'
import QuickAddBar from '@/components/release/QuickAddBar.vue'
</script>

<style scoped>
.app-layout {
  display: flex;
  min-height: 100vh;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--p-surface-700);
  background: var(--p-surface-950);
}

.app-content {
  flex: 1;
  padding: 1.5rem;
  overflow-y: auto;
}
</style>
