<template>
  <div class="artist-avatar" :style="{ width: `${size}px`, height: `${size}px`, fontSize: `${size * 0.4}px` }">
    <!-- One of their album covers. Many albums have none in the archive, and the image fails
         to load; the initial takes its place. -->
    <img
      v-if="artist.imageUrl && !failed"
      :src="coverSrc(artist.imageUrl)"
      alt=""
      @error="failed = true"
    />
    <span v-else aria-hidden="true">{{ initial(artist.name) }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { coverSrc } from '@/api/cache'
import { initial } from '@/utils/artists'
import type { Artist } from '@/types'

const props = withDefaults(defineProps<{ artist: Pick<Artist, 'name' | 'imageUrl'>; size?: number }>(), {
  size: 72
})

const failed = ref(false)
watch(() => props.artist.imageUrl, () => (failed.value = false))
</script>

<style scoped>
.artist-avatar {
  flex-shrink: 0;
  border-radius: 50%;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--tk-font-display);
  font-weight: 700;
  color: var(--tk-accent);
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.22), rgba(0, 229, 176, 0.06));
  border: 1px solid rgba(0, 229, 176, 0.2);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.45);
}

.artist-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
