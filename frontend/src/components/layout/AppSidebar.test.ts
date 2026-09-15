import { describe, it, expect, vi, beforeEach } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'

vi.mock('@/api/releases', () => ({
  releasesApi: { getAll: vi.fn().mockResolvedValue({ totalElements: 0, content: [] }) }
}))
vi.mock('@/api/artists', () => ({ artistsApi: { list: vi.fn().mockResolvedValue([]) } }))

const { default: AppSidebar } = await import('./AppSidebar.vue')
const { default: router } = await import('@/router')
const { useShortcutsHelp } = await import('@/composables/useShortcutsHelp')
const { useShortcut } = await import('@/composables/useShortcut')

beforeEach(() => {
  useShortcutsHelp().open.value = false
})

function mountSidebar() {
  return mount(AppSidebar, {
    global: { plugins: [router], directives: { tooltip: {} } }
  })
}

describe('the keyboard shortcuts button', () => {
  it('is in the sidebar, so the list can be found without knowing the key', () => {
    expect(mountSidebar().find('button[aria-label="Keyboard shortcuts"]').exists()).toBe(true)
  })

  it('opens the shortcuts list', async () => {
    await mountSidebar().find('button[aria-label="Keyboard shortcuts"]').trigger('click')
    expect(useShortcutsHelp().open.value).toBe(true)
  })

  it('opens the same list the ? key toggles', async () => {
    // What ShortcutsHelp wires up. If the button and the key held separate state, one
    // would open a dialog the other could not close.
    const Help = defineComponent({
      setup() {
        useShortcut('?', useShortcutsHelp().toggle)
        return () => h('div')
      }
    })
    const help = mount(Help, { attachTo: document.body })

    await mountSidebar().find('button[aria-label="Keyboard shortcuts"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '?', shiftKey: true, bubbles: true }))
    expect(useShortcutsHelp().open.value).toBe(false)

    help.unmount()
  })
})
