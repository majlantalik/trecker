import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { AlbumCandidate, UnlinkedRelease } from '@/types'

const unlinked = vi.fn()
const search = vi.fn()
const link = vi.fn()
vi.mock('@/api/releases', () => ({ releasesApi: { unlinked, search, link } }))
const fetchGenres = vi.fn()
vi.mock('@/stores/genres', () => ({ useGenresStore: () => ({ fetchGenres }) }))

const { useAlbumMatchStore, CONSECUTIVE_FAILURES_TO_STOP } = await import('./albumMatch')

function album(id: string, artist: string, title: string, releaseYear: number | null = 2022): UnlinkedRelease {
  return { id, artist, title, releaseYear }
}

function candidate(id: string, artist: string, title: string, releaseYear: number | null = 2022): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: id,
    artist,
    title,
    releaseYear,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: ''
  }
}

const exact = album('r1', 'Hexis', 'Aeternum')
const unsure = album('r2', 'KEN Mode', 'NULL')
const nothing = album('r3', 'Nobody', 'Nothing')

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  link.mockResolvedValue({})
  search.mockImplementation(async (q: string) => {
    if (q === 'Hexis - Aeternum') return [candidate('g1', 'Hexis', 'Aeternum'), candidate('g1b', 'Hexis', 'Aeternum', 2011)]
    if (q === 'KEN Mode - NULL') return [candidate('g2', 'KEN mode', 'NULL', 2023), candidate('g2b', 'KEN mode', 'Null Void')]
    return []
  })
})

describe('a run', () => {
  it('links exact matches, queues the rest for review and notes albums with no match', async () => {
    unlinked.mockResolvedValueOnce([exact, unsure, nothing]).mockResolvedValue([unsure, nothing])
    const store = useAlbumMatchStore()
    await store.run()

    expect(search).toHaveBeenCalledWith('Hexis - Aeternum')
    expect(link).toHaveBeenCalledTimes(1)
    expect(link).toHaveBeenCalledWith('r1', 'g1')
    expect(store.linked).toBe(1)
    expect(store.toReview.map((i) => i.release.id)).toEqual(['r2'])
    expect(store.current?.candidates).toHaveLength(2)
    expect(store.noMatches.map((r) => r.id)).toEqual(['r3'])
    expect(store.processed).toBe(3)
    expect(store.running).toBe(false)
    expect(store.finished).toBe(true)
    expect(store.unlinkedCount).toBe(2)
    expect(fetchGenres).toHaveBeenCalledWith(true)
  })

  it('records a duplicate with the release to open', async () => {
    unlinked.mockResolvedValue([exact])
    link.mockRejectedValue({ code: 'ALREADY_IN_LIBRARY', message: 'already in your library as Hexis – Aeternum', releaseId: 'r0' })
    const store = useAlbumMatchStore()
    await store.run()

    expect(store.linked).toBe(0)
    expect(store.duplicates).toEqual([
      { release: exact, reason: 'already in your library as Hexis – Aeternum', existingId: 'r0' }
    ])
  })

  it('stops when MusicBrainz keeps failing instead of failing every album', async () => {
    unlinked.mockResolvedValue(Array.from({ length: 10 }, (_, i) => album(`r${i}`, 'A', `T${i}`)))
    search.mockRejectedValue(new Error('offline'))
    const store = useAlbumMatchStore()
    await store.run()

    expect(search).toHaveBeenCalledTimes(CONSECUTIVE_FAILURES_TO_STOP)
    expect(store.failed).toHaveLength(CONSECUTIVE_FAILURES_TO_STOP)
    expect(store.error).toContain('MusicBrainz')
  })

  it('can be stopped between albums', async () => {
    unlinked.mockResolvedValue([exact, unsure, nothing])
    const store = useAlbumMatchStore()
    search.mockImplementationOnce(async () => {
      store.stop()
      return []
    })
    await store.run()

    expect(search).toHaveBeenCalledTimes(1)
    expect(store.processed).toBe(1)
    expect(store.finished).toBe(true)
  })

  it('does not start twice', async () => {
    unlinked.mockResolvedValue([])
    const store = useAlbumMatchStore()
    await Promise.all([store.run(), store.run()])
    expect(unlinked).toHaveBeenCalledTimes(2) // one run, plus its closing count
  })
})

describe('reviewing', () => {
  async function withOneToReview() {
    unlinked.mockResolvedValue([unsure])
    const store = useAlbumMatchStore()
    await store.run()
    fetchGenres.mockClear()
    return store
  }

  it('links the chosen album and moves on', async () => {
    const store = await withOneToReview()
    await store.choose(store.current!.candidates[1])

    expect(link).toHaveBeenCalledWith('r2', 'g2b')
    expect(store.linked).toBe(1)
    expect(store.current).toBeNull()
    expect(store.choosingId).toBeNull()
    expect(fetchGenres).toHaveBeenCalledWith(true)
  })

  it('moves on without linking for none of these', async () => {
    const store = await withOneToReview()
    store.decline()

    expect(link).not.toHaveBeenCalled()
    expect(store.declined).toBe(1)
    expect(store.toReview).toHaveLength(0)
  })

  it('moves on and reports a failed link', async () => {
    const store = await withOneToReview()
    link.mockRejectedValue(new Error('timeout'))
    await store.choose(store.current!.candidates[0])

    expect(store.failed).toEqual([{ release: unsure, reason: 'timeout' }])
    expect(store.current).toBeNull()
  })

  it('ignores a second pick while the first is linking', async () => {
    const store = await withOneToReview()
    let finish: () => void = () => {}
    link.mockReturnValue(new Promise<void>((resolve) => (finish = resolve)))

    const first = store.choose(store.current!.candidates[0])
    await store.choose(store.current!.candidates[1])
    store.decline()
    finish()
    await first

    expect(link).toHaveBeenCalledTimes(1)
    expect(store.declined).toBe(0)
  })
})
