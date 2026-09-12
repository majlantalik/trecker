import { ref } from 'vue'
import { settingsApi } from '@/api/settings'
import type { Settings } from '@/types'

/**
 * The Settings view's state. Each change saves at once, and a change the app cannot apply,
 * such as a tray icon on a desktop without a tray, is undone on screen and explained
 * rather than left looking saved.
 */
export function useSettings() {
  const settings = ref<Settings | null>(null)
  const saving = ref(false)
  const error = ref('')

  async function load() {
    try {
      settings.value = await settingsApi.get()
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    }
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

  return { settings, saving, error, load, update }
}
