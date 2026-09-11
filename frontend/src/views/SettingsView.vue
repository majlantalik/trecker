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
        <i class="pi pi-database section-icon" />
        <h2 class="section-title">Library</h2>
      </div>
      <div class="section-body">
        <div class="stat-row">
          <span class="stat-label">Releases</span>
          <span class="stat-value">{{ counts.total }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">In queue</span>
          <span class="stat-value">{{ counts.queued }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Listened</span>
          <span class="stat-value">{{ counts.listened }}</span>
        </div>
        <p class="form-hint">
          Storage is in-memory until the local database lands, so changes are lost when the
          app closes.
        </p>
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
          MusicBrainz needs no account and is always on. The streaming services will each
          require connecting your own account.
        </p>
      </div>
    </div>

    <div class="settings-section">
      <div class="section-header">
        <i class="pi pi-info-circle section-icon" />
        <h2 class="section-title">About</h2>
      </div>
      <div class="section-body">
        <div class="stat-row">
          <span class="stat-label">Version</span>
          <span class="stat-value">{{ version }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Tag from 'primevue/tag'
import { releasesApi } from '@/api/releases'

const version = '0.1.0'

const counts = ref({ total: 0, queued: 0, listened: 0 })

// Providers are declared here rather than fetched: nothing is configurable until the
// OAuth work lands, and a toggle that does nothing is worse than an honest label.
const providers = [
  { name: 'MusicBrainz', note: 'no account needed', state: 'Built in', severity: 'success' },
  { name: 'Cover Art Archive', note: 'no account needed', state: 'Built in', severity: 'success' },
  { name: 'Spotify', note: 'your own account', state: 'Not yet', severity: 'secondary' },
  { name: 'Tidal', note: 'your own account', state: 'Not yet', severity: 'secondary' },
  { name: 'YouTube', note: 'your own account', state: 'Not yet', severity: 'secondary' }
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
</style>
