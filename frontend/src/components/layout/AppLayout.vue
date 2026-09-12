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

    <!-- Mounted once, alongside the header bar. Opens on Shift Shift from anywhere. -->
    <QuickAddPalette />
    <ShortcutsHelp />

    <!-- Shows that a sequence is half-entered. Without it, a lone "g" looks like a key
         that did nothing at all. -->
    <Transition name="prefix">
      <div v-if="pendingPrefix" class="key-prefix-hint"><kbd>G</kbd><span>…</span></div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import AppSidebar from './AppSidebar.vue'
import QuickAddBar from '@/components/release/QuickAddBar.vue'
import QuickAddPalette from '@/components/release/QuickAddPalette.vue'
import ShortcutsHelp from '@/components/common/ShortcutsHelp.vue'
import { useRouter } from 'vue-router'
import { useKeySequence } from '@/composables/useKeySequence'

const router = useRouter()

// Gmail-style: the second key is the first letter of where you are going.
const { pending: pendingPrefix } = useKeySequence('g', {
  q: () => router.push('/queue'),
  l: () => router.push('/library'),
  s: () => router.push('/stats'),
  i: () => router.push('/info')
})
</script>

<style scoped>
.key-prefix-hint {
  position: fixed;
  left: 1.25rem;
  bottom: 1.25rem;
  z-index: 1100;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.7rem;
  border-radius: 8px;
  background: var(--tk-surface);
  border: 1px solid var(--tk-border-hover);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  color: rgba(226, 228, 240, 0.6);
  font-size: 0.8rem;
  pointer-events: none;
}

.key-prefix-hint kbd {
  font-family: inherit;
  font-size: 0.72rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  border: 1px solid var(--tk-border-hover);
  background: rgba(255, 255, 255, 0.05);
  color: var(--tk-accent);
}

.prefix-enter-active,
.prefix-leave-active {
  transition: opacity 0.12s ease, transform 0.12s ease;
}

.prefix-enter-from,
.prefix-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

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
