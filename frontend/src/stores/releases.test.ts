import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { Release, PageResponse } from '@/types'

// Mock the releases API
const mockReleasesApi = {
  getAll: vi.fn(),
  create: vi.fn(),
  update: vi.fn(),
  delete: vi.fn()
}

vi.mock('@/api/releases', () => ({
  releasesApi: mockReleasesApi
}))

const { useReleasesStore } = await import('./releases')

function makeRelease(overrides: Partial<Release> = {}): Release {
  return {
    id: 'release-1',
    artist: 'Artist',
    title: 'Title',
    releaseYear: 2024,
    albumArtUrl: null,
    status: 'QUEUED',
    discoveryLink: null,
    streamingLinks: {},
    country: null,
    rating: null,
    didNotFinish: false,
    dateListened: null,
    notes: null,
    createdAt: '2024-01-01T00:00:00Z',
    genres: [],
    ...overrides
  }
}

function makePage(items: Release[]): PageResponse<Release> {
  return {
    content: items,
    totalElements: items.length,
    totalPages: 1,
    number: 0,
    size: 20,
    first: true,
    last: true
  }
}

describe('releases store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('fetchReleases populates releases and sets total', async () => {
    const releases = [makeRelease({ id: '1' }), makeRelease({ id: '2' })]
    mockReleasesApi.getAll.mockResolvedValue(makePage(releases))

    const store = useReleasesStore()
    await store.fetchReleases()

    expect(store.releases).toHaveLength(2)
    expect(store.total).toBe(2)
    expect(store.loading).toBe(false)
  })

  it('addRelease prepends the new release', async () => {
    const existing = makeRelease({ id: 'old' })
    const newRelease = makeRelease({ id: 'new' })
    mockReleasesApi.create.mockResolvedValue(newRelease)

    const store = useReleasesStore()
    store.releases = [existing]
    await store.addRelease({ artist: 'A', title: 'T' })

    expect(store.releases[0].id).toBe('new')
    expect(store.releases[1].id).toBe('old')
    expect(store.total).toBe(1) // incremented from 0 to 1
  })

  it('deleteRelease removes the correct release by id', async () => {
    mockReleasesApi.delete.mockResolvedValue(undefined)

    const store = useReleasesStore()
    store.releases = [makeRelease({ id: '1' }), makeRelease({ id: '2' }), makeRelease({ id: '3' })]
    store.total = 3
    await store.deleteRelease('2')

    expect(store.releases.map(r => r.id)).toEqual(['1', '3'])
    expect(store.total).toBe(2)
  })

  it('markAsListened changes status via updateRelease', async () => {
    const listened = makeRelease({ id: '1', status: 'LISTENED' })
    mockReleasesApi.update.mockResolvedValue(listened)

    const store = useReleasesStore()
    store.releases = [makeRelease({ id: '1' })]
    await store.markAsListened('1', { rating: 5 })

    expect(store.releases[0].status).toBe('LISTENED')
  })

  it('queuedReleases computed returns only QUEUED items', () => {
    const store = useReleasesStore()
    store.releases = [
      makeRelease({ id: '1', status: 'QUEUED' }),
      makeRelease({ id: '2', status: 'LISTENED' }),
      makeRelease({ id: '3', status: 'QUEUED' })
    ]

    expect(store.queuedReleases.map(r => r.id)).toEqual(['1', '3'])
  })

  it('listenedReleases computed returns only LISTENED items', () => {
    const store = useReleasesStore()
    store.releases = [
      makeRelease({ id: '1', status: 'QUEUED' }),
      makeRelease({ id: '2', status: 'LISTENED' }),
      makeRelease({ id: '3', status: 'LISTENED' })
    ]

    expect(store.listenedReleases.map(r => r.id)).toEqual(['2', '3'])
  })

  it('setFilters merges new filters and resets page to 0', () => {
    const store = useReleasesStore()
    store.currentPage = 3

    store.setFilters({ status: 'LISTENED', genre: 'Jazz' })

    expect(store.filters.status).toBe('LISTENED')
    expect(store.filters.genre).toBe('Jazz')
    expect(store.currentPage).toBe(0)
  })
})
