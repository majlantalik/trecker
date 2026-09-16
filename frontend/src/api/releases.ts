import { invoke } from '@tauri-apps/api/core'
import type {
  Release,
  ReleaseRequest,
  ReleaseUpdateRequest,
  ResolveRequest,
  ResolvedMetadata,
  AlbumCandidate,
  UnlinkedRelease,
  PageResponse,
  ReleaseFilterParams
} from '@/types'

export const releasesApi = {
  async resolve(request: ResolveRequest): Promise<ResolvedMetadata> {
    return invoke<ResolvedMetadata>('releases_resolve', { request })
  },

  /** Up to ten albums matching typed text, most likely first. */
  async search(query: string): Promise<AlbumCandidate[]> {
    return invoke<AlbumCandidate[]>('releases_search', { query })
  },

  /** Genres, country and cover for an album chosen from a search. */
  async lookup(releaseGroupId: string): Promise<ResolvedMetadata> {
    return invoke<ResolvedMetadata>('releases_lookup', { releaseGroupId })
  },

  async create(request: ReleaseRequest): Promise<Release> {
    return invoke<Release>('releases_create', { request })
  },

  async getAll(params: ReleaseFilterParams = {}): Promise<PageResponse<Release>> {
    return invoke<PageResponse<Release>>('releases_list', { params })
  },

  async getRandom(): Promise<Release> {
    return invoke<Release>('releases_random')
  },

  async getById(id: string): Promise<Release> {
    return invoke<Release>('releases_get', { id })
  },

  async update(id: string, request: ReleaseUpdateRequest): Promise<Release> {
    return invoke<Release>('releases_update', { id, request })
  },

  async delete(id: string): Promise<void> {
    await invoke<void>('releases_delete', { id })
  },

  /** Refreshes a linked album by its id. One with no id fails with code NOT_LINKED. */
  async refreshMetadata(id: string): Promise<Release> {
    return invoke<Release>('releases_refresh_metadata', { id })
  },

  /**
   * Links an album to the one chosen from a search and fills in its metadata. Fails with
   * code ALREADY_IN_LIBRARY, and the `releaseId` to open, when you already track that album.
   */
  async link(id: string, releaseGroupId: string): Promise<Release> {
    return invoke<Release>('releases_link', { id, releaseGroupId })
  },

  /** Every tracked album with no MusicBrainz id, oldest addition first. */
  async unlinked(): Promise<UnlinkedRelease[]> {
    return invoke<UnlinkedRelease[]>('releases_unlinked')
  },

  async searchCatalog(q: string, limit = 10): Promise<ResolvedMetadata[]> {
    return invoke<ResolvedMetadata[]>('releases_search_catalog', { q, limit })
  }
}
