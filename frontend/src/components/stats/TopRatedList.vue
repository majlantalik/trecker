<template>
  <div class="chart-card">
    <h3>Top Rated</h3>
    <div v-if="releases.length" class="top-list">
      <div v-for="(r, i) in releases" :key="r.id" class="top-item">
        <span class="rank">{{ i + 1 }}</span>
        <img v-if="r.albumArtUrl" :src="r.albumArtUrl" class="item-art" />
        <div class="item-info">
          <RouterLink :to="`/entry/${r.id}`" class="item-title">{{ r.artist }} – {{ r.title }}</RouterLink>
          <div class="item-meta">
            <HalfStarRating :modelValue="r.rating" readonly />
            <span v-if="r.releaseYear" class="year">{{ r.releaseYear }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="empty-state">No rated releases yet.</div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import type { Release } from '@/types'

defineProps<{ releases: Release[] }>()
</script>

<style scoped>
.chart-card {
  background: var(--p-surface-900);
  border: 1px solid var(--p-surface-700);
  border-radius: 8px;
  padding: 1.25rem;
}

.chart-card h3 {
  margin: 0 0 1rem;
  font-size: 1rem;
  color: var(--p-text-muted-color);
}

.top-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.top-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem;
  border-radius: 6px;
  transition: background 0.1s;
}

.top-item:hover {
  background: var(--p-surface-800);
}

.rank {
  width: 1.5rem;
  text-align: center;
  font-weight: 700;
  color: var(--p-text-muted-color);
  font-size: 0.85rem;
}

.item-art {
  width: 40px;
  height: 40px;
  object-fit: cover;
  border-radius: 3px;
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title {
  display: block;
  font-size: 0.9rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--p-text-color);
  text-decoration: none;
}

.item-title:hover {
  color: var(--p-primary-color);
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.15rem;
}

.year {
  font-size: 0.75rem;
  color: var(--p-text-muted-color);
}

.empty-state {
  color: var(--p-text-muted-color);
  text-align: center;
  padding: 2rem;
}
</style>
