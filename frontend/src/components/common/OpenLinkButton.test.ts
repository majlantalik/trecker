import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import type { Settings } from '@/types'

const openUri = vi.fn()
vi.mock('@/api/opener', () => ({ openerApi: { openUri } }))

const get = vi.fn()
vi.mock('@/api/settings', () => ({ settingsApi: { get, update: vi.fn() } }))

const toast = vi.fn()
vi.mock('primevue/usetoast', () => ({ useToast: () => ({ add: toast }) }))

const { default: OpenLinkButton } = await import('./OpenLinkButton.vue')
const { useSettings } = await import('@/composables/useSettings')

const spotify = 'https://open.spotify.com/album/4ZoR'

function settings(openLinksIn: Settings['openLinksIn']): Settings {
  return { closeAction: 'quit', queueSort: 'newest', openLinksIn }
}

async function mountButton(links: Record<string, string> | undefined, parentClick = vi.fn()) {
  const w = mount(
    { components: { OpenLinkButton }, props: ['links'], template: '<div @click="parentClick"><OpenLinkButton :links="links" /></div>', setup: () => ({ parentClick }) },
    { props: { links }, global: { directives: { tooltip: {} } } }
  )
  await flushPromises()
  return w
}

beforeEach(() => {
  vi.clearAllMocks()
  openUri.mockResolvedValue(undefined)
  get.mockResolvedValue(settings('web'))
  // The settings are shared by the whole app, so each test starts them afresh.
  useSettings().settings.value = null
})

describe('open-link button', () => {
  it('stays in place but disabled when the album has no link', async () => {
    const w = await mountButton({})
    const button = w.find('button')
    expect(button.exists()).toBe(true)
    expect(button.attributes('disabled')).toBeDefined()
    expect(button.attributes('aria-label')).toBe('No streaming link')
  })

  it('opens the web page, and not the row under it', async () => {
    const parentClick = vi.fn()
    const w = await mountButton({ spotify }, parentClick)

    await w.find('button').trigger('click')

    expect(openUri).toHaveBeenCalledWith(spotify)
    expect(parentClick).not.toHaveBeenCalled()
  })

  it('opens the desktop app when the settings prefer it', async () => {
    get.mockResolvedValue(settings('app'))
    const w = await mountButton({ spotify })

    await w.find('button').trigger('click')

    expect(openUri).toHaveBeenCalledWith('spotify:album:4ZoR')
    expect(w.find('button').attributes('aria-label')).toBe('Open in Spotify app')
  })

  it('falls back to the web page when the app cannot be opened', async () => {
    get.mockResolvedValue(settings('app'))
    openUri.mockRejectedValueOnce(new Error('no handler for spotify:'))
    const w = await mountButton({ spotify })

    await w.find('button').trigger('click')
    await flushPromises()

    expect(openUri).toHaveBeenLastCalledWith(spotify)
    expect(toast).not.toHaveBeenCalled()
  })

  it('says so when nothing could be opened', async () => {
    openUri.mockRejectedValue(new Error('blocked'))
    const w = await mountButton({ spotify })

    await w.find('button').trigger('click')
    await flushPromises()

    expect(toast).toHaveBeenCalledWith(expect.objectContaining({ summary: 'Could not open the link', detail: 'blocked' }))
  })

  it('reads the settings file once for any number of buttons', async () => {
    const links = { spotify }
    mount(
      { components: { OpenLinkButton }, template: '<div><OpenLinkButton v-for="n in 20" :key="n" :links="links" /></div>', setup: () => ({ links }) },
      { global: { directives: { tooltip: {} } } }
    )
    await flushPromises()
    expect(get).toHaveBeenCalledTimes(1)
  })
})
