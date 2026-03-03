import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import type { Release } from '@/types'

// Mock router and toast before component import
const mockRouterPush = vi.fn()
const mockToastAdd = vi.fn()

vi.mock('vue-router', () => ({
  useRouter: vi.fn(() => ({ push: mockRouterPush }))
}))

vi.mock('primevue/usetoast', () => ({
  useToast: vi.fn(() => ({ add: mockToastAdd }))
}))

// Mock the stores API layer so we control what markAsListened returns
const mockMarkAsListened = vi.fn()
const mockAddGenre = vi.fn()

vi.mock('@/stores/releases', () => ({
  useReleasesStore: vi.fn(() => ({
    markAsListened: mockMarkAsListened
  }))
}))

vi.mock('@/stores/genres', () => ({
  useGenresStore: vi.fn(() => ({
    addGenre: mockAddGenre
  }))
}))

const { default: QuickLogModal } = await import('./QuickLogModal.vue')

function makeRelease(overrides: Partial<Release> = {}): Release {
  return {
    id: 'rel-1',
    artist: 'Test Artist',
    title: 'Test Album',
    releaseYear: 2023,
    albumArtUrl: null,
    status: 'QUEUED',
    discoveryLink: null,
    streamingLinks: {},
    country: 'US',
    rating: 4,
    didNotFinish: false,
    dateListened: null,
    notes: 'Some notes',
    createdAt: '2024-01-01T00:00:00Z',
    genres: ['Jazz', 'Blues'],
    ...overrides
  }
}

describe('QuickLogModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('syncs form fields when release prop changes', async () => {
    const release = makeRelease()
    const wrapper = mount(QuickLogModal, {
      props: { release: null, visible: true }
    })

    await flushPromises()

    // Set the release prop to trigger the watch
    await wrapper.setProps({ release })
    await flushPromises()

    // The form should have the release's country value.
    // Stubs render InputText as <input :value="modelValue" />, country is the first text input.
    const inputs = wrapper.findAll('input')
    const textInputs = inputs.filter(i => (i.element as HTMLInputElement).type !== 'checkbox')
    // textInputs[0] = country input (first InputText in the form)
    expect(textInputs[0].element.value).toBe('US')
  })

  it('"Log it" button emits logged event and calls markAsListened', async () => {
    const release = makeRelease()
    const updated = makeRelease({ status: 'LISTENED' })
    mockMarkAsListened.mockResolvedValue(updated)

    const wrapper = mount(QuickLogModal, {
      props: { release, visible: true }
    })

    await flushPromises()

    // Click the "Log it" button
    const buttons = wrapper.findAll('button')
    const logButton = buttons.find(b => b.element.dataset.label === 'Log it')
    await logButton?.trigger('click')
    await flushPromises()

    expect(mockMarkAsListened).toHaveBeenCalledWith('rel-1', expect.objectContaining({
      didNotFinish: false
    }))
    const emitted = wrapper.emitted('logged')
    expect(emitted).toBeTruthy()
  })

  it('"Log + Details" button emits logged and navigates to entry route', async () => {
    const release = makeRelease()
    const updated = makeRelease({ status: 'LISTENED' })
    mockMarkAsListened.mockResolvedValue(updated)

    const wrapper = mount(QuickLogModal, {
      props: { release, visible: true }
    })

    await flushPromises()

    const buttons = wrapper.findAll('button')
    const logDetailsButton = buttons.find(b => b.element.dataset.label === 'Log + Details')
    await logDetailsButton?.trigger('click')
    await flushPromises()

    expect(mockRouterPush).toHaveBeenCalledWith({ name: 'entry', params: { id: 'rel-1' } })
    expect(wrapper.emitted('logged')).toBeTruthy()
  })
})
