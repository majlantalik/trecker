import { describe, it, expect, beforeEach, vi } from 'vitest'

const get = vi.fn()
const update = vi.fn()
vi.mock('@/api/settings', () => ({ settingsApi: { get, update } }))

const { useSettings } = await import('./useSettings')

beforeEach(() => {
  vi.clearAllMocks()
  get.mockResolvedValue({ closeAction: 'quit', queueSort: 'newest', openLinksIn: 'web' })
  update.mockImplementation(async (s) => s)
})

describe('useSettings', () => {
  it('loads the current settings', async () => {
    const s = useSettings()
    await s.load()
    expect(s.settings.value).toEqual({ closeAction: 'quit', queueSort: 'newest', openLinksIn: 'web' })
  })

  it('saves a change at once', async () => {
    const s = useSettings()
    await s.load()
    await s.update({ closeAction: 'tray' })
    expect(update).toHaveBeenCalledWith({ closeAction: 'tray', queueSort: 'newest', openLinksIn: 'web' })
    expect(s.settings.value?.closeAction).toBe('tray')
    expect(s.error.value).toBe('')
  })

  it('undoes a change the app could not apply, and says why', async () => {
    // A tray icon on a desktop that has no tray. Leaving "tray" selected would claim a
    // behaviour the app does not have.
    update.mockRejectedValue({ code: 'INTERNAL', message: 'could not create the tray icon' })
    const s = useSettings()
    await s.load()
    await s.update({ closeAction: 'tray' })
    expect(s.settings.value?.closeAction).toBe('quit')
    expect(s.error.value).toBe('could not create the tray icon')
    expect(s.saving.value).toBe(false)
  })
})
