import { invoke } from '@tauri-apps/api/core'
import type { DbInfo } from '@/types'

export const settingsApi = {
  async dbInfo(): Promise<DbInfo> {
    return invoke<DbInfo>('settings_db_info')
  }
}
