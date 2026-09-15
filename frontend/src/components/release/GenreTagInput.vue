<template>
  <!-- Capture phase, so a typed genre is turned into a chip before AutoComplete sees the key.
       AutoComplete alone only makes a chip from a picked suggestion; Enter on typed text did
       nothing, and a comma was just a character. -->
  <div class="genre-input" @keydown.capture="onKeydown" @focusout="onFocusOut">
    <AutoComplete
      ref="ac"
      :model-value="modelValue"
      :suggestions="suggestions"
      multiple
      fluid
      :typeahead="true"
      :show-empty-message="false"
      :delay="120"
      :placeholder="modelValue.length ? '' : placeholder"
      :pt="{
        overlay: { class: 'tk-overlay' },
        input: { onPaste, 'aria-label': 'Add a genre' }
      }"
      @complete="search"
      @update:model-value="onPicked"
    >
      <template #chip="{ value, removeCallback }">
        <span class="genre-chip">
          {{ value }}
          <button
            type="button"
            class="genre-chip-remove"
            :aria-label="`Remove ${value}`"
            tabindex="-1"
            @click.stop="removeCallback"
          >
            <i class="pi pi-times" />
          </button>
        </span>
      </template>
    </AutoComplete>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AutoComplete from 'primevue/autocomplete'
import { useGenresStore } from '@/stores/genres'
import { addGenres, splitGenres, suggestGenres } from '@/utils/genres'

/**
 * Genres as chips. Type a genre and press Enter, Tab or a comma; paste a comma-separated
 * list; or pick from the genres already in your library, which are suggested as you type.
 */
const props = withDefaults(defineProps<{ modelValue: string[]; placeholder?: string }>(), {
  placeholder: 'Add genres...'
})

const emit = defineEmits<{ 'update:modelValue': [value: string[]] }>()

const genresStore = useGenresStore()
const suggestions = ref<string[]>([])
const ac = ref<{ $el: HTMLElement; hide?: () => void } | null>(null)

onMounted(() => genresStore.fetchGenres())

function input(): HTMLInputElement | null {
  return ac.value?.$el?.querySelector('input') ?? null
}

function search(event: { query: string }) {
  suggestions.value = suggestGenres(event.query, genresStore.genres, props.modelValue)
}

function commit(genres: string[]) {
  const next = addGenres(props.modelValue, genres, genresStore.genres)
  if (next.length !== props.modelValue.length) emit('update:modelValue', next)
}

/** Turns what is typed into chips and empties the field. False when nothing was typed. */
function commitTyped(): boolean {
  const el = input()
  const typed = el ? splitGenres(el.value) : []
  if (!el || typed.length === 0) return false
  commit(typed)
  el.value = ''
  ac.value?.hide?.()
  return true
}

function onKeydown(event: KeyboardEvent) {
  if (event.target !== input()) return
  // A suggestion highlighted with the arrow keys is AutoComplete's to pick.
  const highlighted = !!input()?.getAttribute('aria-activedescendant')

  if (event.key === ',' || event.key === ';') {
    event.preventDefault()
    event.stopPropagation()
    commitTyped()
  } else if ((event.key === 'Enter' || event.key === 'NumpadEnter') && !highlighted) {
    // Always swallowed: inside a form, Enter on this field must not submit it.
    event.preventDefault()
    event.stopPropagation()
    commitTyped()
  } else if (event.key === 'Tab' && !highlighted) {
    // Focus still moves on; only the typed genre is kept first.
    commitTyped()
  }
}

// Leaving the field keeps a half-typed genre rather than dropping it, which is what closed
// the log dialog with a genre missing. Deferred, because clicking a suggestion also blurs
// the field, and AutoComplete empties it once the pick lands.
function onFocusOut() {
  setTimeout(commitTyped, 150)
}

function onPaste(event: ClipboardEvent) {
  const text = event.clipboardData?.getData('text') ?? ''
  if (!/[,;\n]/.test(text)) return
  event.preventDefault()
  commit(splitGenres(text))
}

/** A suggestion was picked, or a chip removed, through AutoComplete itself. */
function onPicked(value: unknown) {
  const next = Array.isArray(value) ? value.map(String) : []
  // A pick adds to the end; run it through the same rules so case and duplicates agree.
  const removed = next.length < props.modelValue.length
  emit('update:modelValue', removed ? next : addGenres(props.modelValue, next.slice(props.modelValue.length), genresStore.genres))
}
</script>

<style scoped>
.genre-input {
  width: 100%;
}

.genre-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.15rem 0.3rem 0.15rem 0.6rem;
  border-radius: 999px;
  font-size: 0.82rem;
  line-height: 1.4;
  color: var(--tk-accent);
  background: var(--tk-accent-dim);
  border: 1px solid rgba(0, 229, 176, 0.25);
  white-space: nowrap;
}

.genre-chip-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.1rem;
  height: 1.1rem;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: rgba(0, 229, 176, 0.7);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.genre-chip-remove:hover {
  background: rgba(0, 229, 176, 0.18);
  color: var(--tk-accent-hover);
}

.genre-chip-remove i {
  font-size: 0.6rem;
}

:deep(.p-autocomplete-input-multiple) {
  gap: 0.35rem;
}

:deep(.p-autocomplete-chip-item) {
  display: inline-flex;
}
</style>
