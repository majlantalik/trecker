<template>
  <div class="queue-view">
    <div class="view-header">
      <h1>Queue <span class="count" v-if="!loading">({{ total }})</span></h1>
      <div class="header-actions" v-if="total > 0">
        <Button
          :label="QUEUE_SORTS[sort].label"
          :icon="QUEUE_SORTS[sort].icon"
          severity="secondary"
          text
          :title="`Sorted by date added, ${QUEUE_SORTS[sort].label.toLowerCase()}. Click to reverse.`"
          :disabled="loading || savingSort"
          @click="toggleSort"
        />
        <Button
          label="Pick one for me"
          icon="pi pi-shuffle"
          severity="secondary"
          outlined
          :loading="pickingRandom"
          @click="pickRandom"
        />
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <ProgressSpinner />
    </div>

    <div v-else-if="releases.length === 0" class="empty-state">
      <i class="pi pi-inbox empty-icon" />
      <p>Your queue is empty. Paste a link above to add releases.</p>
    </div>

    <TransitionGroup v-else name="list" tag="div" class="queue-list">
      <div v-for="release in releases" :key="release.id" class="queue-item">
        <ReleaseCard
          :release="release"
          show-added
          @click="openRelease"
        >
          <template #actions>
            <QueueActions
              :release="release"
              @log="openLogModal"
              @delete="confirmDelete"
            />
          </template>
        </ReleaseCard>
      </div>
    </TransitionGroup>

    <div class="queue-pagination" v-if="!loading && totalPages > 1">
      <Paginator
        :rows="pageSize"
        :totalRecords="total"
        :first="currentPage * pageSize"
        @page="onPage"
      />
    </div>

    <QuickLogModal
      v-model:visible="showLogModal"
      :release="selectedRelease"
      @logged="onLogged"
    />

    <ConfirmDialog :pt="{ root: { class: 'tk-dialog' } }" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Button from 'primevue/button'
import ProgressSpinner from 'primevue/progressspinner'
import ConfirmDialog from 'primevue/confirmdialog'
import Paginator from 'primevue/paginator'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import ReleaseCard from '@/components/release/ReleaseCard.vue'
import QueueActions from '@/components/queue/QueueActions.vue'
import QuickLogModal from '@/components/release/QuickLogModal.vue'
import { QUEUE_SORTS, queueParams, useQueueSort } from '@/composables/useQueueSort'
import type { Release } from '@/types'

const router = useRouter()
const confirm = useConfirm()
const toast = useToast()
const releasesStore = useReleasesStore()

const releases = computed(() => releasesStore.releases)
const total = computed(() => releasesStore.total)
const loading = computed(() => releasesStore.loading)
const totalPages = computed(() => releasesStore.totalPages)
const currentPage = computed(() => releasesStore.currentPage)
const pageSize = computed(() => releasesStore.pageSize)

const { sort, saving: savingSort, load: loadSort, toggle } = useQueueSort()

const showLogModal = ref(false)
const selectedRelease = ref<Release | null>(null)
const pickingRandom = ref(false)

onMounted(async () => {
  // Read the saved order first, so the list does not load one way and then flip.
  await loadSort()
  releasesStore.setFilters(queueParams(sort.value))
  releasesStore.fetchReleases()
})

async function toggleSort() {
  const failed = await toggle()
  if (failed) {
    toast.add({ severity: 'error', summary: 'Could not save the order', detail: failed, life: 4000 })
    return
  }
  releasesStore.setFilters(queueParams(sort.value))
  releasesStore.fetchReleases()
}

function onPage(event: { page: number }) {
  releasesStore.setPage(event.page)
  releasesStore.fetchReleases()
}

// Logging or removing the last album on a later page would leave that page empty.
async function refetchKeepingPage() {
  await releasesStore.fetchReleases()
  if (releasesStore.releases.length === 0 && releasesStore.currentPage > 0) {
    releasesStore.setPage(releasesStore.currentPage - 1)
    await releasesStore.fetchReleases()
  }
}

function openRelease(release: Release) {
  router.push({ name: 'entry', params: { id: release.id } })
}

function openLogModal(release: Release) {
  selectedRelease.value = release
  showLogModal.value = true
}

function onLogged() {
  refetchKeepingPage()
}

function confirmDelete(release: Release) {
  confirm.require({
    message: `Remove "${release.artist} – ${release.title}" from queue?`,
    header: 'Remove from Queue',
    icon: 'pi pi-trash',
    rejectProps: { label: 'Cancel', severity: 'secondary', outlined: true },
    acceptProps: { label: 'Remove', severity: 'danger' },
    accept: async () => {
      try {
        await releasesStore.deleteRelease(release.id)
        // The next page's first album moves up into the gap.
        if (releasesStore.totalPages > 1) refetchKeepingPage()
        toast.add({ severity: 'info', summary: 'Removed', life: 2000 })
      } catch (e: any) {
        toast.add({ severity: 'error', summary: 'Failed', detail: e.message, life: 3000 })
      }
    }
  })
}

async function pickRandom() {
  pickingRandom.value = true
  try {
    const release = await releasesApi.getRandom()
    selectedRelease.value = release
    showLogModal.value = true
  } catch (e: any) {
    toast.add({ severity: 'warn', summary: 'No queued releases found', life: 2000 })
  } finally {
    pickingRandom.value = false
  }
}
</script>

<style scoped>
.queue-view {
  flex: 1;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.75rem;
}

.view-header h1 {
  font-size: 1.75rem;
  font-weight: 700;
}

.count {
  color: var(--tk-text-muted);
  font-weight: 400;
  font-family: var(--tk-font-body);
  letter-spacing: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.queue-pagination {
  display: flex;
  justify-content: center;
  margin-top: 1.25rem;
}

.loading-state {
  display: flex;
  justify-content: center;
  padding: 3rem;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--tk-text-muted);
}

.empty-icon {
  font-size: 3rem;
  display: block;
  margin-bottom: 1rem;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
