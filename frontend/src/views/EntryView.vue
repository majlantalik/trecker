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
        <Button
          icon="pi pi-refresh"
          severity="secondary"
          text
          :loading="refreshing"
          v-tooltip.bottom="'Refresh metadata from MusicBrainz'"
          aria-label="Refresh metadata"
          @click="refreshMetadata"
        />
        <Button icon="pi pi-trash" severity="danger" text @click="confirmDelete" />
      </div>
    </div>

    <!-- Hero card -->
    <div class="hero-card">
      <div class="hero-inner">
        <!-- Album art -->
        <div class="hero-art">
          <div class="art-wrapper" @click="startEdit('albumArtUrl', release.albumArtUrl ?? '')">
            <img v-if="release.albumArtUrl" :src="release.albumArtUrl" :alt="`${release.artist} – ${release.title}`" />
            <div v-else class="art-placeholder"><i class="pi pi-music" /></div>
            <div class="art-overlay"><i class="pi pi-pencil" /></div>
          </div>
          <div v-if="editingField === 'albumArtUrl'" class="art-url-edit">
            <InputText
              v-focus
              v-model="draft"
              placeholder="Paste image URL..."
              fluid
              @blur="saveField('albumArtUrl', draft)"
              @keydown.enter="saveField('albumArtUrl', draft)"
              @keydown.escape="cancelEdit"
            />
          </div>
        </div>

        <!-- Info -->
        <div class="hero-info">
          <!-- Title -->
          <h1
            v-if="editingField !== 'title'"
            class="entry-title editable-field"
            @click="startEdit('title', release.title)"
          >
            {{ release.title }}<i class="pi pi-pencil edit-hint" />
          </h1>
          <InputText
            v-else
            v-focus
            v-model="draft"
            class="title-input"
            @blur="saveField('title', draft)"
            @keydown.enter="saveField('title', draft)"
            @keydown.escape="cancelEdit"
          />

          <!-- Artist -->
          <h2
            v-if="editingField !== 'artist'"
            class="entry-artist editable-field"
            @click="startEdit('artist', release.artist)"
          >
            {{ release.artist }}<i class="pi pi-pencil edit-hint" />
          </h2>
          <InputText
            v-else
            v-focus
            v-model="draft"
            class="artist-input"
            @blur="saveField('artist', draft)"
            @keydown.enter="saveField('artist', draft)"
            @keydown.escape="cancelEdit"
          />

          <!-- Rating -->
          <div class="rating-row">
            <HalfStarRating :modelValue="release.rating ?? null" :cancel="true" @update:modelValue="updateRating" />
          </div>

          <!-- Meta bar -->
          <div class="meta-bar">
            <!-- Status toggle -->
            <Tag
              :value="release.status"
              :severity="release.status === 'LISTENED' ? 'success' : 'warning'"
              class="status-badge"
              @click="saveField('status', release.status === 'LISTENED' ? 'QUEUED' : 'LISTENED')"
            />

            <!-- Year -->
            <span
              v-if="editingField !== 'releaseYear'"
              class="meta-pill editable"
              @click="startEdit('releaseYear', release.releaseYear)"
            >
              {{ release.releaseYear ?? 'add year' }}
            </span>
            <InputNumber
              v-else
              v-focus
              v-model="draft"
              :useGrouping="false"
              inputStyle="width: 70px"
              @blur="saveField('releaseYear', draft)"
              @keydown.enter="saveField('releaseYear', draft)"
              @keydown.escape="cancelEdit"
            />

            <!-- Country -->
            <span
              v-if="editingField !== 'country'"
              class="meta-pill editable"
              @click="startEdit('country', release.country ?? '')"
            >
              <CountryLabel v-if="release.country" :value="release.country" />
              <template v-else>add country</template>
            </span>
            <InputText
              v-else
              v-focus
              v-model="draft"
              style="width: 90px; font-size: 0.85rem;"
              @blur="saveField('country', draft)"
              @keydown.enter="saveField('country', draft)"
              @keydown.escape="cancelEdit"
            />

            <!-- Date listened -->
            <template v-if="release.status === 'LISTENED'">
              <span class="meta-pill editable" @click="openDatePicker($event)">
                {{ release.dateListened ? formatDate(release.dateListened) : 'add date' }}
              </span>
              <Popover ref="datePopoverRef">
                <DatePicker
                  v-model="dateDraft"
                  inline
                  @update:modelValue="(v) => { if (v instanceof Date) saveDateListened(v) }"
                />
              </Popover>
            </template>

            <!-- DNF toggle -->
            <button
              class="dnf-chip"
              :class="{ active: release.didNotFinish }"
              @click="saveField('didNotFinish', !release.didNotFinish)"
            >
              DNF
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Genres panel -->
    <div class="panel">
      <div class="panel-header">
        <i class="pi pi-tag panel-header-icon" />
        <h3 class="panel-title">Genres</h3>
      </div>
      <div class="panel-body">
        <GenreTagInput :modelValue="release.genres" @update:modelValue="updateGenres" />
      </div>
    </div>

    <!-- Notes panel -->
    <div class="panel">
      <div class="panel-header">
        <i class="pi pi-file-edit panel-header-icon" />
        <h3 class="panel-title">Notes</h3>
      </div>
      <div class="panel-body">
        <Textarea
          v-model="notesDraft"
          rows="4"
          placeholder="Your thoughts..."
          fluid
          @blur="saveNotes"
          class="notes-textarea"
        />
      </div>
    </div>

    <!-- Links panel -->
    <div class="panel">
      <div class="panel-header">
        <i class="pi pi-link panel-header-icon" />
        <h3 class="panel-title">Links</h3>
      </div>
      <div class="panel-body">
        <div class="links-list">
          <div v-for="(url, service) in release.streamingLinks" :key="service" class="link-chip">
            <a :href="url" target="_blank" class="link-chip-label">
              <i class="pi pi-external-link" /> {{ capitalize(String(service)) }}
            </a>
            <button class="link-chip-remove" @click="removeLink(String(service))">
              <i class="pi pi-times" />
            </button>
          </div>
          <a v-if="release.discoveryLink" :href="release.discoveryLink" target="_blank" class="link-chip discovery">
            <i class="pi pi-compass" /> Discovery
          </a>
          <button v-if="!addingLink" class="add-link-btn" @click="addingLink = true">
            <i class="pi pi-plus" /> add link
          </button>
          <div v-else class="add-link-row">
            <InputText
              v-focus
              v-model="newLinkUrl"
              placeholder="Paste Spotify / Tidal / YouTube URL..."
              @keydown.enter="addLink"
              @keydown.escape="addingLink = false"
            />
            <Button icon="pi pi-check" size="small" @click="addLink" />
            <Button icon="pi pi-times" size="small" severity="secondary" @click="addingLink = false" />
          </div>
        </div>
      </div>
    </div>

    <QuickLogModal v-model:visible="showLogModal" :release="release" @logged="onLogged" />
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
import { ref, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Textarea from 'primevue/textarea'
import DatePicker from 'primevue/datepicker'
import Popover from 'primevue/popover'
import ProgressSpinner from 'primevue/progressspinner'
import ConfirmDialog from 'primevue/confirmdialog'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import CountryLabel from '@/components/common/CountryLabel.vue'
import QuickLogModal from '@/components/release/QuickLogModal.vue'
import GenreTagInput from '@/components/release/GenreTagInput.vue'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { Release } from '@/types'

const props = defineProps<{ id: string }>()
const router = useRouter()
const confirm = useConfirm()
const toast = useToast()
const releasesStore = useReleasesStore()
const genresStore = useGenresStore()

const release = ref<Release | null>(null)
const loading = ref(true)
const showLogModal = ref(false)
const editingField = ref<string | null>(null)
const draft = ref<any>(null)
const savingField = ref<string | null>(null)
const notesDraft = ref('')
const addingLink = ref(false)
const newLinkUrl = ref('')
const datePopoverRef = ref<InstanceType<typeof Popover> | null>(null)
const dateDraft = ref<Date | null>(null)
const refreshing = ref(false)

// Auto-focus directive for inline edit inputs
const vFocus = {
  mounted(el: HTMLElement) {
    const input = (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')
      ? el
      : el.querySelector('input, textarea')
    ;(input as HTMLElement | null)?.focus()
  }
}

// Keep notesDraft in sync with release.notes
watch(() => release.value?.notes, (notes) => {
  notesDraft.value = notes ?? ''
})

onMounted(async () => {
  try {
    release.value = await releasesApi.getById(props.id)
  } catch {
    release.value = null
  } finally {
    loading.value = false
  }
})

// Metadata is fetched once when a release is added and never again, so improvements to
// the resolver only ever help new additions. This is the repair path for the rest.
// Catalog fields only: rating, notes, status and links are untouched.
async function refreshMetadata() {
  if (!release.value || refreshing.value) return
  refreshing.value = true
  try {
    const before = release.value
    const updated = await releasesApi.refreshMetadata(release.value.id)
    release.value = updated

    const changes = [
      before.albumArtUrl !== updated.albumArtUrl && 'artwork',
      before.releaseYear !== updated.releaseYear && 'year',
      before.country !== updated.country && 'country',
      before.genres.join() !== updated.genres.join() && 'genres',
      before.artist !== updated.artist && 'artist',
      before.title !== updated.title && 'title'
    ].filter(Boolean)

    toast.add({
      severity: changes.length ? 'success' : 'info',
      summary: changes.length ? 'Metadata updated' : 'Already up to date',
      detail: changes.length ? `Changed: ${changes.join(', ')}.` : 'Nothing new to fetch.',
      life: 4000
    })

    // fetchGenres() short-circuits once loaded, so feed new names in directly.
    updated.genres.forEach((g) => genresStore.addGenre(g))
  } catch (e: any) {
    toast.add({
      severity: 'error',
      summary: 'Refresh failed',
      detail: e?.message ?? 'Could not reach MusicBrainz.',
      life: 5000
    })
  } finally {
    refreshing.value = false
  }
}

function onLogged(updated: Release) {
  release.value = updated
}

function startEdit(field: string, value: any) {
  editingField.value = field
  draft.value = value
}

function cancelEdit() {
  editingField.value = null
  draft.value = null
}

async function saveField(field: string, value: any) {
  if (!release.value) return
  savingField.value = field
  try {
    release.value = await releasesStore.updateRelease(
      release.value.id,
      { [field]: value === '' ? undefined : value }
    )
    if (editingField.value === field) {
      editingField.value = null
    }
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Failed to save', detail: e.message, life: 3000 })
  } finally {
    savingField.value = null
  }
}

async function saveNotes() {
  if (!release.value || notesDraft.value === (release.value.notes ?? '')) return
  await saveField('notes', notesDraft.value)
}

function openDatePicker(event: Event) {
  dateDraft.value = release.value?.dateListened ? new Date(release.value.dateListened) : new Date()
  datePopoverRef.value?.toggle(event)
}

async function saveDateListened(v: Date) {
  datePopoverRef.value?.hide()
  await saveField('dateListened', v.toISOString())
}

async function updateRating(value: number | null) {
  if (!release.value) return
  try {
    release.value = await releasesStore.updateRelease(release.value.id, { rating: value ?? undefined })
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Failed to save rating', detail: e.message, life: 3000 })
  }
}

async function updateGenres(genres: string[]) {
  if (!release.value) return
  if (JSON.stringify(genres) === JSON.stringify(release.value.genres)) return
  release.value = await releasesStore.updateRelease(release.value.id, { genres })
  genres.forEach(g => genresStore.addGenre(g))
}

async function addLink() {
  if (!release.value || !newLinkUrl.value.trim()) return
  const url = newLinkUrl.value.trim()
  const service = url.includes('spotify.com') ? 'spotify'
    : url.includes('tidal.com') ? 'tidal'
    : url.includes('youtube.com') || url.includes('youtu.be') ? 'youtube'
    : 'other'
  const links = { ...(release.value.streamingLinks ?? {}), [service]: url }
  release.value = await releasesStore.updateRelease(release.value.id, { streamingLinks: links })
  newLinkUrl.value = ''
  addingLink.value = false
}

async function removeLink(service: string) {
  if (!release.value) return
  const links = { ...release.value.streamingLinks }
  delete links[service]
  release.value = await releasesStore.updateRelease(release.value.id, { streamingLinks: links })
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

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })
}

