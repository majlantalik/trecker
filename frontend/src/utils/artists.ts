import type { Artist, ArtistCandidate, ArtistLink, ArtistVerdict, DiscographyEntry } from '@/types'

/**
 * Presenting artists: what kind of artist, when active, where their links go, and which of
 * their albums to show.
 */

type ArtistBasics = Pick<ArtistCandidate, 'artistType' | 'beginYear' | 'endYear'>

// MusicBrainz's artist types as a reader would put them. "Other" says nothing.
const KINDS: Record<string, string> = {
  Group: 'Group',
  Person: 'Solo artist',
  Orchestra: 'Orchestra',
  Choir: 'Choir',
  Character: 'Character',
  Other: ''
}

export function artistKind(type: string | null): string | null {
  if (!type) return null
  return (KINDS[type] ?? type) || null
}

/**
 * "Since 1999", "1999–2010", or for a person "Born 1970". A person's life span is their
 * life, not a career, so it reads differently from a band's.
 */
export function activeYears(a: ArtistBasics): string | null {
  const { beginYear: begin, endYear: end } = a
  if (begin && end) return `${begin}–${end}`
  if (begin) return a.artistType === 'Person' ? `Born ${begin}` : `Since ${begin}`
  if (end) return a.artistType === 'Person' ? `Died ${end}` : `Until ${end}`
  return null
}

/** The line under an artist's name, without the country, which is shown with its flag. */
export function describeArtist(a: ArtistBasics & { disambiguation: string | null }): string {
  return [artistKind(a.artistType), activeYears(a), a.disambiguation].filter(Boolean).join(' · ')
}

// Hosts people recognise, checked in order, so music.youtube.com is not called YouTube.
const HOSTS: [string, string][] = [
  ['music.youtube.com', 'YouTube Music'],
  ['youtube.com', 'YouTube'],
  ['youtu.be', 'YouTube'],
  ['bandcamp.com', 'Bandcamp'],
  ['spotify.com', 'Spotify'],
  ['tidal.com', 'Tidal'],
  ['music.apple.com', 'Apple Music'],
  ['itunes.apple.com', 'Apple Music'],
  ['deezer.com', 'Deezer'],
  ['music.amazon', 'Amazon Music'],
  ['soundcloud.com', 'SoundCloud'],
  ['last.fm', 'Last.fm'],
  ['discogs.com', 'Discogs'],
  ['allmusic.com', 'AllMusic'],
  ['wikidata.org', 'Wikidata']
]

function hostOf(url: string): string | null {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return null
  }
}

/** "Bandcamp", "Spotify", "Website", or the bare host for anything else. */
export function linkLabel(link: ArtistLink): string {
  const host = hostOf(link.url)
  if (host) {
    const known = HOSTS.find(([h]) => host === h || host.endsWith(`.${h}`) || host.startsWith(h))
    if (known) return known[1]
  }
  if (link.kind === 'official homepage') return 'Website'
  return host ?? link.url
}

/** Links with a label, dropping a second link that would show the same label. */
export function labelledLinks(links: ArtistLink[]): { label: string; url: string }[] {
  const seen = new Set<string>()
  return links
    .map((link) => ({ label: linkLabel(link), url: link.url }))
    .filter(({ label }) => (seen.has(label) ? false : (seen.add(label), true)))
}

export const VERDICTS: Record<ArtistVerdict, { label: string; icon: string }> = {
  LIKED: { label: 'Into it', icon: 'pi pi-heart-fill' },
  NOT_FOR_ME: { label: 'Not for me', icon: 'pi pi-ban' }
}

export function verdictLabel(artist: Pick<Artist, 'verdict'>): string {
  return artist.verdict ? VERDICTS[artist.verdict].label : 'Checked'
}

/**
 * Whether an album is part of the main discography: a plain album or EP. Live recordings,
 * compilations, soundtracks and remixes are hidden until asked for. MusicBrainz lists many
 * of them, bootlegs included, and they bury the albums.
 */
export function isMainRelease(entry: Pick<DiscographyEntry, 'secondaryTypes'>): boolean {
  return entry.secondaryTypes.length === 0
}

export function visibleDiscography(entries: DiscographyEntry[], showAll: boolean): DiscographyEntry[] {
  // An album you already track stays visible whatever its type.
  return showAll ? entries : entries.filter((e) => isMainRelease(e) || e.library)
}

/** The first letter shown when an artist has no picture. */
export function initial(name: string): string {
  return (name.trim().match(/\p{L}|\p{N}/u)?.[0] ?? '?').toUpperCase()
}
