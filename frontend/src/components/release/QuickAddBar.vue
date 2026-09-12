<template>
  <div class="quick-add-bar" :class="{ 'is-palette': palette }">
    <div class="input-wrapper">
      <i class="pi pi-plus-circle input-icon" />
      <AutoComplete
        ref="acRef"
        class="quick-add-ac"
        v-model="acValue"
        :suggestions="suggestions"
        @complete="fetchSuggestions"
        @item-select="handleCatalogSelect"
        @keydown.enter="handleEnter"
        :pt="{
          input: { onPaste: handlePaste },
          panel: { style: 'background: var(--tk-surface); border: 1px solid var(--tk-border); border-radius: 10px;' },
          option: { style: 'padding: 0.4rem 0.875rem;' }
        }"
        option-label="title"
        :placeholder="palette ? 'Type an artist and album, then press Enter...' : 'Type an artist and album, or paste a link...'"
        auto-highlight
      >
        <template #option="{ option }">
          <div class="catalog-suggestion">
            <img v-if="option.albumArtUrl" :src="option.albumArtUrl" class="suggestion-art" alt="" />
            <div v-else class="suggestion-art-placeholder"><i class="pi pi-music" /></div>
            <span class="suggestion-label">
              {{ option.artist }} &mdash; {{ option.title }}
              <span v-if="option.releaseYear" class="suggestion-year">({{ option.releaseYear }})</span>
            </span>
          </div>
        </template>
      </AutoComplete>
      <Button
        v-if="inputValue"
        icon="pi pi-times"
        text
        rounded
        size="small"
        class="clear-btn"
        @click="clearInput"
        aria-label="Clear"
      />
    </div>

    <Button
      v-if="!palette"
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
import { ref, watch, onUnmounted } from 'vue'
import AutoComplete from 'primevue/autocomplete'
import Button from 'primevue/button'
import { useToast } from 'primevue/usetoast'
import ReleaseForm from './ReleaseForm.vue'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { ResolvedMetadata } from '@/types'

const props = withDefaults(
  defineProps<{
    /** Rendered inside the Shift-Shift palette rather than the header bar. */
    palette?: boolean
  }>(),
  { palette: false }
)

const emit = defineEmits<{ added: []; resolving: [value: boolean] }>()

const acValue = ref<string | ResolvedMetadata>('')
const inputValue = ref('')
const suggestions = ref<ResolvedMetadata[]>([])
const resolving = ref(false)
const showForm = ref(false)
const resolvedMetadata = ref<ResolvedMetadata | null>(null)
const toast = useToast()
const releasesStore = useReleasesStore()
const genresStore = useGenresStore()

const acRef = ref<{ $el: HTMLElement } | null>(null)

/** Focuses the text field. The palette calls this once its dialog has mounted. */
function focus() {
  const input = acRef.value?.$el?.querySelector('input')
  input?.focus()
}
defineExpose({ focus })

let debounceTimer: ReturnType<typeof setTimeout> | null = null

onUnmounted(() => { if (debounceTimer) clearTimeout(debounceTimer) })

watch(acValue, (val) => { if (typeof val === 'string') inputValue.value = val })

const URL_PATTERN = /^https?:\/\//

async function fetchSuggestions(event: { query: string }) {
  const q = event.query.trim()
  if (q.length < 2 || URL_PATTERN.test(q)) { suggestions.value = []; return }
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(async () => {
    try { suggestions.value = await releasesApi.searchCatalog(q) }
    catch { suggestions.value = [] }
  }, 300)
}

function handleCatalogSelect(event: { value: ResolvedMetadata }) {
  resolvedMetadata.value = event.value
  const label = `${event.value.artist} – ${event.value.title}`
  acValue.value = label
  inputValue.value = label
  suggestions.value = []
  showForm.value = true
}

function clearInput() {
  acValue.value = ''
  inputValue.value = ''
  suggestions.value = []
}

async function handlePaste(event: ClipboardEvent) {
  const text = event.clipboardData?.getData('text') || ''
  if (URL_PATTERN.test(text)) {
    event.preventDefault()
    acValue.value = text
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

function parseQueryText(input: string): ResolvedMetadata | null {
  const sep = input.indexOf(' - ')
  if (sep === -1) return null
  return {
    artist: input.slice(0, sep).trim(),
    title: input.slice(sep + 3).trim(),
    releaseYear: null,
    albumArtUrl: null,
    country: null,
    streamingLinks: {},
    genres: []
  }
}

async function resolve(value: string, isUrl: boolean) {
  resolving.value = true
  resolvedMetadata.value = null
  try {
    const request = isUrl ? { url: value } : { query: value }
    const metadata = await releasesApi.resolve(request)
    // If resolve succeeded but returned no identity for a plain query, fall back to parsed text
    if (!isUrl && metadata && !metadata.artist && !metadata.title) {
      resolvedMetadata.value = parseQueryText(value) ?? metadata
    } else {
      resolvedMetadata.value = metadata
    }
  } catch (e) {
    // All resolvers failed — pre-fill from typed text as last resort
    resolvedMetadata.value = isUrl ? null : parseQueryText(value)
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
    clearInput()
    resolvedMetadata.value = null
    emit('added')
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
  gap: 0.875rem;
  width: 100%;
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
  color: var(--tk-text-muted);
  pointer-events: none;
  z-index: 1;
}

.quick-add-ac {
  flex: 1;
  width: 100%;
}

.quick-add-ac :deep(input) {
  width: 100%;
  padding-left: 2.25rem !important;
  padding-right: 2rem !important;
}

.clear-btn {
  position: absolute;
  right: 0.25rem;
}

.catalog-suggestion {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.suggestion-art {
  width: 32px;
  height: 32px;
  object-fit: cover;
  border-radius: 4px;
  flex-shrink: 0;
}

.suggestion-art-placeholder {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.3);
}

.suggestion-label {
  font-size: 0.875rem;
  color: var(--tk-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suggestion-year {
  color: rgba(226, 228, 240, 0.45);
  font-size: 0.8rem;
}
</style>