function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1)
}
</script>

<style scoped>
.entry-view {
  max-width: 820px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.entry-actions {
  display: flex;
  gap: 0.5rem;
}

/* ─── Hero card ─── */
.hero-card {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 16px;
  overflow: visible;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.hero-inner {
  display: flex;
  gap: 1.75rem;
  padding: 1.75rem;
}

/* ─── Art column ─── */
.hero-art {
  flex-shrink: 0;
  width: 260px;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.art-wrapper {
  position: relative;
  cursor: pointer;
  border-radius: 10px;
  overflow: hidden;
}

.art-wrapper img {
  width: 100%;
  display: block;
  border-radius: 10px;
}

.art-placeholder {
  width: 100%;
  aspect-ratio: 1;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 3rem;
  color: rgba(226, 228, 240, 0.2);
}

.art-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
  font-size: 1.4rem;
  color: var(--tk-text);
}

.art-wrapper:hover .art-overlay {
  opacity: 1;
}

.art-url-edit {
  margin-top: 0.1rem;
}

/* ─── Info column ─── */
.hero-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  min-width: 0;
}

.entry-title {
  font-size: 1.85rem;
  font-weight: 700;
  margin: 0;
  line-height: 1.2;
  color: var(--tk-text);
}

.entry-artist {
  font-size: 1.1rem;
  font-weight: 400;
  margin: 0;
  color: rgba(226, 228, 240, 0.55);
}

.editable-field {
  cursor: pointer;
  display: inline-flex;
  align-items: baseline;
  gap: 0.4rem;
}

.edit-hint {
  font-size: 0.6em;
  opacity: 0;
  transition: opacity 0.15s;
  color: var(--tk-accent);
  flex-shrink: 0;
}

.editable-field:hover .edit-hint {
  opacity: 1;
}

.title-input {
  font-size: 1.5rem;
  font-weight: 700;
  width: 100%;
}

.artist-input {
  font-size: 1rem;
  width: 100%;
}

.rating-row {
  display: flex;
  align-items: center;
}

/* ─── Meta bar ─── */
.meta-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.25rem;
}

