import { describe, it, expect, beforeEach, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'

const takeLaunchAction = vi.fn()
const unlisten = vi.fn()
let emitQuickAdd: () => void = () => {}
const onQuickAdd = vi.fn(async (callback: () => void) => {
  emitQuickAdd = callback
  return unlisten
})
vi.mock('@/api/app', () => ({ appApi: { takeLaunchAction, onQuickAdd } }))

const { useLaunchActions } = await import('./useLaunchActions')
const { useQuickAddPalette } = await import('./useQuickAddPalette')

function mountHost() {
  const Host = defineComponent({
    setup() {
      useLaunchActions()
      return () => h('div')
    }
  })
  return mount(Host)
}

beforeEach(() => {
  vi.clearAllMocks()
  useQuickAddPalette().open.value = false
  takeLaunchAction.mockResolvedValue(null)
})

describe('useLaunchActions', () => {
  it('opens quick add when the app was started with --quick-add', async () => {
    takeLaunchAction.mockResolvedValue('quick-add')
    mountHost()
    await flushPromises()
    expect(useQuickAddPalette().open.value).toBe(true)
  })

  it('leaves the palette closed on an ordinary start', async () => {
    mountHost()
    await flushPromises()
    expect(useQuickAddPalette().open.value).toBe(false)
  })

  it('opens quick add when a later launch or the tray asks', async () => {
    mountHost()
    await flushPromises()
    emitQuickAdd()
    expect(useQuickAddPalette().open.value).toBe(true)
  })

  it('stops listening once unmounted', async () => {
    const wrapper = mountHost()
    await flushPromises()
    wrapper.unmount()
    expect(unlisten).toHaveBeenCalledOnce()
  })

  it('does nothing outside Tauri rather than breaking the page', async () => {
    onQuickAdd.mockRejectedValueOnce(new Error('no Tauri'))
    mountHost()
    await flushPromises()
    expect(useQuickAddPalette().open.value).toBe(false)
  })
})
