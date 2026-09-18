<template>
  <div class="queue-actions">
    <Button
      icon="pi pi-headphones"
      label="Log"
      size="small"
      @click="$emit('log', release)"
    />
    <Button
      icon="pi pi-copy"
      size="small"
      severity="secondary"
      text
      v-tooltip.bottom="'Copy “Artist - Album”'"
      aria-label="Copy artist and album"
      @click="copyAlbum(release)"
    />
    <OpenLinkButton :links="release.streamingLinks" />
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
import OpenLinkButton from '@/components/common/OpenLinkButton.vue'
import { useCopyAlbum } from '@/composables/useCopyAlbum'
import type { Release } from '@/types'

defineProps<{ release: Release }>()
defineEmits<{
  log: [release: Release]
  delete: [release: Release]
}>()

// Copying needs nothing from the queue, so it stays here rather than travelling up as a
// third event the view would only hand straight back to the composable.
const { copyAlbum } = useCopyAlbum()
</script>

<style scoped>
.queue-actions {
  display: flex;
  gap: 0.25rem;
}
</style>
