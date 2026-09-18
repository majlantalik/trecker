import type { LinkTarget } from '@/types'

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

/**
 * The `spotify:` URI for a share link, so the desktop app can open it directly instead of
 * the web player. Null for anything that is not a Spotify album/track/playlist/artist link,
 * including share links whose sharing parameters have not been stripped yet.
 */
export function spotifyAppUri(url: string): string | null {
  let parsed: URL
  try {
    parsed = new URL(url)
  } catch {
    return null
  }
  if (parsed.hostname !== 'open.spotify.com') return null
  const match = /^\/(track|album|playlist|artist|episode|show)\/([A-Za-z0-9]+)/.exec(parsed.pathname)
  if (!match) return null
  return `spotify:${match[1]}:${match[2]}`
}

/**
 * The `tidal://` URI for a share link, matching the format the desktop client (SONE, the
 * only one registered as a `tidal:` handler on Linux) reads: `tidal://<type>/<id>`, the
 * type read from the same position as its own share links (`tidal.com/<type>/<id>`, with an
 * older `/browse/` prefix some links still carry). Null for anything else, including a
 * video link, which no desktop client here parses.
 */
export function tidalAppUri(url: string): string | null {
  let parsed: URL
  try {
    parsed = new URL(url)
  } catch {
    return null
  }
  if (parsed.hostname !== 'tidal.com') return null
  const match = /^\/(?:browse\/)?(track|album|artist|playlist|mix)\/([^/]+)/.exec(parsed.pathname)
  if (!match) return null
  return `tidal://${match[1]}/${match[2]}`
}

/**
 * The desktop app's own URI for a share link, when one is known, alongside which app it
 * names in the button that opens it.
 */
export function desktopAppLink(url: string): { uri: string; label: string } | null {
  const spotify = spotifyAppUri(url)
  if (spotify) return { uri: spotify, label: 'Spotify' }
  const tidal = tidalAppUri(url)
  if (tidal) return { uri: tidal, label: 'Tidal' }
  return null
}

/** What the open-link button opens: the address, a web address to try if that fails, and its label. */
export interface LinkChoice {
  uri: string
  fallback: string | null
  label: string
}

/**
 * What the open-link button opens for an album, or null when it has no streaming link.
 *
 * Preferring the app picks the first link a desktop app can open and keeps its web address
 * as the fallback. When no link has an app, as for a YouTube link, the web link opens
 * instead, so the preference never leaves the button doing nothing.
 */
export function linkToOpen(
  links: Record<string, string> | null | undefined,
  target: LinkTarget
): LinkChoice | null {
  const entries = Object.entries(links ?? {}).filter(([, url]) => url)
  if (!entries.length) return null
  if (target === 'app') {
    for (const [, url] of entries) {
      const app = desktopAppLink(url)
      if (app) return { uri: app.uri, fallback: url, label: `Open in ${app.label} app` }
    }
  }
  const [service, url] = entries[0]
  const name = service === 'other' ? 'the web' : service.charAt(0).toUpperCase() + service.slice(1)
  return { uri: url, fallback: null, label: `Open on ${name}` }
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
