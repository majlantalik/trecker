import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { ArtistCandidate } from '@/types'

const api = { search: vi.fn(), add: vi.fn(), list: vi.fn() }
vi.mock('@/api/artists', () => ({ artistsApi: api }))

const { useArtistSearch } = await import('./useArtistSearch')
const { useArtistsStore } = await import('@/stores/artists')

const candidate = (id: string): ArtistCandidate => ({
  musicbrainzArtistId: id,
  name: id,
  disambiguation: null,
  artistType: 'Group',
  country: null,
  beginYear: null,
  endYear: null,
  tags: []
})

beforeEach(() => vi.clearAllMocks())

describe('useArtistSearch', () => {
  it('shows the list even for a single match', async () => {
    api.search.mockResolvedValue([candidate('one')])
    const s = useArtistSearch()
    expect(await s.search('  slint ')).toBe('found')
    expect(api.search).toHaveBeenCalledWith('slint')
    expect(s.pickerVisible.value).toBe(true)
    expect(api.add).not.toHaveBeenCalled()
  })

  it('reports nothing found and a failed search differently', async () => {
    api.search.mockResolvedValueOnce([])
    const s = useArtistSearch()
    expect(await s.search('zzzz')).toBe('none')
    expect(s.pickerVisible.value).toBe(false)

    api.search.mockRejectedValueOnce({ code: 'RESOLVE_FAILED', message: 'could not reach MusicBrainz' })
    expect(await s.search('slint')).toBe('failed')
    expect(s.error.value).toBe('could not reach MusicBrainz')
  })

  it('does not search for nothing', async () => {
    expect(await useArtistSearch().search('   ')).toBe('none')
    expect(api.search).not.toHaveBeenCalled()
  })

  it('adds the chosen artist to the list and closes the picker', async () => {
    api.add.mockResolvedValue({ id: 'a1', name: 'Slint', status: 'TO_CHECK' })
    const s = useArtistSearch()
    s.pickerVisible.value = true
    const artist = await s.choose(candidate('mb-slint'))
    expect(api.add).toHaveBeenCalledWith({ musicbrainzArtistId: 'mb-slint' })
    expect(artist?.id).toBe('a1')
    expect(s.pickerVisible.value).toBe(false)
    expect(s.addingId.value).toBeNull()
    expect(useArtistsStore().artists.map((a) => a.id)).toEqual(['a1'])
  })

  it('keeps the picker open and says why when adding fails', async () => {
    api.add.mockRejectedValue({ message: 'lookup timed out' })
    const s = useArtistSearch()
    s.pickerVisible.value = true
    expect(await s.choose(candidate('x'))).toBeNull()
    expect(s.pickerVisible.value).toBe(true)
    expect(s.error.value).toBe('lookup timed out')
  })

  it('ignores a second pick while the first is being added', async () => {
    let finish: (v: unknown) => void = () => {}
    api.add.mockReturnValue(new Promise((r) => (finish = r)))
    const s = useArtistSearch()
    const first = s.choose(candidate('a'))
    expect(await s.choose(candidate('b'))).toBeNull()
    finish({ id: 'a', status: 'TO_CHECK' })
    await first
    expect(api.add).toHaveBeenCalledTimes(1)
  })
})
