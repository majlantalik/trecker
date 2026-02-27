<template>
  <div class="quick-add-bar">
    <div class="input-wrapper">
      <i class="pi pi-plus-circle input-icon" />
      <InputText
        v-model="inputValue"
        placeholder="Paste a Spotify/Tidal link or type an artist + album..."
        class="quick-add-input"
        @paste="handlePaste"
        @keydown.enter="handleEnter"
        fluid
      />
      <Button
        v-if="inputValue"
        icon="pi pi-times"
        text
        rounded
        size="small"
        class="clear-btn"
        @click="inputValue = ''"
        aria-label="Clear"
      />
    </div>

    <Button
      label="Add"
      icon="pi pi-search"
      :loading="resolving"
      :disabled="!inputValue.trim()"
      @click="handleAdd"
    />

    <ReleaseForm
      v-model:visible="showForm"
      :prefill="resolvedMetadata"
      @submit="handleFormSubmit"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { useToast } from 'primevue/usetoast'
import ReleaseForm from './ReleaseForm.vue'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { ResolvedMetadata } from '@/types'

const inputValue = ref('')
const resolving = ref(false)
const showForm = ref(false)
const resolvedMetadata = ref<ResolvedMetadata | null>(null)
const toast = useToast()
const releasesStore = useReleasesStore()
const genresStore = useGenresStore()

const URL_PATTERN = /^https?:\/\//

async function handlePaste(event: ClipboardEvent) {
  const text = event.clipboardData?.getData('text') || ''
  if (URL_PATTERN.test(text)) {
    event.preventDefault()
    inputValue.value = text
    await resolve(text, true)
  }
}

async function handleEnter() {
  await handleAdd()
}

async function handleAdd() {
  const val = inputValue.value.trim()
  if (!val) return
  await resolve(val, URL_PATTERN.test(val))
}

async function resolve(value: string, isUrl: boolean) {
  resolving.value = true
  resolvedMetadata.value = null
  try {
    const request = isUrl ? { url: value } : { query: value }
    const metadata = await releasesApi.resolve(request)
    resolvedMetadata.value = metadata
  } catch (e) {
    // Open form with empty prefill for manual entry
    resolvedMetadata.value = null
  } finally {
    resolving.value = false
    showForm.value = true
  }
}

async function handleFormSubmit(data: any) {
  try {
    const release = await releasesStore.addRelease(data)
    if (data.genres) {
      data.genres.forEach((g: string) => genresStore.addGenre(g))
    }
    toast.add({
      severity: 'success',
      summary: 'Added to queue',
      detail: `${release.artist} – ${release.title}`,
      life: 3000
    })
    inputValue.value = ''
    resolvedMetadata.value = null
  } catch (e: any) {
    toast.add({
      severity: 'error',
      summary: 'Failed to add',
      detail: e.message,
      life: 4000
    })
  }
}
</script>

<style scoped>
.quick-add-bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  max-width: 800px;
}

.input-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: 0.75rem;
  color: var(--p-text-muted-color);
  pointer-events: none;
  z-index: 1;
}

.quick-add-input {
  padding-left: 2.25rem !important;
  padding-right: 2rem !important;
}

.clear-btn {
  position: absolute;
  right: 0.25rem;
}
</style>
