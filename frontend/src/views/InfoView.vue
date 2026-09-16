<template>
  <div class="info-view">
    <div class="info-header">
      <div class="info-avatar"><i class="pi pi-info-circle" /></div>
      <div class="info-header-text">
        <h1 class="info-title">Info</h1>
        <p class="info-subtitle">This library lives on this machine.</p>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-list section-icon" />
        <h2 class="section-title">Library</h2>
      </div>
      <div class="section-body">
        <p class="stat-row">
          <span class="stat-label">Releases</span>
          <span class="stat-value">{{ counts.total }}</span>
        </p>
        <p class="stat-row">
          <span class="stat-label">In queue</span>
          <span class="stat-value">{{ counts.queued }}</span>
        </p>
        <p class="stat-row">
          <span class="stat-label">Listened</span>
          <span class="stat-value">{{ counts.listened }}</span>
        </p>
        <p class="form-hint">
          Counted from the database below. Deleting a release stops tracking it but keeps
          the catalog entry, so these can be lower than the catalog row count.
        </p>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-database section-icon" />
        <h2 class="section-title">Database</h2>
      </div>
      <div class="section-body">
        <p v-if="dbError" class="db-error">{{ dbError }}</p>
        <template v-else-if="db">
          <p class="stat-row">
            <span class="stat-label">Location</span>
            <span class="stat-value stat-path" :title="db.path"><bdi dir="ltr">{{ db.path }}</bdi></span>
          </p>
          <p class="stat-row">
            <span class="stat-label">Size on disk</span>
            <span class="stat-value">{{ formatBytes(db.sizeBytes) }}</span>
          </p>
          <p class="stat-row">
            <span class="stat-label">Schema version</span>
            <span class="stat-value">{{ db.schemaVersion }}</span>
          </p>
          <p class="stat-row">
            <span class="stat-label">Catalog rows</span>
            <span class="stat-value">{{ db.releaseCount }}</span>
          </p>
          <p class="stat-row">
            <span class="stat-label">Tracking rows</span>
            <span class="stat-value">{{ db.trackedCount }}</span>
          </p>
          <p class="stat-row">
            <span class="stat-label">Journal mode</span>
            <Tag :value="db.journalMode.toUpperCase()" severity="success" />
          </p>
          <p class="stat-row">
            <span class="stat-label">Foreign keys</span>
            <Tag
              :value="db.foreignKeys ? 'On' : 'Off'"
              :severity="db.foreignKeys ? 'success' : 'danger'"
            />
          </p>
          <p class="stat-row">
            <span class="stat-label">Full-text search</span>
            <Tag
              :value="db.fts5 ? 'FTS5' : 'Unavailable'"
              :severity="db.fts5 ? 'success' : 'danger'"
            />
          </p>
        </template>
        <p v-else class="form-hint">Reading database state…</p>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-images section-icon" />
        <h2 class="section-title">Cache</h2>
      </div>
      <div class="section-body">
        <p class="stat-row">
          <span class="stat-label">
            Album covers
            <span class="provider-note">{{ caches.info.value ? describeCovers(caches.info.value) : '…' }}</span>
          </span>
          <Button label="Clear" icon="pi pi-trash" size="small" outlined
                  :disabled="caches.busy.value || !caches.info.value?.count"
                  @click="caches.clearCovers()" />
        </p>
        <p class="form-hint">
          Each cover is downloaded the first time an album is shown and kept on this machine,
          so your library looks the same offline. Clearing frees the space, and covers
          download again as albums are shown.
        </p>

        <p class="stat-row transfer-import">
          <span class="stat-label">Web view data</span>
          <Button label="Clear" icon="pi pi-trash" size="small" outlined
                  :disabled="caches.busy.value" @click="caches.clearWebview()" />
        </p>
        <p class="form-hint">
          Temporary files kept by the browser engine the app runs in. None of your library is
          stored there, so clearing it loses nothing.
        </p>

        <p v-if="caches.error.value" class="db-error">{{ caches.error.value }}</p>
        <p v-else-if="caches.message.value" class="transfer-result">
          <i class="pi pi-check-circle" />
          {{ caches.message.value }}
        </p>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-search section-icon" />
        <h2 class="section-title">Metadata providers</h2>
      </div>
      <div class="section-body">
        <div v-for="p in providers" :key="p.name" class="stat-row">
          <span class="stat-label">
            {{ p.name }}
            <span class="provider-note">{{ p.note }}</span>
          </span>
          <Tag :value="p.state" :severity="p.severity" />
        </div>
        <p class="form-hint">
          MusicBrainz and the Cover Art Archive need no account and are always on. The
          streaming services require a developer account and a paid subscription to query,
          so Trecker saves their links but does not look anything up through them.
        </p>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-link section-icon" />
        <h2 class="section-title">Match to MusicBrainz</h2>
      </div>
      <div class="section-body">
        <p class="stat-row">
          <span class="stat-label">Albums not linked</span>
          <span class="stat-value">{{ match.unlinkedCount ?? '–' }}</span>
        </p>

        <div class="transfer-row">
          <span class="stat-label">
            {{ match.running ? `Matching ${match.processed} of ${match.total}` : 'Find each one on MusicBrainz' }}
          </span>
          <span class="transfer-buttons">
            <Button v-if="!match.running" label="Match albums" icon="pi pi-search" size="small" outlined
                    :disabled="!match.unlinkedCount" @click="match.run()" />
            <Button v-else :label="match.stopping ? 'Stopping' : 'Stop'" icon="pi pi-stop" size="small" outlined
                    severity="secondary" :disabled="match.stopping" @click="match.stop()" />
          </span>
        </div>
        <progress v-if="match.running" class="match-progress" aria-label="Matching albums"
                  :max="match.total || 1" :value="match.processed" />
        <p class="form-hint">
          An album whose artist, title and year match exactly one album on MusicBrainz is linked
          straight away; the rest wait for you to choose. Linking replaces the artist, title,
          year, cover and country, and adds MusicBrainz's genres to yours. MusicBrainz allows one
          request a second, so this takes two to three seconds an album. You can leave this page
          while it runs.
        </p>

        <p v-if="match.error" class="db-error">{{ match.error }}</p>
        <p v-if="match.running || match.finished" class="transfer-result">
          <i class="pi pi-check-circle" />
          {{ matchSummary }}
        </p>

        <div v-if="match.toReview.length" class="transfer-row">
          <span class="stat-label">
            {{ match.toReview.length === 1 ? '1 album needs' : `${match.toReview.length} albums need` }} your choice
          </span>
          <Button label="Review" icon="pi pi-list-check" size="small" @click="reviewing = true" />
        </div>

        <div v-if="match.duplicates.length" class="rejected">
          <p class="rejected-title">Already in your library</p>
          <p v-for="d in match.duplicates" :key="d.release.id" class="rejected-row">
            <span class="rejected-name">{{ d.release.artist }} &mdash; {{ d.release.title }}</span>
            <RouterLink v-if="d.existingId" :to="`/entry/${d.existingId}`" class="match-open">Open the other one</RouterLink>
          </p>
        </div>
        <div v-if="match.noMatches.length" class="rejected">
          <p class="rejected-title">Nothing found on MusicBrainz</p>
          <p v-for="r in match.noMatches" :key="r.id" class="rejected-row">
            <RouterLink :to="`/entry/${r.id}`" class="rejected-name">{{ r.artist }} &mdash; {{ r.title }}</RouterLink>
          </p>
        </div>
        <div v-if="match.failed.length" class="rejected">
          <p class="rejected-title">Could not be linked</p>
          <p v-for="f in match.failed" :key="f.release.id" class="rejected-row">
            <span class="rejected-name">{{ f.release.artist }} &mdash; {{ f.release.title }}</span>
            <span class="rejected-reason">{{ f.reason }}</span>
          </p>
        </div>
      </div>
    </div>

    <AlbumPicker
      v-model:visible="reviewVisible"
      :candidates="match.current?.candidates ?? []"
      :query="match.current ? `${match.current.release.artist} - ${match.current.release.title}` : ''"
      :choosing-id="match.choosingId"
      :remaining="match.toReview.length - 1"
      @choose="match.choose"
      @none="match.decline()"
    />

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-download section-icon" />
        <h2 class="section-title">Backup</h2>
      </div>
      <div class="section-body">
        <div class="transfer-row">
          <span class="stat-label">Export your library</span>
          <span class="transfer-buttons">
            <Button label="JSON" icon="pi pi-file" size="small" outlined
                    :disabled="transfer.busy.value" @click="transfer.exportLibrary('json')" />
            <Button label="CSV" icon="pi pi-table" size="small" outlined
                    :disabled="transfer.busy.value" @click="transfer.exportLibrary('csv')" />
          </span>
        </div>
        <p class="form-hint">
          JSON is the complete record and the one to keep. CSV holds the same releases in a
          shape a spreadsheet can read. Neither carries the album artwork itself, only the
          address it lives at.
        </p>

        <div class="transfer-row transfer-import">
          <span class="stat-label">Import a file</span>
          <span class="transfer-buttons">
            <Select v-model="mode" :options="MODES" aria-label="Import mode" option-label="label" option-value="value"
                    size="small" class="transfer-mode" />
            <Button label="Choose file" icon="pi pi-upload" size="small" outlined
                    :disabled="transfer.busy.value" @click="transfer.importLibrary(mode)" />
          </span>
        </div>
        <p class="form-hint">
          Releases are matched by their MusicBrainz id, then by artist and title. Importing
          the same file twice changes nothing the second time.
        </p>

        <p v-if="transfer.error.value" class="db-error">{{ transfer.error.value }}</p>
        <p v-else-if="transfer.message.value" class="transfer-result">
          <i class="pi pi-check-circle" />
          {{ transfer.message.value }}
        </p>

        <div v-if="transfer.report.value?.rejected.length" class="rejected">
          <p class="rejected-title">Rows that could not be imported</p>
          <p v-for="row in transfer.report.value.rejected" :key="row.row" class="rejected-row">
            <span class="rejected-name">{{ row.artist }} &mdash; {{ row.title || 'untitled' }}</span>
            <span class="rejected-reason">{{ row.reason }}</span>
          </p>
        </div>
      </div>
    </div>

    <div class="info-section">
      <div class="section-header">
        <i class="pi pi-info-circle section-icon" />
        <h2 class="section-title">About</h2>
      </div>
      <div class="section-body">
        <p class="stat-row">
          <span class="stat-label">Version</span>
          <span class="stat-value">{{ version }}</span>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import Tag from 'primevue/tag'
