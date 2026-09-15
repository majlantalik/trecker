import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import PrimeVue from 'primevue/config'

vi.mock('@/api/genres', () => ({
  genresApi: { getAll: vi.fn().mockResolvedValue(['britpop', 'indie rock', 'shoegaze']) }
}))

const { default: GenreTagInput } = await import('./GenreTagInput.vue')

async function mountInput(modelValue: string[] = []) {
  const wrapper = mount(GenreTagInput, {
    props: {
      modelValue,
      'onUpdate:modelValue': (value: string[]) => wrapper.setProps({ modelValue: value })
    },
    attachTo: document.body,
    // The real AutoComplete: what is under test is how typing and keys become chips.
    global: { plugins: [[PrimeVue, { unstyled: true }]], stubs: { AutoComplete: false, GenreTagInput: false } }
  })
  await flushPromises()
  return wrapper
}

async function type(wrapper: Awaited<ReturnType<typeof mountInput>>, text: string, key: string) {
  const input = wrapper.find('input')
  ;(input.element as HTMLInputElement).value = text
  await input.trigger('input')
  await input.trigger('keydown', { key })
}

beforeEach(() => vi.useRealTimers())

describe('GenreTagInput', () => {
  it('turns typed text into a chip on Enter, written the way the library writes it', async () => {
    const wrapper = await mountInput(['britpop'])
    await type(wrapper, 'Indie Rock', 'Enter')
    expect(wrapper.props('modelValue')).toEqual(['britpop', 'indie rock'])
    expect((wrapper.find('input').element as HTMLInputElement).value).toBe('')
  })

  it('turns typed text into a chip on a comma', async () => {
    const wrapper = await mountInput()
    await type(wrapper, 'madchester', ',')
    expect(wrapper.props('modelValue')).toEqual(['madchester'])
  })

  it('does not let Enter submit a surrounding form', async () => {
    const submitted = vi.fn()
    document.addEventListener('keydown', submitted)
    const wrapper = await mountInput()
    await type(wrapper, 'shoegaze', 'Enter')
    expect(submitted).not.toHaveBeenCalled()
    document.removeEventListener('keydown', submitted)
    wrapper.unmount()
  })

  it('splits a pasted list into chips, once each', async () => {
    const wrapper = await mountInput(['britpop'])
    const paste = new Event('paste', { bubbles: true, cancelable: true }) as ClipboardEvent
    Object.defineProperty(paste, 'clipboardData', { value: { getData: () => 'indie rock, Britpop, dream pop' } })
    wrapper.find('input').element.dispatchEvent(paste)
    await flushPromises()
    expect(wrapper.props('modelValue')).toEqual(['britpop', 'indie rock', 'dream pop'])
    expect(paste.defaultPrevented).toBe(true)
  })

  it('keeps a half-typed genre when focus leaves the field', async () => {
    vi.useFakeTimers()
    const wrapper = await mountInput()
    const input = wrapper.find('input')
    ;(input.element as HTMLInputElement).value = 'post-punk'
    await input.trigger('focusout')
    vi.advanceTimersByTime(200)
    await flushPromises()
    expect(wrapper.props('modelValue')).toEqual(['post-punk'])
  })

  it('shows each genre as a chip that can be removed', async () => {
    const wrapper = await mountInput(['britpop', 'indie rock'])
    expect(wrapper.findAll('.genre-chip').map((c) => c.text())).toEqual(['britpop', 'indie rock'])
    await wrapper.find('button[aria-label="Remove britpop"]').trigger('click')
    expect(wrapper.props('modelValue')).toEqual(['indie rock'])
  })

  it('ignores Enter with nothing typed', async () => {
    const wrapper = await mountInput(['britpop'])
    await type(wrapper, '   ', 'Enter')
    expect(wrapper.props('modelValue')).toEqual(['britpop'])
  })
})
