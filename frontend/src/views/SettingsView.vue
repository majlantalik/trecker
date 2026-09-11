<template>
  <div class="settings-view">
    <div class="settings-header">
      <div class="settings-avatar"><i class="pi pi-cog" /></div>
      <div class="settings-header-info">
        <h1 class="settings-title">Settings</h1>
        <p class="settings-subtitle">This library lives on this machine.</p>
      </div>
    </div>

    <div class="settings-section">
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
          Reads still come from memory. The next step moves them onto the database below.
        </p>
      </div>
    </div>

    <div class="settings-section">
      <div class="section-header">
        <i class="pi pi-database section-icon" />
        <h2 class="section-title">Database</h2>
      </div>
      <div class="section-body">
        <p v-if="dbError" class="db-error">{{ dbError }}</p>
        <template v-else-if="db">
          <p class="stat-row">
            <span class="stat-label">Location</span>
            <span class="stat-value stat-path" :title="db.path">{{ db.path }}</span>
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

    <div class="settings-section">
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

    <div class="settings-section">
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
import { ref, onMounted } from 'vue'
import Tag from 'primevue/tag'
import { releasesApi } from '@/api/releases'
import { settingsApi } from '@/api/settings'
import type { DbInfo } from '@/types'

const version = '0.1.0'

const counts = ref({ total: 0, queued: 0, listened: 0 })
const db = ref<DbInfo | null>(null)
const dbError = ref('')

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

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
    db.value = await settingsApi.dbInfo()
  } catch (e: any) {
    dbError.value = e?.message ?? String(e)
  }
})
</script>

<style scoped>
.settings-view {
  max-width: 720px;
  margin: 0 auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--tk-border);
}

.settings-avatar {
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

.settings-header-info {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.settings-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin: 0;
  color: var(--tk-text);
}

.settings-subtitle {
  font-size: 0.875rem;
  color: rgba(226, 228, 240, 0.55);
  margin: 0;
}

.settings-section {
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
  color: rgba(226, 228, 240, 0.45);
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
  color: rgba(226, 228, 240, 0.45);
  margin: 0.25rem 0 0;
}

.stat-path {
  font-size: 0.78rem;
  font-weight: 400;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: right;
  min-width: 0;
}

.db-error {
  color: #f43f5e;
  font-size: 0.85rem;
  margin: 0;
}
</style>
