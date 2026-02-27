<template>
  <div class="app-layout">
    <AppSidebar />
    <div class="app-main">
      <header class="app-header">
        <div class="header-inner">
          <QuickAddBar />
        </div>
      </header>
      <main class="app-content">
        <div class="content-wrapper">
          <slot />
        </div>
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
  background: var(--tk-bg);
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-header {
  border-bottom: 1px solid var(--tk-border);
  background: var(--tk-header-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  justify-content: center;
}

.header-inner {
  width: 100%;
  padding: 1rem 2.5rem;
}

.app-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  background: radial-gradient(ellipse 80% 40% at 60% -10%, rgba(0, 229, 176, 0.04) 0%, transparent 70%),
              var(--tk-bg);
}

.content-wrapper {
  width: 100%;
  padding: 2.25rem 2.5rem;
  flex: 1;
  display: flex;
  flex-direction: column;
}
</style>
