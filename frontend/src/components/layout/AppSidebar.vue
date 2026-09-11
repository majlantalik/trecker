<template>
  <nav class="app-sidebar">
    <div class="sidebar-brand">
      <div class="brand-icon-wrap">
        <i class="pi pi-headphones brand-icon" />
      </div>
      <span class="brand-name">Trecker</span>
    </div>

    <div class="nav-section-label">Navigation</div>
    <ul class="sidebar-nav">
      <li>
        <RouterLink to="/queue" class="nav-link" active-class="active">
          <i class="pi pi-list nav-icon" />
          <span class="nav-label">Queue</span>
          <span v-if="queueCount > 0" class="nav-badge">{{ queueCount }}</span>
        </RouterLink>
      </li>
      <li>
        <RouterLink to="/library" class="nav-link" active-class="active">
          <i class="pi pi-book nav-icon" />
          <span class="nav-label">Library</span>
        </RouterLink>
      </li>
      <li>
        <RouterLink to="/stats" class="nav-link" active-class="active">
          <i class="pi pi-chart-bar nav-icon" />
          <span class="nav-label">Stats</span>
        </RouterLink>
      </li>
    </ul>

    <div class="sidebar-user">
      <RouterLink to="/settings" class="user-info" active-class="user-info--active">
        <div class="user-avatar"><i class="pi pi-cog" /></div>
        <div class="user-text">
          <span class="user-name">Settings</span>
          <span class="user-email">Local library</span>
        </div>
      </RouterLink>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useReleasesStore } from '@/stores/releases'

const releasesStore = useReleasesStore()

const queueCount = computed(() => releasesStore.total)

onMounted(() => {
  releasesStore.fetchReleases({ status: 'QUEUED', size: 1 })
})
</script>

<style scoped>
.app-sidebar {
  width: 260px;
  min-height: 100vh;
  background: linear-gradient(180deg, #0c0d1a 0%, #08090e 100%);
  border-right: 1px solid var(--tk-border);
  display: flex;
  flex-direction: column;
  padding: 0;
  position: relative;
  flex-shrink: 0;
}

/* Subtle accent line at the top */
.app-sidebar::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent 0%, var(--tk-accent) 50%, transparent 100%);
  opacity: 0.6;
}

/* ── Brand ── */
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1.75rem 1.5rem 1.5rem;
}

.brand-icon-wrap {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--tk-accent-dim);
  border: 1px solid rgba(0, 229, 176, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 0 14px var(--tk-accent-glow);
}

.brand-icon {
  font-size: 1rem;
  color: var(--tk-accent);
}

.brand-name {
  font-family: var(--tk-font-display);
  font-size: 1.35rem;
  font-weight: 800;
  background: linear-gradient(135deg, #ffffff 30%, var(--tk-accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: -0.02em;
}

/* ── Nav ── */
.nav-section-label {
  font-size: 0.65rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-text-muted);
  padding: 0 1.5rem;
  margin-bottom: 0.5rem;
  opacity: 0.6;
}

.sidebar-nav {
  list-style: none;
  padding: 0 0.875rem;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.75rem 1rem;
  color: var(--tk-text-muted);
  text-decoration: none;
  border-radius: 10px;
  transition: all 0.18s ease;
  font-size: 0.95rem;
  font-weight: 500;
  position: relative;
}

.nav-link:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--tk-text);
}

.nav-link.active {
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.18), rgba(0, 229, 176, 0.06));
  color: var(--tk-accent);
  box-shadow: inset 0 0 0 1px rgba(0, 229, 176, 0.2);
}

.nav-link.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 25%;
  bottom: 25%;
  width: 3px;
  background: var(--tk-accent);
  border-radius: 0 3px 3px 0;
  box-shadow: 0 0 8px var(--tk-accent-glow);
}

.nav-icon {
  font-size: 1rem;
  width: 1.125rem;
  text-align: center;
  flex-shrink: 0;
}

.nav-label {
  flex: 1;
}

.nav-badge {
  font-size: 0.65rem;
  font-weight: 700;
  background: var(--tk-accent);
  color: #000;
  border-radius: 999px;
  padding: 0.1em 0.45em;
  line-height: 1.5;
  min-width: 18px;
  text-align: center;
}

/* ── User ── */
.sidebar-user {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 0.875rem 1.25rem;
  border-top: 1px solid var(--tk-border);
  gap: 0.5rem;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  min-width: 0;
  flex: 1;
  text-decoration: none;
  border-radius: 8px;
  padding: 0.35rem 0.5rem;
  transition: background 0.15s ease;
}

.user-info:hover {
  background: rgba(255, 255, 255, 0.05);
}

.user-info--active {
  background: rgba(0, 229, 176, 0.08);
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.25), rgba(0, 229, 176, 0.08));
  border: 1px solid rgba(0, 229, 176, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--tk-accent);
  flex-shrink: 0;
  font-family: var(--tk-font-display);
}

.user-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-name {
  font-size: 0.8rem;
  color: var(--tk-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.user-email {
  font-size: 0.72rem;
  color: rgba(226, 228, 240, 0.45);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
