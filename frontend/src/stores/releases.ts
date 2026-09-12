import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { releasesApi } from '@/api/releases'
import type { Release, ReleaseFilterParams, PageResponse } from '@/types'

export const useReleasesStore = defineStore('releases', () => {
  const releases = ref<Release[]>([])
  const total = ref(0)
  // Deliberately separate from `total`. `total` belongs to whatever list was fetched last,
  // so the sidebar badge was showing the Library count while sitting next to "Queue".
  const queuedTotal = ref(0)
  const totalPages = ref(0)
  const currentPage = ref(0)
  const pageSize = ref(20)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const filters = ref<ReleaseFilterParams>({
    sort: 'createdAt',
    direction: 'DESC'
  })

  const queuedReleases = computed(() =>
    releases.value.filter(r => r.status === 'QUEUED')
  )
  const listenedReleases = computed(() =>
    releases.value.filter(r => r.status === 'LISTENED')
  )

  async function fetchReleases(params?: ReleaseFilterParams) {
    loading.value = true
    error.value = null
    try {
      const queryParams = {
        ...filters.value,
        ...params,
        page: currentPage.value,
        size: pageSize.value
      }
      const result: PageResponse<Release> = await releasesApi.getAll(queryParams)
      releases.value = result.content
      total.value = result.totalElements
      totalPages.value = result.totalPages
    } catch (e: any) {
      error.value = e.message || 'Failed to fetch releases'
    } finally {
      loading.value = false
    }
  }

  /// Counts queued releases without disturbing the list the current view is showing.
  async function refreshQueuedCount() {
    try {
      const page = await releasesApi.getAll({ status: 'QUEUED', size: 1 })
      queuedTotal.value = page.totalElements
    } catch {
      // A badge is not worth surfacing an error for.
    }
  }

  async function addRelease(request: Parameters<typeof releasesApi.create>[0]) {
    const release = await releasesApi.create(request)
    releases.value = [release, ...releases.value]
    total.value++
    refreshQueuedCount()
    return release
  }

  async function updateRelease(id: string, request: Parameters<typeof releasesApi.update>[1]) {
    const updated = await releasesApi.update(id, request)
    // Logging or un-logging a release moves it in or out of the queue.
    if (request.status !== undefined) refreshQueuedCount()
    const idx = releases.value.findIndex(r => r.id === id)
    if (idx !== -1) {
      releases.value = [
        ...releases.value.slice(0, idx),
        updated,
        ...releases.value.slice(idx + 1)
      ]
    }
    return updated
  }

  async function deleteRelease(id: string) {
    await releasesApi.delete(id)
    releases.value = releases.value.filter(r => r.id !== id)
    total.value = Math.max(0, total.value - 1)
    refreshQueuedCount()
  }

  async function markAsListened(id: string, data: {
    rating?: number
    genres?: string[]
    didNotFinish?: boolean
    notes?: string
    country?: string
  }) {
    return updateRelease(id, {
      status: 'LISTENED',
      ...data
    })
  }

  function setFilters(newFilters: ReleaseFilterParams) {
    filters.value = { ...filters.value, ...newFilters }
    currentPage.value = 0
  }

  function setPage(page: number) {
    currentPage.value = page
  }

  return {
    releases,
    total,
    queuedTotal,
    refreshQueuedCount,
    totalPages,
    currentPage,
    pageSize,
    loading,
    error,
    filters,
    queuedReleases,
    listenedReleases,
    fetchReleases,
    addRelease,
    updateRelease,
    deleteRelease,
    markAsListened,
    setFilters,
    setPage
  }
})
