<template>
  <div class="release-card" @click="$emit('click', release)">
    <div class="card-art">
      <img v-if="release.albumArtUrl" :src="release.albumArtUrl" :alt="`${release.artist} - ${release.title}`" />
      <div v-else class="art-placeholder">
        <i class="pi pi-music" />
      </div>
    </div>
    <div class="card-info">
      <div class="card-title">{{ release.title }}</div>
      <div class="card-artist">{{ release.artist }}</div>
      <div class="card-meta">
        <span v-if="release.releaseYear" class="meta-item">{{ release.releaseYear }}</span>
        <span v-if="release.country" class="meta-item">{{ release.country }}</span>
      </div>
      <div class="card-genres" v-if="release.genres.length">
        <Tag v-for="genre in release.genres.slice(0, 3)" :key="genre" :value="genre" severity="secondary" />
      </div>
      <div class="card-rating" v-if="release.rating">
        <Rating :modelValue="release.rating" :stars="5" readonly />
      </div>
    </div>
    <div class="card-actions" @click.stop>
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import Rating from 'primevue/rating'
import Tag from 'primevue/tag'
import type { Release } from '@/types'

defineProps<{ release: Release }>()
defineEmits<{ click: [release: Release] }>()
</script>

<style scoped>
.release-card {
  display: flex;
  gap: 1rem;
  padding: 0.75rem;
  border: 1px solid var(--p-surface-700);
  border-radius: 8px;
  background: var(--p-surface-900);
  cursor: pointer;
  transition: border-color 0.15s;
}

.release-card:hover {
  border-color: var(--p-primary-color);
}

.card-art {
  width: 64px;
  height: 64px;
  flex-shrink: 0;
}

.card-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 4px;
}

.art-placeholder {
  width: 100%;
  height: 100%;
  background: var(--p-surface-700);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--p-text-muted-color);
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-weight: 600;
  font-size: 0.95rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-artist {
  color: var(--p-text-muted-color);
  font-size: 0.85rem;
  margin-top: 0.15rem;
}

.card-meta {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.25rem;
  font-size: 0.75rem;
  color: var(--p-text-muted-color);
}

.card-genres {
  display: flex;
  gap: 0.35rem;
  flex-wrap: wrap;
  margin-top: 0.35rem;
}

.card-actions {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 0.35rem;
}
</style>
