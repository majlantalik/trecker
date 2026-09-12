<template>
  <Dialog
    v-model:visible="visible"
    header="Log it"
    modal
    :style="{ width: '440px' }"
    :draggable="false"
  >
    <div v-if="release" class="quick-log-content">
      <div class="release-summary">
        <img v-if="release.albumArtUrl" :src="coverSrc(release.albumArtUrl)" class="summary-art" />
        <div>
          <div class="summary-title">{{ release.title }}</div>
          <div class="summary-artist">{{ release.artist }}</div>
        </div>
      </div>

      <Divider />

      <div class="log-field">
        <label>Rating</label>
        <HalfStarRating v-model="form.rating" :cancel="true" />
      </div>

      <div class="log-field">
        <label>Genres</label>
        <GenreTagInput v-model="form.genres" />
      </div>

      <div class="log-field">
        <label>Country</label>
        <CountrySelect v-model="form.country" fluid />
      </div>

      <div class="log-field">
        <label>Notes</label>
        <Textarea v-model="form.notes" rows="3" placeholder="Your thoughts..." fluid />
      </div>

      <div class="log-field-inline">
        <Checkbox v-model="form.didNotFinish" binary input-id="dnf" />
        <label for="dnf">Did not finish</label>
      </div>

      <Divider />

      <div class="log-actions">
        <Button label="Cancel" severity="secondary" outlined @click="visible = false" />
        <Button label="Log it" icon="pi pi-check" :loading="saving" @click="handleLog(false)" />
        <Button label="Log + Details" icon="pi pi-arrow-right" severity="secondary" :loading="saving" @click="handleLog(true)" />
      </div>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { coverSrc } from '@/api/cache'
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import Checkbox from 'primevue/checkbox'
import Textarea from 'primevue/textarea'
import Divider from 'primevue/divider'
import { useToast } from 'primevue/usetoast'
import CountrySelect from '@/components/common/CountrySelect.vue'
import GenreTagInput from './GenreTagInput.vue'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { Release } from '@/types'

const props = defineProps<{
  release: Release | null
}>()

const emit = defineEmits<{
  logged: [release: Release]
}>()

const visible = defineModel<boolean>('visible', { default: false })
const router = useRouter()
const toast = useToast()

const releasesStore = useReleasesStore()
const genresStore = useGenresStore()
const saving = ref(false)

const form = ref({
  rating: null as number | null,
  genres: [] as string[],
  country: '',
  notes: '',
  didNotFinish: false
})

watch(() => props.release, (r) => {
  if (r) {
    form.value = {
      rating: r.rating ?? null,
      genres: [...r.genres],
      country: r.country || '',
      notes: r.notes || '',
      didNotFinish: r.didNotFinish
    }
  }
})

async function handleLog(navigateToEntry: boolean) {
  if (!props.release) return
  saving.value = true
  try {
    const updated = await releasesStore.markAsListened(props.release.id, {
      rating: form.value.rating ?? undefined,
      genres: form.value.genres,
      country: form.value.country || undefined,
      notes: form.value.notes || undefined,
      didNotFinish: form.value.didNotFinish
    })

    form.value.genres.forEach(g => genresStore.addGenre(g))

    emit('logged', updated)
    visible.value = false

    toast.add({
      severity: 'success',
      summary: 'Logged!',
      detail: `${props.release.artist} – ${props.release.title}`,
      life: 3000
    })

    if (navigateToEntry) {
      router.push({ name: 'entry', params: { id: props.release.id } })
    }
  } catch (e: any) {
    toast.add({
      severity: 'error',
      summary: 'Failed to log',
      detail: e.message,
      life: 4000
    })
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.quick-log-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.release-summary {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.summary-art {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 4px;
}

.summary-title {
  font-weight: 600;
}

.summary-artist {
  color: var(--p-text-muted-color);
  font-size: 0.9rem;
}

.log-field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.log-field label {
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
  font-weight: 500;
}

.log-field-inline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.log-field-inline label {
  cursor: pointer;
}

.log-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
