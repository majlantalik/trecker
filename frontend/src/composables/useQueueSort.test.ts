import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { ReleaseFilterParams } from '@/types'

const get = vi.fn()
const update = vi.fn()
vi.mock('@/api/settings', () => ({ settingsApi: { get, update } }))

const { queueParams, useQueueSort } = await import('./useQueueSort')

beforeEach(() => {
  vi.clearAllMocks()
  get.mockResolvedValue({ closeAction: 'tray', queueSort: 'oldest' })
  update.mockImplementation(async (s) => s)
})

describe('queueParams', () => {
  it('sorts queued releases by date added in the chosen direction', () => {
    expect(queueParams('newest')).toMatchObject({ status: 'QUEUED', sort: 'createdAt', direction: 'DESC' })
    expect(queueParams('oldest')).toMatchObject({ status: 'QUEUED', sort: 'createdAt', direction: 'ASC' })
  })

  it('clears every filter the Library can leave behind in the store', () => {
    const libraryFilters: Required<Omit<ReleaseFilterParams, 'page' | 'size' | 'status' | 'sort' | 'direction'>> = {
      search: 'nightmare',
      genre: 'metal',
      country: 'US',
      year: 2010,
      ratingMin: 3,
      ratingMax: 5,
      didNotFinish: true,
      unlinked: true,
      withoutCover: true
    }
    const merged = { ...libraryFilters, ...queueParams('newest') }
    for (const key of Object.keys(libraryFilters)) {
      expect(merged[key as keyof typeof merged], key).toBeUndefined()
    }
  })
})

describe('useQueueSort', () => {
  it('is newest first until the settings are loaded', () => {
    expect(useQueueSort().sort.value).toBe('newest')
  })

  it('takes the saved order from the settings', async () => {
    const q = useQueueSort()
    await q.load()
    expect(q.sort.value).toBe('oldest')
  })

  it('saves a reversed order without touching the other settings', async () => {
    const q = useQueueSort()
    await q.load()
    expect(await q.toggle()).toBeNull()
    expect(update).toHaveBeenCalledWith({ closeAction: 'tray', queueSort: 'newest' })
    expect(q.sort.value).toBe('newest')
  })

  it('keeps the old order and reports why when saving fails', async () => {
    update.mockRejectedValue({ code: 'INTERNAL', message: 'could not save settings' })
    const q = useQueueSort()
    await q.load()
    expect(await q.toggle()).toBe('could not save settings')
    expect(q.sort.value).toBe('oldest')
  })
})
