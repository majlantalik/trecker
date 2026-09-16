<template>
  <div class="chart-card">
    <div class="ye-header">
      <h3>Year End List</h3>
      <fieldset class="basis-toggle">
        <legend class="visually-hidden">Rank albums by the year they were</legend>
        <button
          v-for="option in BASES"
          :key="option.value"
          type="button"
          class="basis-btn"
          :class="{ active: by === option.value }"
          :aria-pressed="by === option.value"
          @click="by !== option.value && $emit('by-change', option.value)"
        >
          {{ option.label }}
        </button>
      </fieldset>
      <div class="year-selector">
        <Button icon="pi pi-chevron-left" text size="small" @click="$emit('year-change', year - 1)" />
        <span class="year-label">{{ year }}</span>
        <Button icon="pi pi-chevron-right" text size="small" @click="$emit('year-change', year + 1)" :disabled="year >= currentYear" />
      </div>
    </div>

    <div v-if="entries.length" class="ye-list">
      <div v-for="entry in entries" :key="entry.rank" class="ye-item">
        <span class="rank">{{ entry.rank }}</span>
        <img v-if="entry.release.albumArtUrl" :src="coverSrc(entry.release.albumArtUrl)" class="item-art" alt="" />
        <div class="item-info">
          <RouterLink :to="`/entry/${entry.release.id}`" class="item-title">
            {{ entry.release.artist }} – {{ entry.release.title }}
          </RouterLink>
          <div class="item-meta">
            <HalfStarRating :modelValue="entry.release.rating" readonly />
          </div>
        </div>
      </div>
    </div>
    <div v-else class="empty-state">
      No rated albums {{ by === 'released' ? 'released' : 'listened to' }} in {{ year }}.
    </div>
  </div>
</template>

<script setup lang="ts">
import { coverSrc } from '@/api/cache'
import { RouterLink } from 'vue-router'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import Button from 'primevue/button'
import type { YearEndBasis, YearEndEntry } from '@/types'

defineProps<{
  year: number
  by: YearEndBasis
  entries: YearEndEntry[]
}>()

defineEmits<{
  'year-change': [year: number]
  'by-change': [by: YearEndBasis]
}>()

const BASES: { label: string; value: YearEndBasis }[] = [
  { label: 'Listened', value: 'listened' },
  { label: 'Released', value: 'released' }
]

const currentYear = new Date().getFullYear()
</script>

<style scoped>
.chart-card {
  background: var(--p-surface-900);
  border: 1px solid var(--p-surface-700);
  border-radius: 8px;
  padding: 1.25rem;
}

.ye-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.ye-header h3 {
  margin: 0;
  font-size: 1rem;
  color: var(--p-text-muted-color);
}

.basis-toggle {
  display: flex;
  margin: 0 auto 0 1rem;
  padding: 2px;
  border: 1px solid var(--p-surface-700);
  border-radius: 6px;
  min-inline-size: 0;
}

/* Read by screen readers, not shown. */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

.basis-btn {
  padding: 0.2rem 0.7rem;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--p-text-muted-color);
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;
}

.basis-btn:hover {
  color: var(--p-text-color);
}

.basis-btn:focus-visible {
  outline: 2px solid var(--p-primary-color);
  outline-offset: 1px;
}

.basis-btn.active {
  background: var(--p-surface-800);
  color: var(--p-primary-color);
  font-weight: 600;
}

.year-selector {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.year-label {
  font-weight: 700;
  min-width: 3rem;
  text-align: center;
}

.ye-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ye-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem;
  border-radius: 6px;
}

.ye-item:hover {
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
  margin-top: 0.15rem;
}

.empty-state {
  color: var(--p-text-muted-color);
  text-align: center;
  padding: 2rem;
}
</style>
