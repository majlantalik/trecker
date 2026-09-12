import { describe, it, expect } from 'vitest'

// Every .vue file's source, read as text.
const sources = import.meta.glob('../**/*.vue', { query: '?raw', import: 'default', eager: true }) as Record<string, string>

describe('cover rendering', () => {
  it('finds the components it is guarding', () => {
    // If the glob stopped matching, the test below would pass by checking nothing.
    expect(Object.keys(sources).length).toBeGreaterThan(10)
  })

  it('never binds a stored cover URL to an image without coverSrc', () => {
    // The content security policy blocks remote images, so a cover bound directly does
    // not fail a build or a test. It just never appears. This is the only thing that
    // would notice.
    const offenders = Object.entries(sources).flatMap(([file, text]) =>
      [...text.matchAll(/:src="([^"]*albumArtUrl[^"]*)"/g)]
        .map((m) => m[1])
        .filter((expr) => !expr.trim().startsWith('coverSrc('))
        .map((expr) => `${file}: ${expr}`)
    )
    expect(offenders).toEqual([])
  })
})
