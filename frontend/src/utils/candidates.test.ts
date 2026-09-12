import { describe, it, expect } from 'vitest'
import { candidateKind, describeCandidate, prefillFromCandidate, prefillFromText } from './candidates'
import type { AlbumCandidate, ResolvedMetadata } from '@/types'

function candidate(over: Partial<AlbumCandidate> = {}): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: 'group-1',
    artist: 'Avenged Sevenfold',
    title: 'Nightmare',
    releaseYear: 2010,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: 'https://coverartarchive.org/release-group/group-1/front-250',
    ...over
  }
}

describe('candidateKind', () => {
  it('reads as a phrase', () => {
    expect(candidateKind(candidate())).toBe('Album')
    expect(candidateKind(candidate({ primaryType: 'Single' }))).toBe('Single')
    expect(candidateKind(candidate({ secondaryTypes: ['Live'] }))).toBe('Live album')
    expect(candidateKind(candidate({ primaryType: 'EP', secondaryTypes: ['Remix'] }))).toBe('Remix EP')
  })

  it('drops "Other", which says nothing, and keeps what it qualified', () => {
    expect(candidateKind(candidate({ primaryType: 'Other', secondaryTypes: ['Interview'] }))).toBe('Interview')
    expect(candidateKind(candidate({ primaryType: 'Other' }))).toBeNull()
    expect(candidateKind(candidate({ primaryType: null }))).toBeNull()
  })
})

describe('describeCandidate', () => {
  it('joins type, year and disambiguation, skipping what is missing', () => {
    expect(describeCandidate(candidate())).toBe('Album · 2010')
    expect(
      describeCandidate(candidate({ secondaryTypes: ['Live'], releaseYear: 1996, disambiguation: 'bootleg' }))
    ).toBe('Live album · 1996 · bootleg')
    expect(describeCandidate(candidate({ primaryType: null, releaseYear: null }))).toBe('')
  })
})

describe('prefillFromCandidate', () => {
  const details: ResolvedMetadata = {
    artist: 'Avenged Sevenfold',
    title: 'Nightmare',
    releaseYear: 2010,
    albumArtUrl: 'https://coverartarchive.org/release/x/1-250.jpg',
    country: 'US',
    streamingLinks: {},
    genres: ['heavy metal'],
    musicbrainzReleaseGroupId: 'group-1'
  }

  it('takes everything from the lookup and keeps the chosen id', () => {
    const p = prefillFromCandidate(candidate(), details)
    expect(p).toMatchObject({ country: 'US', genres: ['heavy metal'], musicbrainzReleaseGroupId: 'group-1' })
    expect(p.albumArtUrl).toBe(details.albumArtUrl)
  })

  it('falls back to the search result when the lookup failed', () => {
    const p = prefillFromCandidate(candidate(), null)
    expect(p).toMatchObject({ artist: 'Avenged Sevenfold', title: 'Nightmare', releaseYear: 2010 })
    expect(p.musicbrainzReleaseGroupId).toBe('group-1')
  })

  it('never saves the search result\'s guessed cover address', () => {
    // It 404s for many albums. Only a cover the lookup actually found is kept.
    expect(prefillFromCandidate(candidate(), null).albumArtUrl).toBeNull()
    expect(prefillFromCandidate(candidate(), { ...details, albumArtUrl: null }).albumArtUrl).toBeNull()
  })
})

describe('prefillFromText', () => {
  it('splits on a spaced hyphen, en dash or em dash', () => {
    for (const text of ['Slint - Spiderland', 'Slint – Spiderland', 'Slint — Spiderland']) {
      expect(prefillFromText(text)).toMatchObject({ artist: 'Slint', title: 'Spiderland' })
    }
  })

  it('guesses nothing without a separator', () => {
    expect(prefillFromText('slint spiderland')).toBeNull()
    expect(prefillFromText('Blink-182')).toBeNull()
    expect(prefillFromText(' - Spiderland')).toBeNull()
  })
})
