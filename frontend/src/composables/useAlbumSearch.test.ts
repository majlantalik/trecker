import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { AlbumCandidate, ResolvedMetadata } from '@/types'

const search = vi.fn()
const lookup = vi.fn()
vi.mock('@/api/releases', () => ({ releasesApi: { search, lookup } }))

const { useAlbumSearch } = await import('./useAlbumSearch')

function candidate(id: string, over: Partial<AlbumCandidate> = {}): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: id,
    artist: 'Avenged Sevenfold',
    title: 'Nightmare',
    releaseYear: 2010,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: `https://coverartarchive.org/release-group/${id}/front-250`,
    ...over
  }
}

const details: ResolvedMetadata = {
  artist: 'Avenged Sevenfold',
  title: 'Nightmare',
  releaseYear: 2010,
  albumArtUrl: 'https://coverartarchive.org/release/x/1-250.jpg',
  country: 'US',
  streamingLinks: {},
  genres: ['heavy metal'],
  musicbrainzReleaseGroupId: 'album'
}

beforeEach(() => {
  vi.clearAllMocks()
  lookup.mockResolvedValue(details)
})

describe('one match', () => {
  it('is chosen without asking, and the form opens prefilled', async () => {
    search.mockResolvedValue([candidate('album')])
    const s = useAlbumSearch()

    expect(await s.search('avenged sevenfold nightmare')).toBe('one')
    expect(lookup).toHaveBeenCalledWith('album')
    expect(s.pickerVisible.value).toBe(false)
    expect(s.formVisible.value).toBe(true)
    expect(s.prefill.value).toMatchObject({ country: 'US', genres: ['heavy metal'] })
  })
})

describe('several matches', () => {
  it('opens the list and looks nothing up until one is chosen', async () => {
    search.mockResolvedValue([candidate('album'), candidate('single', { primaryType: 'Single' })])
    const s = useAlbumSearch()

    expect(await s.search('avenged sevenfold nightmare')).toBe('many')
    expect(s.pickerVisible.value).toBe(true)
    expect(s.candidates.value).toHaveLength(2)
    expect(lookup).not.toHaveBeenCalled()
    expect(s.formVisible.value).toBe(false)
  })

  it('closes the list and opens the form with the chosen album', async () => {
    search.mockResolvedValue([candidate('album'), candidate('single', { primaryType: 'Single' })])
    const s = useAlbumSearch()
    await s.search('avenged sevenfold nightmare')

    await s.choose(s.candidates.value[1])
    expect(lookup).toHaveBeenCalledWith('single')
    expect(s.pickerVisible.value).toBe(false)
    expect(s.formVisible.value).toBe(true)
    expect(s.prefill.value?.musicbrainzReleaseGroupId).toBe('single')
    expect(s.choosingId.value).toBeNull()
  })

  it('ignores a second pick while the first is still loading', async () => {
    search.mockResolvedValue([candidate('album'), candidate('single')])
    let release: (v: ResolvedMetadata) => void = () => {}
    lookup.mockReturnValue(new Promise<ResolvedMetadata>((r) => (release = r)))
    const s = useAlbumSearch()
    await s.search('nightmare')

    const first = s.choose(s.candidates.value[0])
    await s.choose(s.candidates.value[1])
    expect(lookup).toHaveBeenCalledTimes(1)
    release(details)
    await first
  })

  it('still opens a form when the lookup fails', async () => {
    search.mockResolvedValue([candidate('album'), candidate('single')])
    lookup.mockRejectedValue({ code: 'RESOLVE_FAILED', message: 'timed out' })
    const s = useAlbumSearch()
    await s.search('nightmare')

    await s.choose(s.candidates.value[0])
    expect(s.formVisible.value).toBe(true)
    expect(s.prefill.value).toMatchObject({ artist: 'Avenged Sevenfold', title: 'Nightmare' })
  })

  it('"None of these" opens the form from the typed text', async () => {
    search.mockResolvedValue([candidate('album'), candidate('single')])
    const s = useAlbumSearch()
    await s.search('Avenged Sevenfold - Nightmare (demo)')

    s.chooseNone()
    expect(s.pickerVisible.value).toBe(false)
    expect(s.prefill.value).toMatchObject({ artist: 'Avenged Sevenfold', title: 'Nightmare (demo)' })
  })
})

describe('no match', () => {
  it('opens the form from the typed text and says nothing was found', async () => {
    search.mockResolvedValue([])
    const s = useAlbumSearch()
    expect(await s.search('Duster - Stratosphere')).toBe('none')
    expect(s.formVisible.value).toBe(true)
    expect(s.prefill.value).toMatchObject({ artist: 'Duster', title: 'Stratosphere' })
  })

  it('treats an unreachable MusicBrainz as a failure, not as nothing found', async () => {
    search.mockRejectedValue({ code: 'RESOLVE_FAILED', message: 'could not reach MusicBrainz' })
    const s = useAlbumSearch()
    expect(await s.search('slint spiderland')).toBe('failed')
    expect(s.error.value).toBe('could not reach MusicBrainz')
    expect(s.formVisible.value).toBe(true)
    expect(s.searching.value).toBe(false)
  })
})
