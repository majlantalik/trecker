<template>
  <Dialog
    v-model:visible="visible"
    :header="isEdit ? 'Edit Release' : 'Add to Queue'"
    modal
    :style="{ width: '720px' }"
    :draggable="false"
  >
    <form @submit.prevent="handleSubmit" class="release-form">
      <div class="form-row">
        <div class="form-field" style="flex: 2">
          <label>Artist *</label>
          <InputText v-model="form.artist" placeholder="Artist name" fluid required />
        </div>
        <div class="form-field" style="flex: 3">
          <label>Title *</label>
          <InputText v-model="form.title" placeholder="Album / EP title" fluid required />
        </div>
      </div>

      <div class="form-row">
        <div class="form-field">
          <label>Year</label>
          <InputNumber v-model="form.releaseYear" placeholder="2024" :min="1900" :max="2099" :useGrouping="false" fluid />
        </div>
      </div>

      <div class="form-field">
        <label>Genres</label>
        <GenreTagInput v-model="form.genres" />
      </div>

      <div class="form-row">
        <div class="form-field" style="flex: 1">
          <label>Discovery Link</label>
          <InputText v-model="form.discoveryLink" placeholder="YouTube, Discord..." fluid />
        </div>
        <div class="form-field" style="flex: 1">
          <label>Streaming Link</label>
          <InputText v-model="form.streamingLinkInput" placeholder="Spotify, Tidal..." fluid />
        </div>
      </div>

      <div class="form-field" v-if="form.albumArtUrl">
        <img :src="form.albumArtUrl" class="album-art-preview" alt="Album art" />
      </div>

      <div class="form-actions">
        <Button type="button" label="Cancel" severity="secondary" outlined @click="visible = false" />
        <Button type="submit" :label="isEdit ? 'Save' : 'Add to Queue'" :loading="saving" />
      </div>
    </form>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Button from 'primevue/button'
import GenreTagInput from './GenreTagInput.vue'
import type { Release, ResolvedMetadata } from '@/types'

const props = defineProps<{
  prefill?: ResolvedMetadata | null
  release?: Release | null
}>()

const emit = defineEmits<{
  submit: [data: any]
}>()

const visible = defineModel<boolean>('visible', { default: false })
const isEdit = ref(false)
const saving = ref(false)

const defaultForm = () => ({
  artist: '',
  title: '',
  releaseYear: null as number | null,
  albumArtUrl: '',
  country: '',
  discoveryLink: '',
  streamingLinkInput: '',
  streamingLinks: {} as Record<string, string>,
  musicbrainzId: '',
  genres: [] as string[]
})

const form = ref(defaultForm())

watch(visible, (val) => {
  if (val) {
    if (props.release) {
      isEdit.value = true
      const links = props.release.streamingLinks || {}
      form.value = {
        artist: props.release.artist,
        title: props.release.title,
        releaseYear: props.release.releaseYear,
        albumArtUrl: props.release.albumArtUrl || '',
        country: props.release.country || '',
        discoveryLink: props.release.discoveryLink || '',
        streamingLinkInput: Object.values(links)[0] || '',
        streamingLinks: { ...links },
              musicbrainzId: '',
        genres: [...props.release.genres]
      }
    } else if (props.prefill) {
      isEdit.value = false
      const links = props.prefill.streamingLinks || {}
      form.value = {
        artist: props.prefill.artist || '',
        title: props.prefill.title || '',
        releaseYear: props.prefill.releaseYear ?? null,
        albumArtUrl: props.prefill.albumArtUrl || '',
        country: props.prefill.country || '',
        discoveryLink: '',
        streamingLinkInput: Object.values(links)[0] || '',
        streamingLinks: { ...links },
        musicbrainzId: props.prefill.musicbrainzId || '',
        genres: [...(props.prefill.genres || [])]
      }
    } else {
      isEdit.value = false
      form.value = defaultForm()
    }
  }
})

function buildStreamingLinks(): Record<string, string> | undefined {
  const result: Record<string, string> = { ...form.value.streamingLinks }
  const input = form.value.streamingLinkInput.trim()
  if (input && !Object.values(result).includes(input)) {
    const service = input.includes('spotify.com') ? 'spotify'
      : input.includes('tidal.com') ? 'tidal'
      : input.includes('youtube.com') || input.includes('youtu.be') ? 'youtube'
      : 'other'
    result[service] = input
  }
  return Object.keys(result).length > 0 ? result : undefined
}

async function handleSubmit() {
  if (!form.value.artist || !form.value.title) return
  saving.value = true
  try {
    emit('submit', {
      artist: form.value.artist,
      title: form.value.title,
      releaseYear: form.value.releaseYear || undefined,
      albumArtUrl: form.value.albumArtUrl || undefined,
      country: form.value.country || undefined,
      discoveryLink: form.value.discoveryLink || undefined,
      streamingLinks: buildStreamingLinks(),
      musicbrainzId: form.value.musicbrainzId || undefined,
      genres: form.value.genres
    })
    visible.value = false
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.release-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-row {
  display: flex;
  gap: 1rem;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  flex: 1;
}

.form-field label {
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
  font-weight: 500;
}

.album-art-preview {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding-top: 0.5rem;
}
</style>
