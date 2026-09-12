<template>
  <Dialog
    v-model:visible="open"
    modal
    :closable="false"
    :draggable="false"
    :show-header="false"
    :dismissable-mask="true"
    class="quick-add-palette"
    :pt="{ root: { class: 'tk-dialog qap-root' }, mask: { class: 'qap-mask' } }"
    @show="onShow"
  >
    <QuickAddBar ref="barRef" palette @added="open = false" />
    <div class="qap-hint">
      <kbd>Enter</kbd> to look up
      <span class="qap-sep">·</span>
      <kbd>Esc</kbd> to close
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import Dialog from 'primevue/dialog'
import QuickAddBar from './QuickAddBar.vue'
import { useDoubleTap } from '@/composables/useDoubleTap'
import { useShortcut } from '@/composables/useShortcut'

const open = ref(false)
const barRef = ref<InstanceType<typeof QuickAddBar> | null>(null)

// Shift twice, as in JetBrains. Ctrl+K is kept as an alias because it is what most people
// try first, and because a shortcut nobody can guess is a shortcut nobody uses.
useDoubleTap('Shift', () => {
  open.value = !open.value
})
useShortcut('ctrl+k', () => {
  open.value = true
}, { whileTyping: true })

async function onShow() {
  // The dialog animates in, so the input does not exist yet when the event fires.
  await nextTick()
  barRef.value?.focus()
}
</script>

<style scoped>
.qap-hint {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.85rem;
  font-size: 0.72rem;
  color: rgba(226, 228, 240, 0.4);
}

.qap-sep {
  opacity: 0.5;
}

</style>

<style>
/* Unscoped: PrimeVue renders the dialog into a teleport outside this component. */
.qap-root {
  width: min(640px, calc(100vw - 3rem));
}

.qap-root .p-dialog-content {
  padding: 1.1rem 1.25rem 1rem;
}

/* Sit the palette near the top, where a launcher belongs, rather than dead centre. */
.qap-mask {
  align-items: flex-start;
  padding-top: 14vh;
}
</style>
