<template>
  <div class="settings-view">
    <div class="settings-header">
      <div class="settings-avatar"><i class="pi pi-cog" /></div>
      <div class="settings-header-text">
        <h1 class="settings-title">Settings</h1>
        <p class="settings-subtitle">How Trecker behaves on this computer.</p>
      </div>
    </div>

    <!-- One place for a failed save, whichever section it came from. -->
    <p v-if="error" class="settings-error" role="alert">{{ error }}</p>

    <div class="settings-section">
      <div class="section-header">
        <i class="pi pi-window-maximize section-icon" />
        <h2 class="section-title">Window</h2>
      </div>
      <div class="section-body">
        <p class="tk-label">When you close the window</p>
        <div v-for="option in CLOSE_OPTIONS" :key="option.value" class="choice">
          <RadioButton
            :input-id="`close-${option.value}`"
            name="close-action"
            :value="option.value"
            :model-value="settings?.closeAction"
            :disabled="!settings || saving"
            @update:model-value="(value: CloseAction) => update({ closeAction: value })"
          />
          <label :for="`close-${option.value}`" class="choice-text">
            <span class="choice-title">{{ option.title }}</span>
            <span class="choice-description">{{ option.description }}</span>
          </label>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <div class="section-header">
        <i class="pi pi-inbox section-icon" />
        <h2 class="section-title">Queue</h2>
      </div>
      <div class="section-body">
        <p class="tk-label">Order albums by the date you added them</p>
        <div v-for="option in QUEUE_OPTIONS" :key="option.value" class="choice">
          <RadioButton
            :input-id="`queue-${option.value}`"
            name="queue-sort"
            :value="option.value"
            :model-value="settings?.queueSort"
            :disabled="!settings || saving"
            @update:model-value="(value: QueueSort) => update({ queueSort: value })"
          />
          <label :for="`queue-${option.value}`" class="choice-text">
            <span class="choice-title">{{ option.title }}</span>
            <span class="choice-description">{{ option.description }}</span>
          </label>
        </div>
        <p class="form-hint">The button at the top of the queue changes this too.</p>
      </div>
    </div>

    <div class="settings-section">
      <div class="section-header">
        <i class="pi pi-bolt section-icon" />
        <h2 class="section-title">Quick add from anywhere</h2>
      </div>
      <div class="section-body">
        <p class="section-text">
          Open quick add from any app with a key of your choice. Trecker cannot claim a key
          for the whole desktop by itself, so your desktop does it: in its keyboard settings,
          add a custom shortcut that runs this command.
        </p>
        <div class="command-row">
          <InputText
            ref="commandInput"
            :model-value="command"
            readonly
            fluid
            class="command-input"
            aria-label="Quick add command"
            @focus="selectCommand"
          />
          <Button label="Copy" icon="pi pi-copy" size="small" outlined :disabled="!command" @click="copy" />
        </div>
        <p v-if="copyMessage" class="copy-message">{{ copyMessage }}</p>
        <p class="form-hint">
          If Trecker is not running, the shortcut starts it first, which takes a moment. To
          have quick add appear at once, keep Trecker running in the tray.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import RadioButton from 'primevue/radiobutton'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { appApi } from '@/api/app'
import { useSettings } from '@/composables/useSettings'
import type { CloseAction, QueueSort } from '@/types'

const CLOSE_OPTIONS: { value: CloseAction; title: string; description: string }[] = [
  {
    value: 'quit',
    title: 'Quit Trecker',
    description: 'Closing the window closes the app, like any other window.'
  },
  {
    value: 'tray',
    title: 'Keep running in the tray',
    description:
      'The window hides and an icon stays in the tray, with Open, Quick add and Quit. Starting Trecker again also brings the window back.'
  }
]

const QUEUE_OPTIONS: { value: QueueSort; title: string; description: string }[] = [
  {
    value: 'newest',
    title: 'Newest first',
    description: 'What you added last is at the top.'
  },
  {
    value: 'oldest',
    title: 'Oldest first',
    description: 'Work through the queue in the order albums arrived.'
  }
]

const { settings, saving, error, load, update } = useSettings()

const command = ref('')
const copyMessage = ref('')
const commandInput = ref<{ $el: HTMLInputElement } | null>(null)

function selectCommand(event: FocusEvent) {
  ;(event.target as HTMLInputElement).select()
}

async function copy() {
  try {
    await navigator.clipboard.writeText(command.value)
    copyMessage.value = 'Copied'
  } catch {
    // The clipboard can be refused to a webview. Selecting the text leaves one keystroke.
    commandInput.value?.$el?.focus()
    copyMessage.value = 'Press Ctrl+C to copy the selected command'
  }
}

onMounted(async () => {
  load()
  try {
    command.value = await appApi.quickAddCommand()
  } catch {
    command.value = 'trecker --quick-add'
  }
})
</script>

<style scoped>
.settings-view {
  max-width: 720px;
  margin: 0 auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--tk-border);
}

.settings-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.25), rgba(0, 229, 176, 0.08));
  border: 1px solid rgba(0, 229, 176, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
  color: var(--tk-accent);
  flex-shrink: 0;
}

.settings-header-text {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.settings-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin: 0;
  color: var(--tk-text);
}

.settings-subtitle {
  font-size: 0.875rem;
  color: rgba(226, 228, 240, 0.55);
  margin: 0;
}

.settings-section {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.section-icon {
  font-size: 0.875rem;
  color: var(--tk-accent);
}

.section-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0;
}

.section-body {
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.tk-label {
  margin: 0;
}

.choice {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.choice :deep(.p-radiobutton) {
  margin-top: 0.15rem;
}

.choice-text {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  cursor: pointer;
}

.choice-title {
  font-size: 0.9rem;
  color: var(--tk-text);
}

.choice-description {
  font-size: 0.78rem;
  color: rgba(226, 228, 240, 0.5);
}

.section-text {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.5;
  color: rgba(226, 228, 240, 0.75);
}

.command-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.command-input {
  font-family: ui-monospace, 'SF Mono', Menlo, monospace;
  font-size: 0.82rem;
}

.copy-message {
  margin: 0;
  font-size: 0.78rem;
  color: var(--tk-accent);
}

.form-hint {
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.45);
  margin: 0;
}

.settings-error {
  color: #f43f5e;
  font-size: 0.85rem;
  margin: 0;
}
</style>
