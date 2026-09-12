<template>
  <Dialog
    v-model:visible="open"
    modal
    header="Keyboard shortcuts"
    :draggable="false"
    class="shortcuts-help"
    :pt="{ root: { class: 'sch-root' } }"
  >
    <div v-for="group in SHORTCUT_GROUPS" :key="group" class="sch-group">
      <h3 class="sch-group-title">{{ group }}</h3>
      <div v-for="(s, i) in byGroup(group)" :key="i" class="sch-row">
        <span class="sch-description">{{ s.description }}</span>
        <span class="sch-keys">
          <kbd v-for="(k, ki) in s.keys" :key="ki">{{ displayKey(k) }}</kbd>
        </span>
      </div>
    </div>

    <p class="sch-note">
      Shortcuts are ignored while you are typing, apart from the two that open quick add
      and the one that closes a dialog.
    </p>
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Dialog from 'primevue/dialog'
import { useShortcut } from '@/composables/useShortcut'
import { SHORTCUTS, SHORTCUT_GROUPS, displayKey } from '@/composables/shortcuts'

const open = ref(false)

// Not `whileTyping`: "?" is a character people type into notes and search boxes.
useShortcut('?', () => {
  open.value = !open.value
})

function byGroup(group: string) {
  return SHORTCUTS.filter((s) => s.group === group)
}
</script>

<style scoped>
.sch-group + .sch-group {
  margin-top: 1.4rem;
}

.sch-group-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0 0 0.6rem;
}

.sch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  padding: 0.4rem 0;
}

.sch-description {
  font-size: 0.9rem;
  color: var(--tk-text);
}

.sch-keys {
  display: inline-flex;
  gap: 0.3rem;
  flex-shrink: 0;
}

kbd {
  font-family: inherit;
  font-size: 0.72rem;
  min-width: 1.7rem;
  text-align: center;
  padding: 0.15rem 0.45rem;
  border-radius: 5px;
  border: 1px solid var(--tk-border-hover);
  border-bottom-width: 2px;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(226, 228, 240, 0.85);
}

.sch-note {
  margin: 1.5rem 0 0;
  padding-top: 1rem;
  border-top: 1px solid var(--tk-border);
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.45);
}
</style>

<style>
/* Unscoped: PrimeVue teleports the dialog outside this component. */
.sch-root {
  width: min(460px, calc(100vw - 3rem));
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.sch-root .p-dialog-header,
.sch-root .p-dialog-content {
  background: transparent;
}

.sch-root .p-dialog-header {
  padding: 1rem 1.25rem 0.5rem;
}

.sch-root .p-dialog-content {
  padding: 0.5rem 1.25rem 1.25rem;
  border-radius: 0 0 14px 14px;
}
</style>
