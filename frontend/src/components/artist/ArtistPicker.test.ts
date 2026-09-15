import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import PrimeVue from 'primevue/config'
import type { ArtistCandidate } from '@/types'

const { default: ArtistPicker } = await import('./ArtistPicker.vue')

function candidate(id: string, name: string): ArtistCandidate {
  return {
    musicbrainzArtistId: id,
    name,
    disambiguation: id === 'uk' ? 'UK punk band' : null,
    artistType: 'Group',
    country: id === 'uk' ? 'GB' : 'US',
    beginYear: 1999,
    endYear: null,
    tags: ['metal', 'rock']
  }
}

function mountPicker(addingId: string | null = null) {
  return mount(ArtistPicker, {
    props: {
      visible: true,
      query: 'the band',
      addingId,
      candidates: [candidate('us', 'The Band'), candidate('uk', 'The Band')]
    },
    global: { plugins: [[PrimeVue, { unstyled: true }]], stubs: { Listbox: false } }
  })
}

describe('ArtistPicker', () => {
  it('tells same-named artists apart', () => {
    const text = mountPicker().text()
    expect(text).toContain('UK punk band')
    expect(text).toContain('Since 1999')
    expect(text).toContain('metal, rock')
  })

  it('marks the list for the dialog to focus when it opens', () => {
    expect(mountPicker().find('[role="listbox"]').attributes('autofocus')).toBeDefined()
  })

  it('hands back the chosen artist', async () => {
    const picker = mountPicker()
    await picker.findAll('[role="option"]')[1].trigger('click')
    expect(picker.emitted('choose')?.[0]).toEqual([candidate('uk', 'The Band')])
  })

  it('ignores a pick while another artist is being added', async () => {
    const picker = mountPicker('us')
    await picker.findAll('[role="option"]')[1].trigger('click')
    expect(picker.emitted('choose')).toBeUndefined()
  })
})
