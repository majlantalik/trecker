<template>
  <div class="library-view">
    <div class="view-header">
      <h1>Library <span class="count" v-if="!loading">({{ total }})</span></h1>
      <ViewToggle v-model="viewMode" />
    </div>

    <div class="library-layout">
      <aside class="filters-panel">
        <LibraryFilters @update:filters="applyFilters" :initial-genre="initialGenre" initial-status="LISTENED" />
      </aside>

      <div class="library-content">
        <div v-if="loading" class="loading-state">
          <ProgressSpinner />
        </div>

        <div v-else-if="releases.length === 0" class="empty-state">
          <i class="pi pi-search empty-icon" />
          <p>No releases found matching your filters.</p>
        </div>

        <!-- Table view -->
        <DataTable
          v-else-if="viewMode === 'table'"
          :value="releases"
          lazy
          :paginator="totalPages > 1"
          :rows="pageSize"
          :totalRecords="total"
          :rowsPerPageOptions="[10, 20, 50]"
          @page="onPage"
          @sort="onSort"
          removableSort
          :sortField="sortField"
          :sortOrder="sortOrder"
          row-hover
          @rowClick="openEntry"
          class="library-table"
        >
          <Column field="albumArtUrl" header="" style="width: 60px">
            <template #body="{ data }">
              <img v-if="data.albumArtUrl" :src="coverSrc(data.albumArtUrl)" class="table-art" />
              <div v-else class="table-art-placeholder"><i class="pi pi-music" /></div>
            </template>
          </Column>
          <Column field="artist" header="Artist" sortable />
          <Column field="title" header="Title" sortable />
          <Column field="releaseYear" header="Year" sortable style="width: 80px" />
          <Column field="country" header="Country" style="width: 160px">
            <template #body="{ data }">
              <CountryLabel :value="data.country" />
            </template>
          </Column>
          <Column field="rating" header="Rating" sortable style="width: 130px">
            <template #body="{ data }">
              <HalfStarRating v-if="data.rating" :modelValue="data.rating" readonly />
            </template>
          </Column>
          <Column field="dateListened" header="Listened" sortable style="width: 130px">
            <template #body="{ data }">
              {{ data.dateListened ? formatDate(data.dateListened) : '' }}
            </template>
          </Column>
          <Column style="width: 80px">
            <template #body="{ data }">
              <div class="table-actions" @click.stop>
                <Button
                  icon="pi pi-pencil"
                  text
                  size="small"
                  @click="openEntry(data)"
                  aria-label="Edit"
                />
                <Button
                  icon="pi pi-trash"
                  text
                  severity="danger"
                  size="small"
                  @click="confirmDelete(data)"
                  aria-label="Delete"
                />
              </div>
            </template>
          </Column>
        </DataTable>

        <!-- Grid view -->
        <div v-else class="releases-grid">
          <ReleaseCard
            v-for="r in releases"
            :key="r.id"
            :release="r"
            @click="openEntry"
          />
          <div class="grid-pagination" v-if="totalPages > 1">
            <Paginator
              :rows="pageSize"
              :totalRecords="total"
              :first="currentPage * pageSize"
              @page="onPage"
            />
          </div>
        </div>
      </div>
    </div>

    <QuickLogModal
      v-model:visible="showLogModal"
      :release="selectedRelease"
      @logged="releasesStore.fetchReleases()"
    />

    <ConfirmDialog />
  </div>
</template>

<script setup lang="ts">
import { coverSrc } from '@/api/cache'
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Button from 'primevue/button'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import CountryLabel from '@/components/common/CountryLabel.vue'
import ProgressSpinner from 'primevue/progressspinner'
import ConfirmDialog from 'primevue/confirmdialog'
import Paginator from 'primevue/paginator'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { useReleasesStore } from '@/stores/releases'
import ViewToggle from '@/components/library/ViewToggle.vue'
import LibraryFilters from '@/components/library/LibraryFilters.vue'
import ReleaseCard from '@/components/release/ReleaseCard.vue'
import QuickLogModal from '@/components/release/QuickLogModal.vue'
import type { Release, ReleaseFilterParams } from '@/types'

const router = useRouter()
const route = useRoute()
const confirm = useConfirm()
const toast = useToast()
const releasesStore = useReleasesStore()

