import { invoke } from '@tauri-apps/api/core'

export const genresApi = {
  async getAll(): Promise<string[]> {
    return invoke<string[]>('genres_list')
  }
}

export const countriesApi = {
  /** Only the countries present in the library; filtering by any other returns nothing. */
  async getAll(): Promise<string[]> {
    return invoke<string[]>('countries_list')
  }
}
