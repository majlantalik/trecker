<template>
  <div
    class="artist-card"
    :class="{ 'is-checked': artist.status === 'CHECKED' }"
    role="link"
    tabindex="0"
    @click="$emit('click', artist)"
    @keydown.enter="$emit('click', artist)"
  >
    <ArtistAvatar :artist="artist" :size="72" />

    <div class="card-info">
      <div class="card-name">{{ artist.name }}</div>
      <div class="card-meta">
        <CountryLabel v-if="artist.country" :value="artist.country" compact />
        <span v-if="describeArtist(artist)">{{ describeArtist(artist) }}</span>
      </div>
      <div v-if="artist.genres.length" class="card-genres">
        <Tag v-for="genre in artist.genres.slice(0, 3)" :key="genre" :value="genre" severity="secondary" />
      </div>
      <p v-if="artist.note" class="card-note">{{ artist.note }}</p>
    </div>

    <div v-if="artist.status === 'CHECKED'" class="card-verdict" :class="artist.verdict?.toLowerCase()">
      <i :class="artist.verdict ? VERDICTS[artist.verdict].icon : 'pi pi-check'" />
      {{ verdictLabel(artist) }}
    </div>
  </div>
</template>

<script setup lang="ts">
import Tag from 'primevue/tag'
import CountryLabel from '@/components/common/CountryLabel.vue'
import ArtistAvatar from './ArtistAvatar.vue'
import { describeArtist, verdictLabel, VERDICTS } from '@/utils/artists'
import type { Artist } from '@/types'

defineProps<{ artist: Artist }>()
defineEmits<{ click: [artist: Artist] }>()
</script>

<style scoped>
.artist-card {
  display: flex;
  align-items: center;
  gap: 1.125rem;
  padding: 1rem 1.125rem;
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  background: var(--tk-surface);
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease, transform 0.15s ease;
}

.artist-card:hover,
.artist-card:focus-visible {
  outline: none;
  border-color: rgba(0, 229, 176, 0.35);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(0, 229, 176, 0.1);
  transform: translateY(-1px);
}

.artist-card.is-checked {
  opacity: 0.75;
}

.artist-card.is-checked:hover {
  opacity: 1;
}

.card-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.card-name {
  font-family: var(--tk-font-display);
  font-weight: 600;
  font-size: 1.05rem;
  color: var(--tk-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: rgba(226, 228, 240, 0.55);
  min-width: 0;
}

.card-meta span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-genres {
  display: flex;
  gap: 0.35rem;
  flex-wrap: wrap;
  margin-top: 0.15rem;
}

.card-note {
  margin: 0.15rem 0 0;
  font-size: 0.82rem;
  font-style: italic;
  color: rgba(226, 228, 240, 0.6);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-verdict {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  flex-shrink: 0;
  padding: 0.3rem 0.65rem;
  border-radius: 999px;
  font-size: 0.78rem;
  color: rgba(226, 228, 240, 0.7);
  border: 1px solid var(--tk-border);
}

.card-verdict.liked {
  color: var(--tk-accent);
  border-color: rgba(0, 229, 176, 0.3);
  background: var(--tk-accent-dim);
}

.card-verdict.not_for_me {
  color: rgba(248, 113, 113, 0.9);
  border-color: rgba(248, 113, 113, 0.3);
}
</style>
