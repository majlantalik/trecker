import { invoke } from '@tauri-apps/api/core'
import type {
  Release,
  ReleaseRequest,
  ReleaseUpdateRequest,
  ResolveRequest,
  ResolvedMetadata,
  PageResponse,
  ReleaseFilterParams
} from '@/types'

export const releasesApi = {
  async resolve(request: ResolveRequest): Promise<ResolvedMetadata> {
    return invoke<ResolvedMetadata>('releases_resolve', { request })
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

  async refreshMetadata(id: string): Promise<Release> {
    return invoke<Release>('releases_refresh_metadata', { id })
  },

  async searchCatalog(q: string, limit = 10): Promise<ResolvedMetadata[]> {
    return invoke<ResolvedMetadata[]>('releases_search_catalog', { q, limit })
  }
}
