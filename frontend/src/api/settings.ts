import { invoke } from '@tauri-apps/api/core'
import type { Settings } from '@/types'

export const settingsApi = {
  async get(): Promise<Settings> {
    return invoke<Settings>('settings_get')
  },

  /** Applies and saves. Rejects without saving when the change cannot be applied. */
  async update(request: Settings): Promise<Settings> {
    return invoke<Settings>('settings_update', { request })
  }
}
