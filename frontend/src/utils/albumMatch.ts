import type { AlbumCandidate, UnlinkedRelease } from '@/types'

/**
 * Text as a match compares it: accents dropped, typographic quotes and dashes made plain,
 * case and runs of spaces ignored. MusicBrainz writes "You’re" where a note says "You're".
 * Punctuation is otherwise kept, so "IV" and "I.V." stay different albums.
 */
export function normalizeForMatch(text: string | null | undefined): string {
  return (text ?? '')
    .normalize('NFKD')
    .replaceAll(/\p{M}/gu, '')
    .replaceAll(/[‘’‚‛′`´]/g, "'")
    .replaceAll(/[“”„‟″]/g, '"')
    .replaceAll(/[‐‑‒–—―]/g, '-')
    .replaceAll(/…/g, '...')
    .toLowerCase()
    .replaceAll(/\s+/g, ' ')
    .trim()
}

/**
 * The one candidate that is unmistakably this album, or null when a person has to choose.
 *
 * Artist and title must be equal once normalized, and when the album has a year the
 * candidate's must be the same year. Exactly one candidate may pass: two that do, such as an
 * album and a single of the same name and year, are a choice, not a match.
 */
export function exactMatch(release: UnlinkedRelease, candidates: AlbumCandidate[]): AlbumCandidate | null {
  const artist = normalizeForMatch(release.artist)
  const title = normalizeForMatch(release.title)
  const hits = candidates.filter(
    (c) =>
      normalizeForMatch(c.artist) === artist &&
      normalizeForMatch(c.title) === title &&
      (release.releaseYear == null || c.releaseYear === release.releaseYear)
  )
  return hits.length === 1 ? hits[0] : null
}