import Button from 'primevue/button'
import Select from 'primevue/select'
import { releasesApi } from '@/api/releases'
import { infoApi } from '@/api/info'
import { useLibraryTransfer } from '@/composables/useLibraryTransfer'
import { useAlbumMatchStore } from '@/stores/albumMatch'
import AlbumPicker from '@/components/release/AlbumPicker.vue'
import { useCaches, describeCovers } from '@/composables/useCaches'
import { formatBytes } from '@/utils/bytes'
import type { DbInfo, ImportMode } from '@/types'

const transfer = useLibraryTransfer()
const match = useAlbumMatchStore()

// Closing the review with Esc or the close button pauses it; the albums stay in the list.
const reviewing = ref(false)
const reviewVisible = computed({
  get: () => reviewing.value && match.current !== null,
  set: (open: boolean) => { reviewing.value = open }
})
const matchSummary = computed(() => {
  const parts = [`${match.linked} linked`]
  if (match.toReview.length) parts.push(`${match.toReview.length} to review`)
  if (match.declined) parts.push(`${match.declined} left as they were`)
  if (match.noMatches.length) parts.push(`${match.noMatches.length} not found`)
  if (match.duplicates.length) parts.push(`${match.duplicates.length} already in your library`)
  if (match.failed.length) parts.push(`${match.failed.length} failed`)
  return parts.join(', ')
})
const caches = useCaches()

