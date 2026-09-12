import type { AlbumCandidate, ResolvedMetadata } from '@/types'

/**
 * Presenting quick add search results, and turning a chosen one into a prefilled form.
 */

// MusicBrainz's primary types as they read in a sentence. "Other" says nothing, so it is
// dropped and the secondary type carries the label alone.
const PRIMARY: Record<string, string> = {
  Album: 'album',
  EP: 'EP',
  Single: 'single',
  Broadcast: 'broadcast',
  Other: ''
}

/** "Album", "Live album", "Remix EP", "Interview". Null when MusicBrainz gives no type. */
export function candidateKind(c: Pick<AlbumCandidate, 'primaryType' | 'secondaryTypes'>): string | null {
  const primary = c.primaryType ? (PRIMARY[c.primaryType] ?? c.primaryType.toLowerCase()) : ''
  const words = [...c.secondaryTypes.map((t) => t.toLowerCase()), primary].filter(Boolean)
  if (words.length === 0) return null
  const label = words.join(' ')
  return label.charAt(0).toUpperCase() + label.slice(1)
}

/** The line under a result's title: "Live album · 1996 · bootleg". */
export function describeCandidate(c: AlbumCandidate): string {
  return [candidateKind(c), c.releaseYear, c.disambiguation].filter(Boolean).join(' · ')
}

/**
 * The form's prefill for a chosen album.
 *
 * The lookup is the authority, and the search result fills in only what a failed lookup
 * left out, so a network hiccup still opens a form with the artist and title. The cover
 * is taken from the lookup alone: the search result's cover address is a guess that 404s
 * for many albums, and a broken address should not be saved.
 */
export function prefillFromCandidate(
  c: AlbumCandidate,
  details: ResolvedMetadata | null
): ResolvedMetadata {
  return {
    artist: details?.artist ?? c.artist,
    title: details?.title ?? c.title,
    releaseYear: details?.releaseYear ?? c.releaseYear,
    albumArtUrl: details?.albumArtUrl ?? null,
    country: details?.country ?? null,
    streamingLinks: details?.streamingLinks ?? {},
    genres: details?.genres ?? [],
    musicbrainzReleaseGroupId: c.musicbrainzReleaseGroupId
  }
}

/**
 * A prefill from the typed text alone, for when MusicBrainz has nothing. Only splits on a
 * spaced dash, the same separators the search understands; anything else is too ambiguous
 * to guess which words are the artist.
 */
export function prefillFromText(text: string): ResolvedMetadata | null {
  for (const sep of [' - ', ' – ', ' — ']) {
    const at = text.indexOf(sep)
    if (at === -1) continue
    const artist = text.slice(0, at).trim()
    const title = text.slice(at + sep.length).trim()
    if (!artist || !title) continue
    return {
      artist,
      title,
      releaseYear: null,
      albumArtUrl: null,
      country: null,
      streamingLinks: {},
      genres: []
    }
  }
  return null
}
