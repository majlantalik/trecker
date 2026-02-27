<template>
  <nav class="app-sidebar">
    <div class="sidebar-brand">
      <span class="brand-icon">🎵</span>
      <span class="brand-name">Trecker</span>
    </div>

    <ul class="sidebar-nav">
      <li>
        <RouterLink to="/queue" class="nav-link" active-class="active">
          <i class="pi pi-list" />
          <span>Queue</span>
          <Badge v-if="queueCount > 0" :value="queueCount" severity="secondary" />
        </RouterLink>
      </li>
      <li>
        <RouterLink to="/library" class="nav-link" active-class="active">
          <i class="pi pi-book" />
          <span>Library</span>
        </RouterLink>
      </li>
      <li>
        <RouterLink to="/stats" class="nav-link" active-class="active">
          <i class="pi pi-chart-bar" />
          <span>Stats</span>
        </RouterLink>
      </li>
    </ul>

    <div class="sidebar-user">
      <span class="user-email">{{ authStore.user?.email }}</span>
      <Button icon="pi pi-sign-out" text @click="handleLogout" v-tooltip.right="'Sign out'" />
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import Badge from 'primevue/badge'
import Button from 'primevue/button'
import { useReleasesStore } from '@/stores/releases'
import { useAuthStore } from '@/stores/auth'

const releasesStore = useReleasesStore()
const authStore = useAuthStore()
const router = useRouter()

const queueCount = computed(() => releasesStore.total)

onMounted(() => {
  releasesStore.fetchReleases({ status: 'QUEUED', size: 1 })
})

async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.app-sidebar {
  width: 220px;
  min-height: 100vh;
  background: var(--p-surface-900);
  border-right: 1px solid var(--p-surface-700);
  display: flex;
  flex-direction: column;
  padding: 1rem 0;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1.25rem 1.5rem;
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--p-text-color);
}

.brand-icon {
  font-size: 1.5rem;
}

.sidebar-nav {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1.25rem;
  color: var(--p-text-muted-color);
  text-decoration: none;
  border-radius: 0;
  transition: all 0.15s;
  font-size: 0.95rem;
}

.nav-link:hover {
  background: var(--p-surface-800);
  color: var(--p-text-color);
}

.nav-link.active {
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color);
}

.nav-link i {
  width: 1rem;
}

.sidebar-user {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem 0.75rem 1.25rem;
  border-top: 1px solid var(--p-surface-700);
}

.user-email {
  font-size: 0.8rem;
  color: var(--p-text-muted-color);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}
</style>
