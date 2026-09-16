import { describe, it, expect } from 'vitest'
import { harmonyLookupUrl, streamingService } from './links'

describe('streamingService', () => {
  it('names the services a link is stored under', () => {
    expect(streamingService('https://open.spotify.com/album/1')).toBe('spotify')
    expect(streamingService('https://tidal.com/browse/album/1')).toBe('tidal')
    expect(streamingService('https://www.youtube.com/watch?v=x')).toBe('youtube')
    expect(streamingService('https://youtu.be/x')).toBe('youtube')
    expect(streamingService('https://band.bandcamp.com/album/x')).toBe('other')
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
