import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { Artist } from '@/types'

const api = { list: vi.fn(), add: vi.fn(), update: vi.fn(), delete: vi.fn() }
vi.mock('@/api/artists', () => ({ artistsApi: api }))

const { useArtistsStore } = await import('./artists')

function makeArtist(overrides: Partial<Artist> = {}): Artist {
  return {
    id: 'a1',
    musicbrainzArtistId: 'mb1',
    name: 'Artist',
    disambiguation: null,
    artistType: 'Group',
    country: 'US',
    beginYear: 1999,
    endYear: null,
    imageUrl: null,
    genres: [],
    links: [],
    status: 'TO_CHECK',
    verdict: null,
    note: null,
    checkedAt: null,
    createdAt: '2026-09-14T10:00:00Z',
    ...overrides
  }
}

beforeEach(() => vi.clearAllMocks())

describe('artists store', () => {
  it('splits the list into artists to check and checked artists', async () => {
    api.list.mockResolvedValue([
      makeArtist({ id: '1' }),
      makeArtist({ id: '2', status: 'CHECKED', verdict: 'LIKED' }),
      makeArtist({ id: '3' })
    ])
    const store = useArtistsStore()
    await store.fetchArtists()
    expect(store.toCheck.map((a) => a.id)).toEqual(['1', '3'])
    expect(store.checked.map((a) => a.id)).toEqual(['2'])
    expect(store.loaded).toBe(true)
  })

  it('puts a newly added artist at the top', async () => {
    api.add.mockResolvedValue(makeArtist({ id: 'new' }))
    const store = useArtistsStore()
    store.artists = [makeArtist({ id: 'old' })]
    await store.addArtist({ musicbrainzArtistId: 'mb-new' })
    expect(store.artists.map((a) => a.id)).toEqual(['new', 'old'])
  })

  it('adding someone already on the list does not list them twice', async () => {
    api.add.mockResolvedValue(makeArtist({ id: 'old', note: 'kept' }))
    const store = useArtistsStore()
    store.artists = [makeArtist({ id: 'old' })]
    await store.addArtist({ musicbrainzArtistId: 'mb1' })
    expect(store.artists).toHaveLength(1)
    expect(store.artists[0].note).toBe('kept')
  })

  it('reloads the list when an update moves an artist between sections', async () => {
    api.update.mockResolvedValue(makeArtist({ id: '1', status: 'CHECKED' }))
    api.list.mockResolvedValue([makeArtist({ id: '1', status: 'CHECKED' })])
    const store = useArtistsStore()
    store.artists = [makeArtist({ id: '1' })]
    await store.updateArtist('1', { status: 'CHECKED' })
    expect(api.list).toHaveBeenCalled()
  })

  it('does not reload for a note edit', async () => {
    api.update.mockResolvedValue(makeArtist({ id: '1', note: 'x' }))
    const store = useArtistsStore()
    store.artists = [makeArtist({ id: '1' })]
    await store.updateArtist('1', { note: 'x' })
    expect(api.list).not.toHaveBeenCalled()
    expect(store.artists[0].note).toBe('x')
  })

  it('removes a deleted artist', async () => {
    api.delete.mockResolvedValue(undefined)
    const store = useArtistsStore()
    store.artists = [makeArtist({ id: '1' }), makeArtist({ id: '2' })]
    await store.deleteArtist('1')
    expect(store.artists.map((a) => a.id)).toEqual(['2'])
  })
})
