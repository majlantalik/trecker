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
            <img v-if="option.albumArtUrl" :src="coverSrc(option.albumArtUrl)" class="suggestion-art" alt="" />
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
      :loading="busy"
      :disabled="!inputValue.trim()"
      @click="handleAdd"
    />

    <AlbumPicker
      v-model:visible="pickerVisible"
      :candidates="candidates"
      :query="query"
      :choosing-id="choosingId"
      @choose="choose"
      @none="chooseNone"
    />

    <ReleaseForm
      v-model:visible="formVisible"
      :prefill="prefill"
      @submit="handleFormSubmit"
    />
  </div>
</template>

<script setup lang="ts">
import { coverSrc } from '@/api/cache'
import { computed, defineAsyncComponent, ref, watch, onUnmounted } from 'vue'
import AutoComplete from 'primevue/autocomplete'
import Button from 'primevue/button'
import { useToast } from 'primevue/usetoast'
import { useAlbumSearch } from '@/composables/useAlbumSearch'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { ResolvedMetadata } from '@/types'

// The bar sits in the header of every page, so it is in the entry chunk. These two dialogs
// open only on demand and would otherwise pull InputNumber, Listbox and the rest of the
// form into that chunk too.
const AlbumPicker = defineAsyncComponent(() => import('./AlbumPicker.vue'))
const ReleaseForm = defineAsyncComponent(() => import('./ReleaseForm.vue'))

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
// Pasted links still resolve to a single album: a link names one release, and the
// streaming services it comes from cannot be searched anyway.
const resolvingLink = ref(false)
const {
  query,
  searching,
  candidates,
  pickerVisible,
  choosingId,
  prefill,
  formVisible,
  error: searchError,
  search,
  choose,
  chooseNone,
  openForm
} = useAlbumSearch()
const busy = computed(() => resolvingLink.value || searching.value || !!choosingId.value)
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
  const label = `${event.value.artist} – ${event.value.title}`
  acValue.value = label
  inputValue.value = label
  suggestions.value = []
  openForm(event.value)
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
    await resolveLink(text)
  }
}

async function handleEnter() {
  await handleAdd()
}

async function handleAdd() {
  const val = inputValue.value.trim()
  if (!val || busy.value) return
  if (URL_PATTERN.test(val)) {
    await resolveLink(val)
    return
  }

  const outcome = await search(val)
  if (outcome === 'none') {
    toast.add({
      severity: 'info',
      summary: 'Nothing found on MusicBrainz',
      detail: 'Fill in the details yourself.',
      life: 3500
    })
  } else if (outcome === 'failed') {
    toast.add({
      severity: 'warn',
      summary: 'Could not search MusicBrainz',
      detail: searchError.value,
      life: 4000
    })
  }
}

async function resolveLink(url: string) {
  resolvingLink.value = true
  try {
    openForm(await releasesApi.resolve({ url }))
  } catch {
    openForm(null)
  } finally {
    resolvingLink.value = false
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
    prefill.value = null
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
  color: rgba(226, 228, 240, 0.55);
}

.suggestion-label {
  font-size: 0.875rem;
  color: var(--tk-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suggestion-year {
  color: rgba(226, 228, 240, 0.6);
  font-size: 0.8rem;
}
</style>