.status-badge {
  cursor: pointer;
  transition: opacity 0.15s;
}

.status-badge:hover {
  opacity: 0.8;
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.6rem;
  border-radius: 6px;
  font-size: 0.82rem;
  color: var(--tk-text);
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--tk-border);
  transition: background 0.15s, border-color 0.15s;
}

.meta-pill.editable {
  cursor: pointer;
}

.meta-pill.editable:hover {
  background: rgba(0, 229, 176, 0.08);
  border-color: rgba(0, 229, 176, 0.3);
}


.dnf-chip {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.6rem;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  border: 1px solid var(--tk-border);
  background: transparent;
  color: rgba(226, 228, 240, 0.4);
  cursor: pointer;
  transition: all 0.15s;
}

.dnf-chip:hover {
  border-color: #f87171;
  color: #f87171;
}

.dnf-chip.active {
  background: rgba(248, 113, 113, 0.12);
  border-color: #f87171;
  color: #f87171;
}

/* ─── Panels ─── */
.panel {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.panel-header-icon {
  font-size: 0.875rem;
  color: var(--tk-accent);
}

.panel-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0;
}

.panel-body {
  padding: 1.25rem;
}

/* ─── Notes ─── */
.notes-textarea {
  width: 100%;
  resize: vertical;
}

