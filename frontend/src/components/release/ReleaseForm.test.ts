import { describe, it, expect, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ReleaseForm from './ReleaseForm.vue'
import type { Release, ResolvedMetadata } from '@/types'

function makePrefill(overrides: Partial<ResolvedMetadata> = {}): ResolvedMetadata {
  return {
    artist: 'Prefilled Artist',
    title: 'Prefilled Title',
    releaseYear: 2023,
    albumArtUrl: null,
    country: 'US',
    streamingLinks: { spotify: 'https://open.spotify.com/album/abc' },
    genres: ['Jazz', 'Blues'],
    spotifyId: 'abc123',
    musicbrainzId: undefined,
    ...overrides
  }
}

function makeRelease(overrides: Partial<Release> = {}): Release {
  return {
    id: 'rel-1',
    artist: 'Edit Artist',
    title: 'Edit Title',
    releaseYear: 2020,
    albumArtUrl: null,
    status: 'QUEUED',
    discoveryLink: null,
    streamingLinks: { tidal: 'https://tidal.com/browse/album/123' },
    country: 'GB',
    rating: null,
    didNotFinish: false,
    dateListened: null,
    notes: null,
    createdAt: '2024-01-01T00:00:00Z',
    genres: ['Rock'],
    ...overrides
  }
}

describe('ReleaseForm', () => {
  describe('prefill mode (add to queue)', () => {
    it('populates form fields from prefill when dialog opens', async () => {
      const prefill = makePrefill()
      const wrapper = mount(ReleaseForm, {
        props: {
          visible: false,
          prefill
        }
      })

      // Open the dialog (triggers the watch)
      await wrapper.setProps({ visible: true })
      await flushPromises()

      // Verify artist and title inputs have the prefill values
      const inputs = wrapper.findAll('input')
      const artistInput = inputs[0]
      const titleInput = inputs[1]
      expect(artistInput.element.value).toBe('Prefilled Artist')
      expect(titleInput.element.value).toBe('Prefilled Title')
    })

    it('auto-populates streaming link when prefill has streamingLinks', async () => {
      const prefill = makePrefill({ streamingLinks: { spotify: 'https://open.spotify.com/album/xyz' } })
      const wrapper = mount(ReleaseForm, {
        props: { visible: false, prefill }
      })

      await wrapper.setProps({ visible: true })
      await flushPromises()

      // The streaming link input should have the first streaming link value
      const inputs = wrapper.findAll('input')
      const streamingLinkInput = inputs.find(i =>
        (i.element as HTMLInputElement).value?.includes('spotify.com'))
      expect(streamingLinkInput).toBeDefined()
    })
  })

  describe('edit mode', () => {
    it('populates form fields from release when dialog opens in edit mode', async () => {
      const release = makeRelease()
      const wrapper = mount(ReleaseForm, {
        props: { visible: false, release }
      })

      await wrapper.setProps({ visible: true })
      await flushPromises()

      const inputs = wrapper.findAll('input')
      const artistInput = inputs[0]
      const titleInput = inputs[1]
      expect(artistInput.element.value).toBe('Edit Artist')
      expect(titleInput.element.value).toBe('Edit Title')
    })
  })

  describe('form submission', () => {
    it('emits submit event with correct payload when form is valid', async () => {
      const prefill = makePrefill({ genres: [] })
      const wrapper = mount(ReleaseForm, {
        props: { visible: false, prefill }
      })
      // Open dialog to trigger watch and populate form
      await wrapper.setProps({ visible: true })
      await flushPromises()

      await wrapper.find('form').trigger('submit')
      await flushPromises()

      const emitted = wrapper.emitted('submit')
      expect(emitted).toBeTruthy()
      expect(emitted![0][0]).toMatchObject({
        artist: 'Prefilled Artist',
        title: 'Prefilled Title',
        spotifyId: 'abc123'
      })
    })

    it('does not emit submit when artist or title is empty', async () => {
      // Mount with visible: false (no prefill) then open — form stays empty
      const wrapper = mount(ReleaseForm, {
        props: { visible: false }
      })
      await wrapper.setProps({ visible: true })
      await flushPromises()

      await wrapper.find('form').trigger('submit')
      await flushPromises()

      expect(wrapper.emitted('submit')).toBeFalsy()
    })

    it('detects spotify URL and sets service key in streaming links', async () => {
      const prefill = makePrefill({
        streamingLinks: { spotify: 'https://open.spotify.com/album/abc123' },
        genres: []
      })
      const wrapper = mount(ReleaseForm, {
        props: { visible: false, prefill }
      })
      await wrapper.setProps({ visible: true })
      await flushPromises()

      await wrapper.find('form').trigger('submit')
      await flushPromises()

      const payload = wrapper.emitted('submit')![0][0]
      expect(payload.streamingLinks).toHaveProperty('spotify')
    })
  })
})