// Named for what happens to a release already in your library, because that is the only
// thing the choice changes. There is no merge: see docs/export-format.md.
const MODES = [
  { label: 'Keep mine', value: 'skip' },
  { label: 'Replace mine', value: 'overwrite' }
]
const mode = ref<ImportMode>('skip')

const version = '0.1.0'

const counts = ref({ total: 0, queued: 0, listened: 0 })
const db = ref<DbInfo | null>(null)
const dbError = ref('')


// Declared here rather than fetched: nothing is configurable, and a toggle that does
// nothing would be worse than an honest label.
const providers = [
  { name: 'MusicBrainz', note: 'artist, album, year, country, genres', state: 'Built in', severity: 'success' },
  { name: 'Cover Art Archive', note: 'album artwork', state: 'Built in', severity: 'success' },
  { name: 'Spotify', note: 'link saved, no lookup', state: 'Link only', severity: 'secondary' },
  { name: 'Tidal', note: 'link saved, no lookup', state: 'Link only', severity: 'secondary' },
  { name: 'YouTube', note: 'link saved, no lookup', state: 'Link only', severity: 'secondary' },
  { name: 'Bandcamp', note: 'link saved, title read from the URL', state: 'Link only', severity: 'secondary' },
  { name: 'Apple Music', note: 'link saved, title read from the URL', state: 'Link only', severity: 'secondary' }
] as const

