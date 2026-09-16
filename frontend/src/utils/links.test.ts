import { describe, it, expect } from 'vitest'
import { streamingService } from './links'

describe('streamingService', () => {
  it('names the services a link is stored under', () => {
    expect(streamingService('https://open.spotify.com/album/1')).toBe('spotify')
    expect(streamingService('https://tidal.com/browse/album/1')).toBe('tidal')
    expect(streamingService('https://www.youtube.com/watch?v=x')).toBe('youtube')
    expect(streamingService('https://youtu.be/x')).toBe('youtube')
    expect(streamingService('https://band.bandcamp.com/album/x')).toBe('other')
  })
})
