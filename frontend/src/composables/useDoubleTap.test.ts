import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { useDoubleTap } from './useDoubleTap'

/** Mounts a component that arms the double-tap, and returns the spy plus an unmount. */
function arm(windowMs = 400) {
  const spy = vi.fn()
  const Host = defineComponent({
    setup() {
      useDoubleTap('Shift', spy, windowMs)
      return () => h('div')
    }
  })
  const wrapper = mount(Host, { attachTo: document.body })
  return { spy, unmount: () => wrapper.unmount() }
}

function press(key: string, init: Partial<KeyboardEventInit> = {}) {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, ...init }))
}

describe('useDoubleTap', () => {
  let now = 0

  beforeEach(() => {
    now = 1000
    vi.spyOn(performance, 'now').mockImplementation(() => now)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('fires on two taps inside the window', () => {
    const { spy, unmount } = arm()
    press('Shift')
    now += 150
    press('Shift')
    expect(spy).toHaveBeenCalledTimes(1)
    unmount()
  })

  it('does not fire when the taps are too far apart', () => {
    const { spy, unmount } = arm()
    press('Shift')
    now += 900
    press('Shift')
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('does not fire while typing capital letters', () => {
    // Typing "AB" is Shift, A, Shift, B. The letters in between must cancel the tap, or
    // every capitalised word would open the palette.
    const { spy, unmount } = arm()
    press('Shift')
    press('a', { shiftKey: true })
    now += 100
    press('Shift')
    press('b', { shiftKey: true })
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('ignores auto-repeat from a held key', () => {
    // Holding Shift emits a stream of keydown events, which would otherwise read as a
    // double tap within milliseconds.
    const { spy, unmount } = arm()
    press('Shift')
    now += 30
    press('Shift', { repeat: true })
    now += 30
    press('Shift', { repeat: true })
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('ignores chords that merely start with the key', () => {
    // Ctrl+Shift+P begins with a Shift press; pressing two such chords must not fire.
    const { spy, unmount } = arm()
    press('Shift', { ctrlKey: true })
    now += 100
    press('Shift', { ctrlKey: true })
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('needs three taps to fire twice, not two', () => {
    const { spy, unmount } = arm()
    press('Shift')
    now += 100
    press('Shift')
    expect(spy).toHaveBeenCalledTimes(1)

    // The pair is consumed; a third tap starts a new pair rather than completing one.
    now += 100
    press('Shift')
    expect(spy).toHaveBeenCalledTimes(1)

    now += 100
    press('Shift')
    expect(spy).toHaveBeenCalledTimes(2)
    unmount()
  })

  it('forgets a pending tap when the window loses focus', () => {
    // Otherwise a tap left armed on the way out fires on a single press coming back.
    const { spy, unmount } = arm()
    press('Shift')
    window.dispatchEvent(new Event('blur'))
    now += 100
    press('Shift')
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('stops listening once unmounted', () => {
    const { spy, unmount } = arm()
    unmount()
    press('Shift')
    now += 100
    press('Shift')
    expect(spy).not.toHaveBeenCalled()
  })
})