onMounted(async () => {
  caches.load()
  if (!match.running) match.countUnlinked()

  const [all, queued, listened] = await Promise.all([
    releasesApi.getAll({ size: 1 }),
    releasesApi.getAll({ size: 1, status: 'QUEUED' }),
    releasesApi.getAll({ size: 1, status: 'LISTENED' })
  ])
  counts.value = {
    total: all.totalElements,
    queued: queued.totalElements,
    listened: listened.totalElements
  }

  try {
    db.value = await infoApi.dbInfo()
  } catch (e: any) {
    dbError.value = e?.message ?? String(e)
  }
})
</script>

<style scoped>
.info-view {
  max-width: 720px;
  margin: 0 auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.info-header {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--tk-border);
}

.info-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.25), rgba(0, 229, 176, 0.08));
  border: 1px solid rgba(0, 229, 176, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
  color: var(--tk-accent);
  flex-shrink: 0;
}

.info-header-text {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.info-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin: 0;
  color: var(--tk-text);
}

.info-subtitle {
  font-size: 0.875rem;
  color: rgba(226, 228, 240, 0.55);
  margin: 0;
}

.info-section {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.section-icon {
  font-size: 0.875rem;
  color: var(--tk-accent);
}

.section-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0;
}

.section-body {
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.stat-row {
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--tk-text);
}

.provider-note {
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.6);
  margin-left: 0.5rem;
}

.stat-value {
  font-size: 0.875rem;
  color: var(--tk-text);
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.form-hint {
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.6);
  margin: 0.25rem 0 0;
}

.stat-path {
  font-size: 0.78rem;
  font-weight: 400;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* Right to left so a long path loses its start, not its file name. The <bdi> inside keeps
     the path itself left to right, or its leading slash is drawn at the end. */
  direction: rtl;
  text-align: right;
  min-width: 0;
}

.transfer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}

.transfer-import {
  margin-top: 0.5rem;
  padding-top: 1rem;
  border-top: 1px solid var(--tk-border);
}

.transfer-buttons {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.transfer-mode {
  min-width: 9rem;
}

.transfer-result {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.5rem 0 0;
  font-size: 0.85rem;
  color: var(--tk-accent);
}

.rejected {
  margin-top: 0.75rem;
  padding: 0.75rem 0.9rem;
  border-radius: 10px;
  background: rgba(244, 63, 94, 0.08);
  border: 1px solid rgba(244, 63, 94, 0.2);
}

.rejected-title {
  margin: 0 0 0.5rem;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: #f43f5e;
}

.rejected-row {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  margin: 0.2rem 0;
  font-size: 0.8rem;
}

.rejected-name {
  color: var(--tk-text);
}

.rejected-reason {
  color: rgba(226, 228, 240, 0.55);
  text-align: right;
}

/* A native <progress>, restyled: the track through the element and its WebKit bar, the fill
   through the value pseudo-elements, which WebKitGTK and Firefox name differently. */
.match-progress {
  appearance: none;
  display: block;
  width: 100%;
  height: 6px;
  border: 0;
  border-radius: 999px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.06);
}

.match-progress::-webkit-progress-bar {
  background: rgba(255, 255, 255, 0.06);
}

.match-progress::-webkit-progress-value {
  background: var(--tk-accent);
  transition: width 0.3s ease;
}

.match-progress::-moz-progress-bar {
  background: var(--tk-accent);
}

.match-open {
  color: var(--tk-accent);
  font-size: 0.8rem;
  white-space: nowrap;
}

.rejected-row a.rejected-name {
  text-decoration: none;
}

.rejected-row a.rejected-name:hover {
  color: var(--tk-accent);
}

.db-error {
  color: #f43f5e;
  font-size: 0.85rem;
  margin: 0;
}
</style>
