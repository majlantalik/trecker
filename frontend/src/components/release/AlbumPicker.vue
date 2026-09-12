<template>
  <Dialog
    v-model:visible="visible"
    modal
    header="Choose the album"
    :draggable="false"
    :pt="{ root: { class: 'tk-dialog ap-root' } }"
  >
    <p class="ap-subtitle">
      {{ candidates.length }} matches on MusicBrainz for <strong>{{ query }}</strong>
    </p>

    <!-- `autofocus` on the list, because once its opening animation ends the dialog focuses
         whatever carries it, or else its close button. Focused, the arrow keys work at once,
         which matters in the palette where the whole flow is driven from the keyboard. -->
    <Listbox
      :model-value="null"
      :options="candidates"
      option-label="title"
      auto-option-focus
      list-style="max-height: min(60vh, 520px)"
      class="ap-list"
      :pt="{ list: { autofocus: true } }"
      @change="onChange"
    >
      <template #option="{ option }">
        <div class="ap-option" :class="{ 'is-choosing': choosingId === option.musicbrainzReleaseGroupId }">
          <img
            v-if="!missingCovers.has(option.musicbrainzReleaseGroupId)"
            :src="coverSrc(option.albumArtUrl)"
            class="ap-art"
            alt=""
            @error="missingCovers.add(option.musicbrainzReleaseGroupId)"
          />
          <div v-else class="ap-art ap-art-empty"><i class="pi pi-music" /></div>

          <div class="ap-text">
            <span class="ap-title">{{ option.title }}</span>
            <span class="ap-artist">{{ option.artist }}</span>
            <span v-if="describeCandidate(option)" class="ap-meta">{{ describeCandidate(option) }}</span>
          </div>

          <i v-if="choosingId === option.musicbrainzReleaseGroupId" class="pi pi-spin pi-spinner ap-busy" />
        </div>
      </template>
    </Listbox>

    <div class="ap-footer">
      <span class="ap-hint">
        <kbd>↑</kbd><kbd>↓</kbd> to move
        <span class="ap-sep">·</span>
        <kbd>Enter</kbd> to choose
        <span class="ap-sep">·</span>
        <kbd>Esc</kbd> to close
      </span>
      <Button label="None of these" text size="small" :disabled="!!choosingId" @click="emit('none')" />
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'
import Dialog from 'primevue/dialog'
import Listbox from 'primevue/listbox'
import Button from 'primevue/button'
import { coverSrc } from '@/api/cache'
import { describeCandidate } from '@/utils/candidates'
import type { AlbumCandidate } from '@/types'

/**
 * The list quick add shows when a search matches more than one album. Choosing one hands
 * it back; the parent fetches its details and opens the add form.
 */
const props = defineProps<{
  candidates: AlbumCandidate[]
  query: string
  /** The candidate being looked up, which shows a spinner and blocks another pick. */
  choosingId: string | null
}>()

const emit = defineEmits<{ choose: [candidate: AlbumCandidate]; none: [] }>()

const visible = defineModel<boolean>('visible', { default: false })

// Many albums have no cover in the archive, and the address answers 404. Remembered per
// search so a missing cover shows a placeholder rather than a broken image.
const missingCovers = reactive(new Set<string>())
watch(() => props.candidates, () => missingCovers.clear())

function onChange(event: { value: AlbumCandidate | null }) {
  if (event.value && !props.choosingId) emit('choose', event.value)
}
</script>

<style scoped>
.ap-subtitle {
  margin: 0 0 0.85rem;
  font-size: 0.82rem;
  color: rgba(226, 228, 240, 0.55);
}

.ap-subtitle strong {
  color: var(--tk-text);
  font-weight: 500;
}

.ap-list {
  width: 100%;
}

.ap-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  min-width: 0;
}

.ap-art {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
}

.ap-art-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(226, 228, 240, 0.3);
}

.ap-text {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
  flex: 1;
}

.ap-title,
.ap-artist,
.ap-meta {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ap-title {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--tk-text);
}

.ap-artist {
  font-size: 0.82rem;
  color: rgba(226, 228, 240, 0.7);
}

.ap-meta {
  font-size: 0.74rem;
  color: rgba(226, 228, 240, 0.45);
}

.ap-busy {
  color: var(--tk-accent);
  flex-shrink: 0;
}

.ap-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-top: 0.85rem;
}

.ap-hint {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.72rem;
  color: rgba(226, 228, 240, 0.4);
}

.ap-sep {
  opacity: 0.5;
}
</style>

<style>
/* Unscoped: PrimeVue teleports the dialog outside this component. */
.ap-root {
  width: min(560px, calc(100vw - 3rem));
}
</style>
