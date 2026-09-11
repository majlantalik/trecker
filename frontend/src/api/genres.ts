import { invoke } from '@tauri-apps/api/core'

export const genresApi = {
  async getAll(): Promise<string[]> {
    return invoke<string[]>('genres_list')
  }
}
