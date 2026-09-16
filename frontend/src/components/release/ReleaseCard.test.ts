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
  it('opens on a click and on Enter', async () => {
    const w = mountCard()
    await w.find('.release-card').trigger('click')
    await w.find('.release-card').trigger('keydown', { key: 'Enter' })
    expect(w.emitted('click')).toEqual([[release], [release]])
  })

  it('does not open from its own action buttons', async () => {
    const w = mountCard()
    await w.find('.action').trigger('click')
    await w.find('.action').trigger('keydown', { key: 'Enter' })
    expect(w.emitted('click')).toBeUndefined()
  })
})
