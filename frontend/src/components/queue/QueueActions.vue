<template>
  <div class="queue-actions">
    <Button
      icon="pi pi-headphones"
      label="Log"
      size="small"
      @click="$emit('log', release)"
    />
    <template v-for="(url, service) in release.streamingLinks" :key="service">
      <Button
        icon="pi pi-external-link"
        size="small"
        severity="secondary"
        text
        :href="url"
        tag="a"
        target="_blank"
        :aria-label="`Open on ${service}`"
      />
    </template>
    <Button
      icon="pi pi-trash"
      size="small"
      severity="danger"
      text
      @click="$emit('delete', release)"
      aria-label="Remove from queue"
    />
  </div>
</template>

<script setup lang="ts">
import Button from 'primevue/button'
import type { Release } from '@/types'

defineProps<{ release: Release }>()
defineEmits<{
  log: [release: Release]
  delete: [release: Release]
}>()
</script>

<style scoped>
.queue-actions {
  display: flex;
  gap: 0.25rem;
}
</style>
