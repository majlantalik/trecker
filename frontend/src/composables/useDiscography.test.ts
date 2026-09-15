import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { DiscographyEntry, Release } from '@/types'

const artists = { discography: vi.fn() }
const releases = { lookup: vi.fn(), getById: vi.fn(), create: vi.fn(), delete: vi.fn(), getAll: vi.fn() }
vi.mock('@/api/artists', () => ({ artistsApi: artists }))
vi.mock('@/api/releases', () => ({ releasesApi: releases }))
vi.mock('@/api/genres', () => ({ genresApi: { getAll: vi.fn().mockResolvedValue([]) } }))

const { useDiscography } = await import('./useDiscography')

function entry(id: string, overrides: Partial<DiscographyEntry> = {}): DiscographyEntry {
  return {
    musicbrainzReleaseGroupId: id,
    artist: null,
    title: id,
    releaseYear: 2005,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: `https://coverartarchive.org/release-group/${id}/front-250`,
    library: null,
    ...overrides
  }
}

function release(id: string, overrides: Partial<Release> = {}): Release {
  return {
    id,
    artist: 'Avenged Sevenfold',
    title: 'City of Evil',
    releaseYear: 2005,
    albumArtUrl: null,
    status: 'QUEUED',
    discoveryLink: null,
    streamingLinks: {},
    country: 'US',
    rating: null,
    didNotFinish: false,
    dateListened: null,
    notes: null,
    createdAt: '2026-09-14T10:00:00Z',
    genres: [],
    ...overrides
  }
}

const lookedUp = {
  artist: 'Avenged Sevenfold',
  title: 'City of Evil',
  releaseYear: 2005,
  albumArtUrl: 'https://coverartarchive.org/release/x/1-250.jpg',
  country: 'US',
  streamingLinks: {},
  genres: ['heavy metal'],
  musicbrainzReleaseGroupId: 'city'
}

function setup() {
  return useDiscography(() => 'artist-1', () => 'Avenged Sevenfold')
}

beforeEach(() => {
  vi.clearAllMocks()
  releases.getAll.mockResolvedValue({ totalElements: 0, content: [] })
  releases.delete.mockResolvedValue(undefined)
})

describe('loading', () => {
  it('hides live albums and compilations until asked, and counts them', async () => {
    artists.discography.mockResolvedValue([entry('studio'), entry('live', { secondaryTypes: ['Live'] })])
    const d = setup()
    await d.load()
    expect(artists.discography).toHaveBeenCalledWith('artist-1')
    expect(d.visible.value.map((e) => e.title)).toEqual(['studio'])
    expect(d.hiddenCount.value).toBe(1)
    d.showAll.value = true
    expect(d.visible.value).toHaveLength(2)
  })

  it('keeps the error when MusicBrainz cannot be reached', async () => {
    artists.discography.mockRejectedValue({ message: 'could not reach MusicBrainz' })
    const d = setup()
    await d.load()
    expect(d.error.value).toBe('could not reach MusicBrainz')
    expect(d.loaded.value).toBe(false)
  })
})

describe('queueing an album', () => {
  it('opens the add form with the looked-up album, then marks it queued', async () => {
    artists.discography.mockResolvedValue([entry('city')])
    releases.lookup.mockResolvedValue(lookedUp)
    releases.create.mockResolvedValue(release('r1'))
    const d = setup()
    await d.load()

    await d.queue(d.entries.value[0])
    expect(releases.lookup).toHaveBeenCalledWith('city')
    expect(d.formVisible.value).toBe(true)
    expect(d.prefill.value).toMatchObject({ artist: 'Avenged Sevenfold', genres: ['heavy metal'], musicbrainzReleaseGroupId: 'city' })

    await d.submitForm({ artist: 'Avenged Sevenfold', title: 'City of Evil', musicbrainzReleaseGroupId: 'city' })
    expect(d.entries.value[0].library).toEqual({ releaseId: 'r1', status: 'QUEUED', rating: null })
  })

  it('fills the artist in from the page when the lookup fails', async () => {
    // Discography entries carry no artist credit.
    releases.lookup.mockRejectedValue({ message: 'timed out' })
    const d = setup()
    await d.queue(entry('city'))
    expect(d.prefill.value).toMatchObject({ artist: 'Avenged Sevenfold', title: 'city', genres: [] })
  })

  it('does nothing for an album already in the library', async () => {
    const d = setup()
    await d.queue(entry('city', { library: { releaseId: 'r', status: 'QUEUED', rating: null } }))
    expect(releases.lookup).not.toHaveBeenCalled()
  })
})

describe('logging an album', () => {
  it('logs a queued album without adding it again', async () => {
    releases.getById.mockResolvedValue(release('r1'))
    const d = setup()
    await d.log(entry('city', { library: { releaseId: 'r1', status: 'QUEUED', rating: null } }))
    expect(releases.create).not.toHaveBeenCalled()
    expect(d.logRelease.value?.id).toBe('r1')
    expect(d.logVisible.value).toBe(true)
  })

  it('adds an album that is not in the library, then marks it listened once logged', async () => {
    artists.discography.mockResolvedValue([entry('city')])
    releases.lookup.mockResolvedValue(lookedUp)
    releases.create.mockResolvedValue(release('r1'))
    const d = setup()
    await d.load()

    await d.log(d.entries.value[0])
    expect(releases.create).toHaveBeenCalledWith(
      expect.objectContaining({ artist: 'Avenged Sevenfold', genres: ['heavy metal'], musicbrainzReleaseGroupId: 'city' })
    )
    expect(d.logVisible.value).toBe(true)

    d.logged(release('r1', { status: 'LISTENED', rating: 4 }))
    await d.logClosed()
    expect(releases.delete).not.toHaveBeenCalled()
    expect(d.entries.value[0].library).toEqual({ releaseId: 'r1', status: 'LISTENED', rating: 4 })
  })

  it('takes an album added only to be logged back out when the dialog closes unlogged', async () => {
    artists.discography.mockResolvedValue([entry('city')])
    releases.lookup.mockResolvedValue(lookedUp)
    releases.create.mockResolvedValue(release('r1'))
    const d = setup()
    await d.load()

    await d.log(d.entries.value[0])
    await d.logClosed()
    expect(releases.delete).toHaveBeenCalledWith('r1')
    expect(d.entries.value[0].library).toBeNull()
  })

  it('never removes an album that was already queued', async () => {
    releases.getById.mockResolvedValue(release('r1'))
    const d = setup()
    await d.log(entry('city', { library: { releaseId: 'r1', status: 'QUEUED', rating: null } }))
    await d.logClosed()
    expect(releases.delete).not.toHaveBeenCalled()
  })
})
