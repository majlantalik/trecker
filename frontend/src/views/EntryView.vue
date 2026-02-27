<template>
  <div class="entry-view" v-if="release">
    <div class="entry-header">
      <Button icon="pi pi-arrow-left" label="Back" text @click="$router.back()" />
      <div class="entry-actions">
        <Button
          v-if="release.status === 'QUEUED'"
          icon="pi pi-headphones"
          label="Log"
          @click="showLogModal = true"
        />
        <Button icon="pi pi-pencil" label="Edit" severity="secondary" @click="showEditForm = true" />
        <Button icon="pi pi-trash" severity="danger" text @click="confirmDelete" />
      </div>
    </div>

    <div class="entry-content">
      <div class="entry-art">
        <img v-if="release.albumArtUrl" :src="release.albumArtUrl" :alt="`${release.artist} – ${release.title}`" />
        <div v-else class="art-placeholder"><i class="pi pi-music" /></div>
      </div>

      <div class="entry-meta">
        <div class="entry-status">
          <Tag :value="release.status" :severity="release.status === 'LISTENED' ? 'success' : 'warning'" />
          <Tag v-if="release.didNotFinish" value="DNF" severity="danger" />
        </div>

        <h1 class="entry-title">{{ release.title }}</h1>
        <h2 class="entry-artist">{{ release.artist }}</h2>

        <div class="meta-row" v-if="release.rating">
          <label>Rating</label>
          <Rating :modelValue="release.rating" :stars="5" readonly />
        </div>

        <div class="meta-grid">
          <div class="meta-item" v-if="release.releaseYear">
            <label>Year</label>
            <span>{{ release.releaseYear }}</span>
          </div>
          <div class="meta-item" v-if="release.country">
            <label>Country</label>
            <span>{{ release.country }}</span>
          </div>
          <div class="meta-item" v-if="release.dateListened">
            <label>Listened</label>
            <span>{{ formatDate(release.dateListened) }}</span>
          </div>
        </div>

        <div class="meta-row" v-if="release.genres.length">
          <label>Genres</label>
          <div class="genres-list">
            <Tag
              v-for="g in release.genres"
              :key="g"
              :value="g"
              severity="secondary"
              class="genre-tag"
              @click="goToGenre(g)"
            />
          </div>
        </div>

        <div class="meta-row" v-if="release.notes">
          <label>Notes</label>
          <p class="notes-text">{{ release.notes }}</p>
        </div>

        <div class="meta-row links" v-if="Object.keys(release.streamingLinks).length || release.discoveryLink">
          <a
            v-for="(url, service) in release.streamingLinks"
            :key="service"
            :href="url"
            target="_blank"
            class="link-btn"
          >
            <i class="pi pi-external-link" /> {{ capitalize(String(service)) }}
          </a>
          <a v-if="release.discoveryLink" :href="release.discoveryLink" target="_blank" class="link-btn">
            <i class="pi pi-external-link" /> Discovery
          </a>
        </div>
      </div>
    </div>

    <QuickLogModal
      v-model:visible="showLogModal"
      :release="release"
      @logged="onLogged"
    />

    <ReleaseForm
      v-model:visible="showEditForm"
      :release="release"
      @submit="handleEdit"
    />

    <ConfirmDialog />
  </div>

  <div v-else-if="loading" class="loading-state">
    <ProgressSpinner />
  </div>

  <div v-else class="error-state">
    <p>Release not found.</p>
    <Button label="Back to Library" @click="$router.push('/library')" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import Rating from 'primevue/rating'
import ProgressSpinner from 'primevue/progressspinner'
import ConfirmDialog from 'primevue/confirmdialog'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import QuickLogModal from '@/components/release/QuickLogModal.vue'
import ReleaseForm from '@/components/release/ReleaseForm.vue'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import type { Release } from '@/types'

const props = defineProps<{ id: string }>()
const router = useRouter()
const confirm = useConfirm()
const toast = useToast()
const releasesStore = useReleasesStore()

const release = ref<Release | null>(null)
const loading = ref(true)
const showLogModal = ref(false)
const showEditForm = ref(false)

onMounted(async () => {
  try {
    release.value = await releasesApi.getById(props.id)
  } catch {
    release.value = null
  } finally {
    loading.value = false
  }
})

function onLogged(updated: Release) {
  release.value = updated
}

async function handleEdit(data: any) {
  if (!release.value) return
  try {
    release.value = await releasesStore.updateRelease(release.value.id, data)
    toast.add({ severity: 'success', summary: 'Saved', life: 2000 })
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Failed', detail: e.message, life: 3000 })
  }
}

function confirmDelete() {
  if (!release.value) return
  confirm.require({
    message: `Delete "${release.value.artist} – ${release.value.title}"?`,
    header: 'Delete Release',
    icon: 'pi pi-trash',
    rejectProps: { label: 'Cancel', severity: 'secondary', outlined: true },
    acceptProps: { label: 'Delete', severity: 'danger' },
    accept: async () => {
      try {
        await releasesStore.deleteRelease(release.value!.id)
        router.push('/library')
      } catch (e: any) {
        toast.add({ severity: 'error', summary: 'Failed', detail: e.message, life: 3000 })
      }
    }
  })
}

function goToGenre(genre: string) {
  router.push({ name: 'library', query: { genre } })
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })
}

function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1)
}
</script>

<style scoped>
.entry-view {
  max-width: 900px;
}

.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.entry-actions {
  display: flex;
  gap: 0.5rem;
}

.entry-content {
  display: flex;
  gap: 2rem;
}

.entry-art {
  width: 240px;
  flex-shrink: 0;
}

.entry-art img {
  width: 100%;
  border-radius: 8px;
}

.art-placeholder {
  width: 100%;
  aspect-ratio: 1;
  background: var(--p-surface-800);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 3rem;
  color: var(--p-text-muted-color);
}

.entry-meta {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.entry-status {
  display: flex;
  gap: 0.5rem;
}

.entry-title {
  font-size: 2rem;
  font-weight: 700;
  margin: 0;
  line-height: 1.2;
}

.entry-artist {
  font-size: 1.25rem;
  font-weight: 400;
  color: var(--p-text-muted-color);
  margin: 0;
}

.meta-row {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.meta-row label {
  font-size: 0.8rem;
  color: var(--p-text-muted-color);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 1rem;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.meta-item label {
  font-size: 0.75rem;
  color: var(--p-text-muted-color);
  font-weight: 500;
  text-transform: uppercase;
}

.meta-item span {
  font-weight: 500;
}

.genres-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.genre-tag {
  cursor: pointer;
  transition: opacity 0.15s;
}

.genre-tag:hover {
  opacity: 0.75;
}

.notes-text {
  margin: 0;
  line-height: 1.6;
  white-space: pre-wrap;
}

.links {
  flex-direction: row;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.link-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.85rem;
  border-radius: 6px;
  background: var(--p-surface-800);
  color: var(--p-text-color);
  text-decoration: none;
  font-size: 0.9rem;
  transition: background 0.15s;
}

.link-btn:hover {
  background: var(--p-surface-700);
}

.loading-state, .error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  gap: 1rem;
}
</style>
