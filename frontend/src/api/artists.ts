import { invoke } from '@tauri-apps/api/core'
import type {
  Artist,
  ArtistAddRequest,
  ArtistCandidate,
  ArtistUpdateRequest,
  DiscographyEntry
} from '@/types'

export const artistsApi = {
  /** Up to ten artists matching typed text, as MusicBrainz ranks them. */
  async search(query: string): Promise<ArtistCandidate[]> {
    return invoke<ArtistCandidate[]>('artists_search', { query })
  },

  /** Looks the artist up and puts them on your list. Already there: returns them as they are. */
  async add(request: ArtistAddRequest): Promise<Artist> {
    return invoke<Artist>('artists_add', { request })
  },

  /** Artists to check first, then checked ones. */
  async list(): Promise<Artist[]> {
    return invoke<Artist[]>('artists_list')
  },

  async get(id: string): Promise<Artist> {
    return invoke<Artist>('artists_get', { id })
  },

  async update(id: string, request: ArtistUpdateRequest): Promise<Artist> {
    return invoke<Artist>('artists_update', { id, request })
  },

  async delete(id: string): Promise<void> {
    return invoke<void>('artists_delete', { id })
  },

  /** Albums and EPs from MusicBrainz, oldest first, each marked with your library's copy. */
  async discography(id: string): Promise<DiscographyEntry[]> {
    return invoke<DiscographyEntry[]>('artists_discography', { id })
  }
}
