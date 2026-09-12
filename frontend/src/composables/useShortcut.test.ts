import { describe, it, expect, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { useShortcut, isTyping } from './useShortcut'
import { SHORTCUTS, SHORTCUT_GROUPS } from './shortcuts'
import router from '@/router'

function arm(combo: string, options?: { whileTyping?: boolean }) {
  const spy = vi.fn()
  const Host = defineComponent({
    setup() {
      useShortcut(combo, spy, options)
      return () => h('div')
    }
  })
  const wrapper = mount(Host, { attachTo: document.body })
  return { spy, unmount: () => wrapper.unmount() }
}

function press(init: KeyboardEventInit & { target?: EventTarget }) {
  const event = new KeyboardEvent('keydown', { ...init, bubbles: true })
  const target = (init as { target?: EventTarget }).target
  ;(target ?? window).dispatchEvent(event)
}

describe('useShortcut', () => {
  it('fires on the combination it was given', () => {
    const { spy, unmount } = arm('ctrl+k')
    press({ key: 'k', ctrlKey: true })
    expect(spy).toHaveBeenCalledTimes(1)
    unmount()
  })

  it('treats Command as Ctrl, so one binding covers macOS', () => {
    const { spy, unmount } = arm('ctrl+k')
    press({ key: 'k', metaKey: true })
    expect(spy).toHaveBeenCalledTimes(1)
    unmount()
  })

  it('does not fire on the bare key', () => {
    const { spy, unmount } = arm('ctrl+k')
    press({ key: 'k' })
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('does not fire when an extra modifier is held', () => {
    const { spy, unmount } = arm('ctrl+k')
    press({ key: 'k', ctrlKey: true, altKey: true })
    expect(spy).not.toHaveBeenCalled()
    unmount()
  })

  it('ignores keystrokes aimed at a text field by default', () => {
    // This guard is what makes single-character shortcuts safe to have at all.
    const input = document.createElement('input')
    document.body.appendChild(input)
    const { spy, unmount } = arm('?')
    press({ key: '?', target: input })
    expect(spy).not.toHaveBeenCalled()
    unmount()
    input.remove()
  })

  it('fires inside a text field when asked to', () => {
    const input = document.createElement('input')
    document.body.appendChild(input)
    const { spy, unmount } = arm('ctrl+k', { whileTyping: true })
    press({ key: 'k', ctrlKey: true, target: input })
    expect(spy).toHaveBeenCalledTimes(1)
    unmount()
    input.remove()
  })

  it('stops listening once unmounted', () => {
    const { spy, unmount } = arm('ctrl+k')
    unmount()
    press({ key: 'k', ctrlKey: true })
    expect(spy).not.toHaveBeenCalled()
  })
})

describe('isTyping', () => {
  it('recognises the fields a person types into', () => {
    for (const tag of ['input', 'textarea', 'select']) {
      expect(isTyping(document.createElement(tag))).toBe(true)
    }
    expect(isTyping(document.createElement('div'))).toBe(false)
    expect(isTyping(null)).toBe(false)
  })
})

describe('the shortcut catalogue', () => {
  it('puts every shortcut in a group the help dialog renders', () => {
    // The help dialog iterates SHORTCUT_GROUPS, so a shortcut in an unlisted group would
    // exist in the app and be invisible in the documentation.
    for (const s of SHORTCUTS) {
      expect(SHORTCUT_GROUPS).toContain(s.group)
    }
  })

  it('promises only routes the router actually has', () => {
    // The catalogue says "Go to Queue"; if the route moved, the help would lie. This is
    // the same class of drift the Settings-to-Info rename could have caused.
    const routes = router.getRoutes().map((r) => r.path)
    for (const path of ['/queue', '/library', '/stats', '/info']) {
      expect(routes).toContain(path)
    }
    const navKeys = SHORTCUTS.filter((s) => s.group === 'Navigation').map((s) => s.keys[1])
    expect(navKeys).toEqual(['Q', 'L', 'S', 'I'])
  })

  it('describes every shortcut', () => {
    for (const s of SHORTCUTS) {
      expect(s.keys.length).toBeGreaterThan(0)
      expect(s.description.trim()).not.toBe('')
    }
  })
})
