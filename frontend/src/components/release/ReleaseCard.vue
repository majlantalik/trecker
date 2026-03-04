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
        <HalfStarRating :modelValue="release.rating" readonly />
      </div>
    </div>
    <div class="card-actions" @click.stop>
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import Tag from 'primevue/tag'
import type { Release } from '@/types'

defineProps<{ release: Release }>()
defineEmits<{ click: [release: Release] }>()
</script>

<style scoped>
.release-card {
  display: flex;
  gap: 1.125rem;
  padding: 1.125rem;
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  background: var(--tk-surface);
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease, transform 0.15s ease;
}

.release-card:hover {
  border-color: rgba(0, 229, 176, 0.35);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(0, 229, 176, 0.1);
  transform: translateY(-1px);
}

.card-art {
  width: 88px;
  height: 88px;
  flex-shrink: 0;
}

.card-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 8px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.45);
}

.art-placeholder {
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--tk-border);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--tk-text-muted);
  font-size: 1.25rem;
}

.card-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.card-title {
  font-family: var(--tk-font-display);
  font-weight: 600;
  font-size: 1.05rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--tk-text);
  letter-spacing: -0.01em;
}

.card-artist {
  color: var(--tk-text-muted);
  font-size: 0.9rem;
  margin-top: 0.2rem;
}

.card-meta {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.3rem;
  font-size: 0.78rem;
  color: var(--tk-text-muted);
  opacity: 0.7;
}

.card-genres {
  display: flex;
  gap: 0.35rem;
  flex-wrap: wrap;
  margin-top: 0.45rem;
}

.card-rating {
  margin-top: 0.35rem;
}

.card-actions {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 0.4rem;
}
</style>
