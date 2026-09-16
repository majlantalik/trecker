import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { AlbumCandidate, Release } from '@/types'

const search = vi.fn()
const link = vi.fn()
vi.mock('@/api/releases', () => ({ releasesApi: { search, link } }))

const { useAlbumLink } = await import('./useAlbumLink')

function candidate(id: string): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: id,
    artist: 'KEN mode',
    title: 'NULL',
    releaseYear: 2022,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: `https://coverartarchive.org/release-group/${id}/front-250`
  }
}

const imported = { artist: 'KEN Mode', title: 'NULL' }
const linked = { id: 'r1', artist: 'KEN mode', title: 'NULL' } as Release

beforeEach(() => {
  vi.clearAllMocks()
  link.mockResolvedValue(linked)
})

describe('searching', () => {
  it('searches artist and title separately', async () => {
    search.mockResolvedValue([])
    await useAlbumLink().search(imported)
    expect(search).toHaveBeenCalledWith('KEN Mode - NULL')
  })

  it('asks even when there is only one match', async () => {
    search.mockResolvedValue([candidate('g1')])
    const l = useAlbumLink()

    expect(await l.search(imported)).toBe('found')
    expect(l.pickerVisible.value).toBe(true)
    expect(link).not.toHaveBeenCalled()
  })

  it('reports no matches without opening the list', async () => {
    search.mockResolvedValue([])
    const l = useAlbumLink()
    expect(await l.search(imported)).toBe('none')
    expect(l.pickerVisible.value).toBe(false)
  })

  it('reports an unreachable MusicBrainz', async () => {
    search.mockRejectedValue(new Error('offline'))
    const l = useAlbumLink()
    expect(await l.search(imported)).toBe('failed')
    expect(l.error.value).toBe('offline')
    expect(l.searching.value).toBe(false)
  })
})

describe('choosing', () => {
  it('links the chosen album and closes the list', async () => {
    search.mockResolvedValue([candidate('g1'), candidate('g2')])
    const l = useAlbumLink()
    await l.search(imported)

    expect(await l.choose('r1', l.candidates.value[1])).toBe(linked)
    expect(link).toHaveBeenCalledWith('r1', 'g2')
    expect(l.pickerVisible.value).toBe(false)
    expect(l.choosingId.value).toBeNull()
  })

  it('ignores a second pick while the first is linking', async () => {
    let finish: (r: Release) => void = () => {}
    link.mockReturnValue(new Promise((resolve) => (finish = resolve)))
    const l = useAlbumLink()

    const first = l.choose('r1', candidate('g1'))
    expect(await l.choose('r1', candidate('g2'))).toBeNull()
    finish(linked)
    await first
    expect(link).toHaveBeenCalledTimes(1)
  })

  it('closes the list and rethrows when the album is already in the library', async () => {
    link.mockRejectedValue({ code: 'ALREADY_IN_LIBRARY', message: 'already', releaseId: 'r0' })
    search.mockResolvedValue([candidate('g1')])
    const l = useAlbumLink()
    await l.search(imported)

    await expect(l.choose('r1', candidate('g1'))).rejects.toMatchObject({ releaseId: 'r0' })
    expect(l.pickerVisible.value).toBe(false)
    expect(l.choosingId.value).toBeNull()
  })
})
