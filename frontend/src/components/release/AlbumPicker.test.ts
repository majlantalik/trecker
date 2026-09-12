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

function mountPicker(choosingId: string | null = null) {
  return mount(AlbumPicker, {
    props: {
      visible: true,
      query: 'avenged sevenfold waking the fallen',
      choosingId,
      candidates: [candidate('album', 'Waking the Fallen'), candidate('comp', 'Waking the Fallen / Sounding the Seventh Trumpet')]
    },
    // The real Listbox, not a stub: what is under test is its focus and selection wiring.
    global: { plugins: [[PrimeVue, { unstyled: true }]], stubs: { Listbox: false } }
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
})
