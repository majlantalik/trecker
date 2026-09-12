import { invoke } from '@tauri-apps/api/core'
import type { DbInfo } from '@/types'

export const infoApi = {
  async dbInfo(): Promise<DbInfo> {
    return invoke<DbInfo>('info_db')
  }
}
