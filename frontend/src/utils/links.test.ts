import { describe, it, expect } from 'vitest'
import { harmonyLookupUrl, streamingService, spotifyAppUri, tidalAppUri, desktopAppLink, linkToOpen } from './links'

describe('streamingService', () => {
  it('names the services a link is stored under', () => {
    expect(streamingService('https://open.spotify.com/album/1')).toBe('spotify')
    expect(streamingService('https://tidal.com/browse/album/1')).toBe('tidal')
    expect(streamingService('https://www.youtube.com/watch?v=x')).toBe('youtube')
    expect(streamingService('https://youtu.be/x')).toBe('youtube')
    expect(streamingService('https://band.bandcamp.com/album/x')).toBe('other')
  })
})

describe('spotifyAppUri', () => {
  it('converts a share link to the app URI for every kind of Spotify page', () => {
    expect(spotifyAppUri('https://open.spotify.com/album/3b6NBQHsOh1gEMIiiQarA5')).toBe('spotify:album:3b6NBQHsOh1gEMIiiQarA5')
    expect(spotifyAppUri('https://open.spotify.com/track/abc123')).toBe('spotify:track:abc123')
    expect(spotifyAppUri('https://open.spotify.com/playlist/abc123')).toBe('spotify:playlist:abc123')
    expect(spotifyAppUri('https://open.spotify.com/artist/abc123')).toBe('spotify:artist:abc123')
  })

  it('ignores a share link that still carries sharing parameters', () => {
    expect(spotifyAppUri('https://open.spotify.com/album/3b6NBQHsOh1gEMIiiQarA5?si=xyz')).toBe('spotify:album:3b6NBQHsOh1gEMIiiQarA5')
  })

  it('gives nothing for anything that is not a Spotify album page', () => {
    expect(spotifyAppUri('https://tidal.com/browse/album/1')).toBeNull()
    expect(spotifyAppUri('https://open.spotify.com/')).toBeNull()
    expect(spotifyAppUri('not a url')).toBeNull()
  })
})

describe('tidalAppUri', () => {
  it('converts a share link to the app URI for every kind of Tidal page', () => {
    expect(tidalAppUri('https://tidal.com/album/12345')).toBe('tidal://album/12345')
    expect(tidalAppUri('https://tidal.com/track/12345/u')).toBe('tidal://track/12345')
    expect(tidalAppUri('https://tidal.com/artist/12345')).toBe('tidal://artist/12345')
    expect(tidalAppUri('https://tidal.com/playlist/abc-def-uuid')).toBe('tidal://playlist/abc-def-uuid')
    expect(tidalAppUri('https://tidal.com/mix/abc123')).toBe('tidal://mix/abc123')
  })

  it('also reads the older /browse/ prefixed links', () => {
    expect(tidalAppUri('https://tidal.com/browse/album/12345')).toBe('tidal://album/12345')
  })

  it('gives nothing for a video link or anything that is not Tidal', () => {
    expect(tidalAppUri('https://tidal.com/video/12345')).toBeNull()
    expect(tidalAppUri('https://open.spotify.com/album/1')).toBeNull()
    expect(tidalAppUri('not a url')).toBeNull()
  })
})

describe('desktopAppLink', () => {
  it('names the app a link opens in', () => {
    expect(desktopAppLink('https://open.spotify.com/album/1')).toEqual({ uri: 'spotify:album:1', label: 'Spotify' })
    expect(desktopAppLink('https://tidal.com/album/1')).toEqual({ uri: 'tidal://album/1', label: 'Tidal' })
    expect(desktopAppLink('https://www.youtube.com/watch?v=x')).toBeNull()
  })
})

describe('linkToOpen', () => {
  const spotify = 'https://open.spotify.com/album/4ZoR'
  const youtube = 'https://www.youtube.com/playlist?list=OLAK'

  it('has nothing to open without a streaming link', () => {
    expect(linkToOpen(undefined, 'web')).toBeNull()
    expect(linkToOpen({}, 'app')).toBeNull()
    expect(linkToOpen({ spotify: '' }, 'web')).toBeNull()
  })

  it('opens the web page when the browser is preferred', () => {
    expect(linkToOpen({ spotify }, 'web')).toEqual({ uri: spotify, fallback: null, label: 'Open on Spotify' })
    expect(linkToOpen({ other: 'https://x.bandcamp.com/album/y' }, 'web')?.label).toBe('Open on the web')
  })

  it('opens the app when preferred, keeping the web page to fall back on', () => {
    expect(linkToOpen({ spotify }, 'app')).toEqual({
      uri: 'spotify:album:4ZoR',
      fallback: spotify,
      label: 'Open in Spotify app'
    })
  })

  it('skips to a link that has an app', () => {
    expect(linkToOpen({ youtube, spotify }, 'app')?.uri).toBe('spotify:album:4ZoR')
  })

  it('opens the web page when no link has an app, rather than nothing', () => {
    expect(linkToOpen({ youtube }, 'app')).toEqual({ uri: youtube, fallback: null, label: 'Open on Youtube' })
  })
})

describe('harmonyLookupUrl', () => {
  it('passes every link Harmony can read', () => {
    const url = new URL(harmonyLookupUrl({
      spotify: 'https://open.spotify.com/album/1',
      tidal: 'https://tidal.com/browse/album/2',
      other: 'https://band.bandcamp.com/album/x'
    })!)
    expect(url.origin + url.pathname).toBe('https://harmony.pulsewidth.org.uk/release')
    expect(url.searchParams.getAll('url')).toEqual([
      'https://open.spotify.com/album/1',
      'https://tidal.com/browse/album/2',
      'https://band.bandcamp.com/album/x'
    ])
  })

  it('leaves out links it cannot read, and gives nothing without one', () => {
    const url = harmonyLookupUrl({ youtube: 'https://www.youtube.com/watch?v=x', spotify: 'https://open.spotify.com/album/1' })
    expect(new URL(url!).searchParams.getAll('url')).toEqual(['https://open.spotify.com/album/1'])
    expect(harmonyLookupUrl({ youtube: 'https://youtu.be/x', other: 'not a url' })).toBeNull()
    expect(harmonyLookupUrl({ other: 'https://notspotify.com/album/1' })).toBeNull()
    expect(harmonyLookupUrl(undefined)).toBeNull()
  })
})
