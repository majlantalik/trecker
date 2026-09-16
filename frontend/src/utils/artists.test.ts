import { describe, it, expect } from 'vitest'
import {
  activeYears,
  artistKind,
  describeArtist,
  initial,
  labelledLinks,
  linkLabel,
  verdictLabel,
  mainDiscography
} from './artists'
import type { DiscographyEntry } from '@/types'

describe('artistKind and activeYears', () => {
  it('names the kinds of artist in plain words', () => {
    expect(artistKind('Person')).toBe('Solo artist')
    expect(artistKind('Group')).toBe('Group')
    expect(artistKind('Other')).toBeNull()
    expect(artistKind(null)).toBeNull()
  })

  it('reads a band and a person differently', () => {
    expect(activeYears({ artistType: 'Group', beginYear: 1999, endYear: null })).toBe('Since 1999')
    expect(activeYears({ artistType: 'Group', beginYear: 1987, endYear: 1991 })).toBe('1987–1991')
    expect(activeYears({ artistType: 'Person', beginYear: 1970, endYear: null })).toBe('Born 1970')
    expect(activeYears({ artistType: 'Person', beginYear: null, endYear: null })).toBeNull()
  })

  it('describes an artist in one line', () => {
    expect(
      describeArtist({ artistType: 'Group', beginYear: 1999, endYear: null, disambiguation: 'American metal band' })
    ).toBe('Group · Since 1999 · American metal band')
    expect(describeArtist({ artistType: null, beginYear: null, endYear: null, disambiguation: null })).toBe('')
  })
})

describe('link labels', () => {
  it('names well-known services by their host', () => {
    expect(linkLabel({ kind: 'streaming', url: 'https://open.spotify.com/artist/x' })).toBe('Spotify')
    expect(linkLabel({ kind: 'youtube music', url: 'https://music.youtube.com/channel/x' })).toBe('YouTube Music')
    expect(linkLabel({ kind: 'youtube', url: 'https://www.youtube.com/user/x' })).toBe('YouTube')
    expect(linkLabel({ kind: 'bandcamp', url: 'https://duster.bandcamp.com/' })).toBe('Bandcamp')
  })

  it('calls an unknown homepage a website, and anything else by its host', () => {
    expect(linkLabel({ kind: 'official homepage', url: 'https://avengedsevenfold.com/' })).toBe('Website')
    expect(linkLabel({ kind: 'streaming', url: 'https://www.napster.com/artist/x' })).toBe('napster.com')
  })

  it('does not show two links with the same label', () => {
    const links = labelledLinks([
      { kind: 'streaming', url: 'https://open.spotify.com/artist/a' },
      { kind: 'free streaming', url: 'https://open.spotify.com/artist/b' },
      { kind: 'bandcamp', url: 'https://x.bandcamp.com' }
    ])
    expect(links.map((l) => l.label)).toEqual(['Spotify', 'Bandcamp'])
    expect(links[0].url).toBe('https://open.spotify.com/artist/a')
  })
})

describe('verdicts', () => {
  it('reads a verdict, or just "Checked" without one', () => {
    expect(verdictLabel({ verdict: 'LIKED' })).toBe('Into it')
    expect(verdictLabel({ verdict: 'NOT_FOR_ME' })).toBe('Not for me')
    expect(verdictLabel({ verdict: null })).toBe('Checked')
  })
})

describe('mainDiscography', () => {
  const entry = (id: string, secondaryTypes: string[], tracked = false): DiscographyEntry => ({
    musicbrainzReleaseGroupId: id,
    artist: null,
    title: id,
    releaseYear: null,
    primaryType: 'Album',
    secondaryTypes,
    disambiguation: null,
    albumArtUrl: '',
    library: tracked ? { releaseId: 'r', status: 'QUEUED', rating: null } : null
  })

  it('hides live albums and compilations until asked, except ones you track', () => {
    const all = [entry('studio', []), entry('live', ['Live']), entry('best of', ['Compilation'], true)]
    expect(mainDiscography(all).map((e) => e.title)).toEqual(['studio', 'best of'])
  })
})

describe('initial', () => {
  it('takes the first letter or digit', () => {
    expect(initial('avenged sevenfold')).toBe('A')
    expect(initial('...And You Will Know Us')).toBe('A')
    expect(initial('ßig')).toBe('SS')
    expect(initial('  ')).toBe('?')
  })
})
