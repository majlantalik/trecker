<template>
  <Dialog
    v-model:visible="visible"
    modal
    header="Choose the artist"
    :draggable="false"
    :pt="{ root: { class: 'tk-dialog arp-root' } }"
  >
    <p class="arp-subtitle">
      {{ candidates.length === 1 ? '1 match' : `${candidates.length} matches` }} on MusicBrainz for
      <strong>{{ query }}</strong>
    </p>

    <!-- `autofocus` for the same reason as AlbumPicker: the dialog focuses it once open, so the
         arrow keys work at once. -->
    <Listbox
      :model-value="null"
      :options="candidates"
      option-label="name"
      auto-option-focus
      list-style="max-height: min(60vh, 520px)"
      class="arp-list"
      :pt="{ list: { autofocus: true } }"
      @change="onChange"
    >
      <template #option="{ option }">
        <div class="arp-option" :class="{ 'is-adding': addingId === option.musicbrainzArtistId }">
          <div class="arp-avatar" aria-hidden="true">{{ initial(option.name) }}</div>

          <div class="arp-text">
            <span class="arp-name">{{ option.name }}</span>
            <span class="arp-meta">
              <CountryLabel v-if="option.country" :value="option.country" compact />
              <span v-if="describeArtist(option)">{{ describeArtist(option) }}</span>
            </span>
            <span v-if="option.tags.length" class="arp-tags">{{ option.tags.join(', ') }}</span>
          </div>

          <i v-if="addingId === option.musicbrainzArtistId" class="pi pi-spin pi-spinner arp-busy" />
        </div>
      </template>
    </Listbox>

    <p v-if="error" class="arp-error">{{ error }}</p>

    <div class="arp-footer">
      <span class="arp-hint">
        <kbd>↑</kbd><kbd>↓</kbd> to move
        <span class="arp-sep">·</span>
        <kbd>Enter</kbd> to add
        <span class="arp-sep">·</span>
        <kbd>Esc</kbd> to close
      </span>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import Dialog from 'primevue/dialog'
import Listbox from 'primevue/listbox'
import CountryLabel from '@/components/common/CountryLabel.vue'
import { describeArtist, initial } from '@/utils/artists'
import type { ArtistCandidate } from '@/types'

/** The matches for an artist search. Choosing one hands it back for the parent to add. */
const props = defineProps<{
  candidates: ArtistCandidate[]
  query: string
  /** The candidate being added, which shows a spinner and blocks another pick. */
  addingId: string | null
  error?: string
}>()

const emit = defineEmits<{ choose: [candidate: ArtistCandidate] }>()

const visible = defineModel<boolean>('visible', { default: false })

function onChange(event: { value: ArtistCandidate | null }) {
  if (event.value && !props.addingId) emit('choose', event.value)
}
</script>

<style scoped>
.arp-subtitle {
  margin: 0 0 0.85rem;
  font-size: 0.82rem;
  color: rgba(226, 228, 240, 0.55);
}

.arp-subtitle strong {
  color: var(--tk-text);
  font-weight: 500;
}

.arp-list {
  width: 100%;
}

.arp-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  min-width: 0;
}

.arp-avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--tk-font-display);
  font-weight: 700;
  font-size: 1.05rem;
  color: var(--tk-accent);
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.22), rgba(0, 229, 176, 0.06));
  border: 1px solid rgba(0, 229, 176, 0.2);
}

.arp-text {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
  flex: 1;
}

.arp-name,
.arp-meta,
.arp-tags {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.arp-name {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--tk-text);
}

.arp-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: rgba(226, 228, 240, 0.7);
}

.arp-tags {
  font-size: 0.74rem;
  color: rgba(226, 228, 240, 0.6);
}

.arp-busy {
  color: var(--tk-accent);
  flex-shrink: 0;
}

.arp-error {
  margin: 0.75rem 0 0;
  font-size: 0.82rem;
  color: #f43f5e;
}

.arp-footer {
  display: flex;
  align-items: center;
  margin-top: 0.85rem;
}

.arp-hint {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.72rem;
  color: rgba(226, 228, 240, 0.6);
}

.arp-sep {
  opacity: 0.5;
}
</style>

<style>
/* Unscoped: PrimeVue teleports the dialog outside this component. */
.arp-root {
  width: min(560px, calc(100vw - 3rem));
}
</style>
