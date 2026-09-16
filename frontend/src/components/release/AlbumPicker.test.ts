import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import PrimeVue from 'primevue/config'
import type { AlbumCandidate } from '@/types'

vi.mock('@/api/cache', () => ({ coverSrc: (url: string) => `cover://localhost/${encodeURIComponent(url)}` }))

const { default: AlbumPicker } = await import('./AlbumPicker.vue')

function candidate(id: string, title: string): AlbumCandidate {
  return {
    musicbrainzReleaseGroupId: id,
    artist: 'Avenged Sevenfold',
    title,
    releaseYear: 2003,
    primaryType: 'Album',
    secondaryTypes: [],
    disambiguation: null,
    albumArtUrl: `https://coverartarchive.org/release-group/${id}/front-250`
  }
}

const BOTH = [candidate('album', 'Waking the Fallen'), candidate('comp', 'Waking the Fallen / Sounding the Seventh Trumpet')]

function mountPicker(choosingId: string | null = null, extra: { candidates?: AlbumCandidate[]; links?: Record<string, string> } = {}) {
  return mount(AlbumPicker, {
    props: {
      visible: true,
      query: 'avenged sevenfold waking the fallen',
      choosingId,
      candidates: BOTH,
      ...extra
    },
    // The real Listbox and Button, not stubs: what is under test is focus, selection and the link.
    global: { plugins: [[PrimeVue, { unstyled: true }]], stubs: { Listbox: false, Button: false } }
  })
}

describe('AlbumPicker', () => {
  it('marks the list for the dialog to focus when it opens', () => {
    // Without this the dialog focuses its close button, and the arrow keys do nothing
    // until the list is clicked.
    const list = mountPicker().find('[role="listbox"]')
    expect(list.exists()).toBe(true)
    expect(list.attributes()).toHaveProperty('autofocus')
  })

  it('lists every candidate with its details', () => {
    const text = mountPicker().text()
    expect(text).toContain('Waking the Fallen / Sounding the Seventh Trumpet')
    expect(text).toContain('Album · 2003')
  })

  it('hands back the clicked album', async () => {
    const wrapper = mountPicker()
    await wrapper.findAll('[role="option"]')[1].trigger('click')
    expect(wrapper.emitted('choose')?.[0]?.[0]).toMatchObject({ musicbrainzReleaseGroupId: 'comp' })
  })

  it('ignores clicks while an album is being looked up', async () => {
    const wrapper = mountPicker('album')
    await wrapper.findAll('[role="option"]')[1].trigger('click')
    expect(wrapper.emitted('choose')).toBeUndefined()
  })

  it('offers to add the album to MusicBrainz through Harmony from its streaming links', () => {
    const wrapper = mountPicker(null, { links: { spotify: 'https://open.spotify.com/album/1' } })
    const add = wrapper.find('a[href^="https://harmony.pulsewidth.org.uk/release?"]')
    expect(add.exists()).toBe(true)
    expect(add.text()).toContain('Add to MusicBrainz')
    expect(add.attributes('target')).toBe('_blank')
  })

  it('has no add button for an album without a link Harmony reads', () => {
    expect(mountPicker().find('a').exists()).toBe(false)
    expect(mountPicker(null, { links: { youtube: 'https://youtu.be/x' } }).find('a').exists()).toBe(false)
  })

  it('still offers to add an album when nothing matched', async () => {
    const wrapper = mountPicker(null, { candidates: [], links: { tidal: 'https://tidal.com/browse/album/2' } })
    expect(wrapper.find('[role="listbox"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('MusicBrainz has no album like this')
    expect(wrapper.find('a').exists()).toBe(true)
    const close = wrapper.findAll('button').find((b) => b.text() === 'Close')
    await close!.trigger('click')
    expect(wrapper.emitted('none')).toHaveLength(1)
  })
})
