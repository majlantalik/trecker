import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ReleaseCard from './ReleaseCard.vue'
import type { Release } from '@/types'

const release = {
  id: 'r1',
  artist: 'Sigur Rós',
  title: 'Ágætis byrjun',
  genres: [],
  albumArtUrl: null,
  rating: null
} as unknown as Release

function mountCard() {
  return mount(ReleaseCard, {
    props: { release },
    slots: { actions: '<button class="action">Log</button>' }
  })
}

describe('opening a release card', () => {
  it('opens from its title button, which covers the card and names the release', async () => {
    const w = mountCard()
    const open = w.find('button.card-open')
    expect(open.attributes('aria-label')).toBe('Open Sigur Rós – Ágætis byrjun')
    await open.trigger('click')
    expect(w.emitted('click')).toEqual([[release]])
  })

  it('does not open from its own action buttons', async () => {
    const w = mountCard()
    await w.find('.action').trigger('click')
    expect(w.emitted('click')).toBeUndefined()
  })
})
