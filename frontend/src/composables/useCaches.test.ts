import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { CoverCacheInfo } from '@/types'

const covers = vi.fn()
const clearCovers = vi.fn()
const clearWebview = vi.fn()
vi.mock('@/api/cache', () => ({ cacheApi: { covers, clearCovers, clearWebview } }))

const { useCaches, describeCovers, describeCleared } = await import('./useCaches')

function info(count: number, sizeBytes: number): CoverCacheInfo {
  return { path: '/home/me/.cache/cz.mtulek.trecker/covers', count, sizeBytes }
}

beforeEach(() => {
  vi.clearAllMocks()
  covers.mockResolvedValue(info(2, 67_362))
  clearCovers.mockResolvedValue(info(2, 67_362))
  clearWebview.mockResolvedValue(undefined)
})

describe('describeCovers', () => {
  it('says how many covers and how much space', () => {
    expect(describeCovers(info(1, 33_681))).toBe('1 cover, 32.9 KB')
    expect(describeCovers(info(1200, 40 * 1024 * 1024))).toBe('1200 covers, 40.0 MB')
  })

  it('reads as a state rather than a zero when nothing has been downloaded', () => {
    expect(describeCovers(info(0, 0))).toBe('none downloaded yet')
  })
})

describe('describeCleared', () => {
  it('reports what was removed', () => {
    expect(describeCleared(info(2, 67_362))).toBe('Removed 2 covers, 65.8 KB')
    expect(describeCleared(info(0, 0))).toBe('There were no covers to remove')
  })
})

describe('useCaches', () => {
  it('loads what the cover cache holds', async () => {
    const c = useCaches()
    await c.load()
    expect(c.info.value).toEqual(info(2, 67_362))
  })

  it('clears covers, says what went, and shows the cache as empty afterwards', async () => {
    const c = useCaches()
    await c.load()
    covers.mockResolvedValue(info(0, 0))

    await c.clearCovers()
    expect(clearCovers).toHaveBeenCalledOnce()
    expect(c.message.value).toBe('Removed 2 covers, 65.8 KB')
    expect(c.info.value?.count).toBe(0)
  })

  it('clears the webview data', async () => {
    const c = useCaches()
    await c.clearWebview()
    expect(clearWebview).toHaveBeenCalledOnce()
    expect(c.message.value).toBe('Web view data cleared')
  })

  it('surfaces a failure instead of claiming success', async () => {
    clearCovers.mockRejectedValue({ code: 'INTERNAL', message: 'permission denied' })
    const c = useCaches()
    await c.clearCovers()
    expect(c.error.value).toBe('permission denied')
    expect(c.message.value).toBe('')
    expect(c.busy.value).toBe(false)
  })
})
