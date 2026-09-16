import { describe, it, expect } from 'vitest'
import { exactMatch, normalizeForMatch } from './albumMatch'
import type { AlbumCandidate, UnlinkedRelease } from '@/types'

function candidate(id: string, over: Partial<AlbumCandidate> = {}): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: id,
    artist: 'Rare Americans',
    title: 'You’re Not A Bad Person, It’s Just A Bad World',
    releaseYear: 2022,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: '',
    ...over
  }
}

const release: UnlinkedRelease = {
  id: 'r1',
  artist: 'Rare Americans',
  title: "You're Not A Bad Person, it's Just A Bad World",
  releaseYear: 2022
}

describe('normalizeForMatch', () => {
  it('ignores case, accents, spacing and typographic punctuation', () => {
    expect(normalizeForMatch('  Björk ')).toBe('bjork')
    expect(normalizeForMatch('Madness  /  Medicine')).toBe('madness / medicine')
    expect(normalizeForMatch('It’s – “here”…')).toBe('it\'s - "here"...')
  })

  it('keeps other punctuation', () => {
    expect(normalizeForMatch('P.O.K.E.R.F.A.C.E.')).not.toBe(normalizeForMatch('POKERFACE'))
  })

  it('treats a missing value as empty', () => {
    expect(normalizeForMatch(null)).toBe('')
  })
})

describe('exactMatch', () => {
  it('matches the same artist, title and year despite quotes and case', () => {
    const hit = candidate('album')
    expect(exactMatch(release, [candidate('other', { title: 'Something Else' }), hit])).toBe(hit)
  })

  it('refuses a different year, so a reissue or a namesake is not taken', () => {
    expect(exactMatch(release, [candidate('old', { releaseYear: 2019 })])).toBeNull()
  })

  it('needs the candidate to have the year when the album has one', () => {
    expect(exactMatch(release, [candidate('undated', { releaseYear: null })])).toBeNull()
  })

  it('ignores the year when the album has none', () => {
    const hit = candidate('album', { releaseYear: 2019 })
    expect(exactMatch({ ...release, releaseYear: null }, [hit])).toBe(hit)
  })

  it('leaves two equal candidates to a person', () => {
    expect(exactMatch(release, [candidate('album'), candidate('single', { primaryType: 'Single' })])).toBeNull()
  })

  it('does not match on the title alone', () => {
    expect(exactMatch(release, [candidate('cover', { artist: 'Someone Else' })])).toBeNull()
  })
})
