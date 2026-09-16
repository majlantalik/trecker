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