const releases = computed(() => releasesStore.releases)
const total = computed(() => releasesStore.total)
const totalPages = computed(() => releasesStore.totalPages)
const currentPage = computed(() => releasesStore.currentPage)
const pageSize = computed(() => releasesStore.pageSize)
const loading = computed(() => releasesStore.loading)

const viewMode = ref<'table' | 'grid'>('table')
const showLogModal = ref(false)
const selectedRelease = ref<Release | null>(null)
const sortField = ref('dateListened')
const sortOrder = ref(-1)

const initialGenre = route.query.genre as string | undefined

onMounted(() => {
  releasesStore.setFilters({
    status: 'LISTENED',
    sort: 'dateListened',
    direction: 'DESC',
    ...(initialGenre ? { genre: initialGenre } : {})
  })
  releasesStore.fetchReleases()
})

function applyFilters(filters: ReleaseFilterParams) {
  releasesStore.setFilters(filters)
  releasesStore.fetchReleases()
}

function onPage(event: any) {
  releasesStore.setPage(event.page)
  releasesStore.fetchReleases()
}

function onSort(event: any) {
  sortField.value = event.sortField
  sortOrder.value = event.sortOrder
  releasesStore.setFilters({
    sort: event.sortField,
    direction: event.sortOrder === 1 ? 'ASC' : 'DESC'
  })
  releasesStore.fetchReleases()
}

function openEntry(release: Release) {
  router.push({ name: 'entry', params: { id: release.id } })
}

function confirmDelete(release: Release) {
  confirm.require({
    message: `Delete "${release.artist} – ${release.title}"?`,
    header: 'Delete Release',
    icon: 'pi pi-trash',
    rejectProps: { label: 'Cancel', severity: 'secondary', outlined: true },
    acceptProps: { label: 'Delete', severity: 'danger' },
    accept: async () => {
      try {
        await releasesStore.deleteRelease(release.id)
        toast.add({ severity: 'info', summary: 'Deleted', life: 2000 })
      } catch (e: any) {
        toast.add({ severity: 'error', summary: 'Failed', detail: e.message, life: 3000 })
      }
    }
  })
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })
}
</script>

<style scoped>
.library-view {
  flex: 1;
  display: flex;
  flex-direction: column;
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

.library-layout {
  display: flex;
  gap: 1.5rem;
  flex: 1;
  align-items: flex-start;
}

.filters-panel {
  width: 260px;
  flex-shrink: 0;
  position: sticky;
  top: 1rem;
}

.library-content {
  flex: 1;
  min-width: 0;
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

/* ── Album art ── */
.table-art {
  width: 52px;
  height: 52px;
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

.table-art-placeholder {
  width: 52px;
  height: 52px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--tk-border);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--tk-text-muted);
}

.table-actions {
  display: flex;
  gap: 0.25rem;
}

/* ── DataTable deep overrides ── */
.library-table :deep(table) {
  border-collapse: separate;
  border-spacing: 0;
}

.library-table :deep(thead th) {
  font-family: var(--tk-font-display);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--tk-text-muted) !important;
  background: var(--tk-surface) !important;
  border-bottom: 1px solid var(--tk-border) !important;
  padding: 0.875rem 1rem !important;
  white-space: nowrap;
}

.library-table :deep(thead th:first-child) {
  border-radius: 12px 0 0 0;
}

.library-table :deep(thead th:last-child) {
  border-radius: 0 12px 0 0;
}

.library-table :deep(tbody tr) {
  transition: background 0.15s ease;
}

.library-table :deep(tbody tr:hover td) {
  background: rgba(0, 229, 176, 0.05) !important;
}

.library-table :deep(tbody tr:hover td:first-child) {
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.library-table :deep(tbody td) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.04) !important;
  padding: 0.75rem 1rem !important;
  color: var(--tk-text);
}

.library-table :deep(.p-datatable-table-container) {
  border-radius: 14px;
  border: 1px solid var(--tk-border);
  overflow: hidden;
  background: var(--tk-surface);
}

/* ── Grid view ── */
.releases-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1rem;
}

.grid-pagination {
  grid-column: 1 / -1;
  display: flex;
  justify-content: center;
  margin-top: 1.25rem;
}
</style>
