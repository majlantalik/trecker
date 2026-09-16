/**
 * The key a pasted streaming link is stored under in `streamingLinks`. Anything that is not
 * Spotify, Tidal or YouTube is kept as "other".
 */
export function streamingService(url: string): string {
  if (url.includes('spotify.com')) return 'spotify'
  if (url.includes('tidal.com')) return 'tidal'
  if (url.includes('youtube.com') || url.includes('youtu.be')) return 'youtube'
  return 'other'
}

/** Hosts whose album pages Harmony can read. YouTube is not one of them. */
const HARMONY_HOSTS = ['spotify.com', 'tidal.com', 'deezer.com', 'music.apple.com', 'itunes.apple.com', 'bandcamp.com', 'beatport.com']

/**
 * A Harmony lookup of an album's streaming links, or null when it has none Harmony reads.
 * Harmony merges what every service says and fills in MusicBrainz's "Add release" form, so a
 * person can add an album MusicBrainz is missing without typing in its tracklist.
 */
export function harmonyLookupUrl(links: Record<string, string> | null | undefined): string | null {
  const urls = Object.values(links ?? {}).filter((url) => {
    try {
      const host = new URL(url).hostname
      return HARMONY_HOSTS.some((h) => host === h || host.endsWith(`.${h}`))
    } catch {
      return false
    }
  })
  if (!urls.length) return null
  const params = new URLSearchParams(urls.map((url) => ['url', url]))
  return `https://harmony.pulsewidth.org.uk/release?${params}`
}
