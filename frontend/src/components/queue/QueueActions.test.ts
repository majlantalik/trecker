import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import QueueActions from './QueueActions.vue'
import type { Release } from '@/types'

const add = vi.fn()
vi.mock('primevue/usetoast', () => ({ useToast: () => ({ add }) }))

const writeText = vi.fn()

const release = {
  id: 'r1',
  artist: 'Slint',
  title: 'Spiderland',
  streamingLinks: { spotify: 'https://open.spotify.com/album/1' }
} as unknown as Release

beforeEach(() => {
  vi.clearAllMocks()
  writeText.mockResolvedValue(undefined)
  Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
})

function mountActions() {
  return mount(QueueActions, { props: { release }, global: { directives: { tooltip: {} } } })
}

describe('queue row actions', () => {
  it('copies the album without leaving the queue', async () => {
    const w = mountActions()

    await w.find('[aria-label="Copy artist and album"]').trigger('click')

    expect(writeText).toHaveBeenCalledWith('Slint - Spiderland')
    // Copying is the one action that stays here: the queue hears nothing about it.
    expect(w.emitted('log')).toBeUndefined()
    expect(w.emitted('delete')).toBeUndefined()
  })

  it('hands logging and removing to the queue', async () => {
    const w = mountActions()

    await w.find('[data-label="Log"]').trigger('click')
    await w.find('[aria-label="Remove from queue"]').trigger('click')

    expect(w.emitted('log')).toEqual([[release]])
    expect(w.emitted('delete')).toEqual([[release]])
  })
})
