import { ref } from 'vue'
import { settingsApi } from '@/api/settings'
import type { Settings } from '@/types'

// One copy for the whole app, so a change saved in the Settings view reaches every open-link
// button and the queue's order without either reading the file again.
const settings = ref<Settings | null>(null)
let loading: Promise<void> | null = null

/**
 * The settings file's contents. Each change saves at once, and a change the app cannot apply,
 * such as a tray icon on a desktop without a tray, is undone on screen and explained rather
 * than left looking saved.
 */
export function useSettings() {
  const saving = ref(false)
  const error = ref('')

  async function load() {
    try {
      settings.value = await settingsApi.get()
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    }
  }

  /**
   * Loads unless another caller already has. For components that appear many times at once,
   * such as a button on every library row, which would otherwise each read the file.
   */
  function loadOnce(): Promise<void> {
    if (settings.value) return Promise.resolve()
    loading ??= load().finally(() => {
      loading = null
    })
    return loading
  }

  async function update(change: Partial<Settings>) {
    if (!settings.value) return
    const previous = settings.value
    settings.value = { ...previous, ...change }
    error.value = ''
    saving.value = true
    try {
      settings.value = await settingsApi.update(settings.value)
    } catch (e: any) {
      settings.value = previous
      error.value = e?.message ?? String(e)
    } finally {
      saving.value = false
    }
  }

  return { settings, saving, error, load, loadOnce, update }
}