/* ─── Links ─── */
.links-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
}

.link-chip {
  display: inline-flex;
  align-items: center;
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--tk-border);
  background: rgba(255, 255, 255, 0.04);
}

.link-chip-label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.7rem;
  color: var(--tk-text);
  text-decoration: none;
  font-size: 0.85rem;
  transition: background 0.15s;
}

.link-chip-label:hover {
  background: rgba(255, 255, 255, 0.06);
}

.link-chip-remove {
  padding: 0.3rem 0.5rem;
  border: none;
  background: transparent;
  color: rgba(226, 228, 240, 0.35);
  cursor: pointer;
  transition: color 0.15s, background 0.15s;
  border-left: 1px solid var(--tk-border);
  line-height: 1;
}

.link-chip-remove:hover {
  color: #f87171;
  background: rgba(248, 113, 113, 0.1);
}

.link-chip.discovery {
  padding: 0.3rem 0.7rem;
  color: var(--tk-text);
  text-decoration: none;
  font-size: 0.85rem;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  transition: background 0.15s;
}

.link-chip.discovery:hover {
  background: rgba(255, 255, 255, 0.06);
}

.add-link-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.7rem;
  border-radius: 6px;
  border: 1px dashed rgba(0, 229, 176, 0.3);
  background: transparent;
  color: var(--tk-accent);
  font-size: 0.82rem;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}

.add-link-btn:hover {
  background: rgba(0, 229, 176, 0.08);
  border-color: rgba(0, 229, 176, 0.5);
}

.add-link-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  flex-wrap: wrap;
  width: 100%;
}

.add-link-row :deep(.p-inputtext) {
  flex: 1;
  min-width: 200px;
}

/* ─── Loading / error states ─── */
.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  gap: 1rem;
}
</style>
