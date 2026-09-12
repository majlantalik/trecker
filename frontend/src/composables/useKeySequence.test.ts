import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { useKeySequence } from './useKeySequence'

function arm(timeoutMs = 1500) {
  const q = vi.fn()
  const l = vi.fn()
  const Host = defineComponent({
    setup() {
      const { pending } = useKeySequence('g', { q, l }, timeoutMs)
      return () => h('div', { 'data-pending': String(pending.value) })
    }
  })
  const wrapper = mount(Host, { attachTo: document.body })
  return {
    q,
    l,
    pending: () => wrapper.find('div').attributes('data-pending') === 'true',
    wrapper,
    unmount: () => wrapper.unmount()
  }
}

function press(key: string, init: KeyboardEventInit & { target?: EventTarget } = {}) {
  const event = new KeyboardEvent('keydown', { key, ...init, bubbles: true })
  const target = (init as { target?: EventTarget }).target
  ;(target ?? window).dispatchEvent(event)
}

describe('useKeySequence', () => {
  beforeEach(() => vi.useFakeTimers())
  afterEach(() => vi.useRealTimers())

  it('runs the handler for the second key', () => {
    const { q, unmount } = arm()
    press('g')
    press('q')
    expect(q).toHaveBeenCalledTimes(1)
    unmount()
  })

  it('does nothing on the second key alone', () => {
    const { q, unmount } = arm()
    press('q')
    expect(q).not.toHaveBeenCalled()
    unmount()
  })

  it('reports a half-entered sequence so the interface can show it', async () => {
    const { pending, wrapper, unmount } = arm()
    expect(pending()).toBe(false)
    press('g')
    await wrapper.vm.$nextTick()
    expect(pending()).toBe(true)
    press('q')
    await wrapper.vm.$nextTick()
    expect(pending()).toBe(false)
    unmount()
  })

  it('expires the prefix rather than swallowing a later keystroke', () => {
    // Without this, a stray "g" would eat the next key pressed minutes afterwards, which
    // reads as the app randomly ignoring input.
    const { q, unmount } = arm(1000)
    press('g')
    vi.advanceTimersByTime(1200)
    press('q')
    expect(q).not.toHaveBeenCalled()
    unmount()
  })

  it('ends the sequence on an unmapped second key', () => {
    const { q, unmount } = arm()
    press('g')
    press('z')
    press('q')
    expect(q).not.toHaveBeenCalled()
    unmount()
  })

  it('stays out of the way while typing', () => {
    const input = document.createElement('input')
    document.body.appendChild(input)
    const { q, unmount } = arm()
    press('g', { target: input })
    press('q', { target: input })
    expect(q).not.toHaveBeenCalled()
    unmount()
    input.remove()
  })

  it('ignores chords that happen to use the prefix letter', () => {
    const { q, unmount } = arm()
    press('g', { ctrlKey: true })
    press('q')
    expect(q).not.toHaveBeenCalled()
    unmount()
  })

  it('is case-insensitive, so Caps Lock does not break it', () => {
    const { l, unmount } = arm()
    press('G')
    press('L')
    expect(l).toHaveBeenCalledTimes(1)
    unmount()
  })

  it('forgets a half-entered sequence when the window loses focus', () => {
    const { q, unmount } = arm()
    press('g')
    window.dispatchEvent(new Event('blur'))
    press('q')
    expect(q).not.toHaveBeenCalled()
    unmount()
  })

  it('stops listening once unmounted', () => {
    const { q, unmount } = arm()
    unmount()
    press('g')
    press('q')
    expect(q).not.toHaveBeenCalled()
  })
})
