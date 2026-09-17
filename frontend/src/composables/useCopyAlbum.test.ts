import { describe, it, expect, beforeEach, vi } from 'vitest'

const add = vi.fn()
vi.mock('primevue/usetoast', () => ({ useToast: () => ({ add }) }))

const { albumText, useCopyAlbum } = await import('./useCopyAlbum')

const writeText = vi.fn()

beforeEach(() => {
  vi.clearAllMocks()
  writeText.mockResolvedValue(undefined)
  Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
})

describe('albumText', () => {
  it('is the artist and the title joined by a spaced hyphen', () => {
    expect(albumText({ artist: 'Sigur Rós', title: 'Ágætis byrjun' })).toBe('Sigur Rós - Ágætis byrjun')
  })

  it('leaves a title with punctuation of its own alone', () => {
    expect(albumText({ artist: 'Godspeed You! Black Emperor', title: "F♯ A♯ ∞" }))
      .toBe('Godspeed You! Black Emperor - F♯ A♯ ∞')
  })
})

describe('useCopyAlbum', () => {
  it('writes the line to the clipboard and says what was copied', async () => {
    await useCopyAlbum().copyAlbum({ artist: 'Boards of Canada', title: 'Geogaddi' })

    expect(writeText).toHaveBeenCalledWith('Boards of Canada - Geogaddi')
    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ severity: 'success', detail: 'Boards of Canada - Geogaddi' })
    )
  })

  // A webview can refuse the clipboard. Silence would leave the button looking broken,
  // so the line goes into the toast to be read or selected from there.
  it('shows the line instead when the clipboard is refused', async () => {
    writeText.mockRejectedValue(new Error('NotAllowedError'))

    await useCopyAlbum().copyAlbum({ artist: 'Burial', title: 'Untrue' })

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ severity: 'warn', detail: 'Burial - Untrue' })
    )
  })

  it('shows the line when the webview has no clipboard at all', async () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true })

    await useCopyAlbum().copyAlbum({ artist: 'Burial', title: 'Untrue' })

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ severity: 'warn', detail: 'Burial - Untrue' })
    )
  })
})
