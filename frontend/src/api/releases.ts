import api from './axios'
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
    const { data } = await api.post<ResolvedMetadata>('/releases/resolve', request)
    return data
  },

  async create(request: ReleaseRequest): Promise<Release> {
    const { data } = await api.post<Release>('/releases', request)
    return data
  },

  async getAll(params: ReleaseFilterParams = {}): Promise<PageResponse<Release>> {
    const { data } = await api.get<PageResponse<Release>>('/releases', { params })
    return data
  },

  async getRandom(): Promise<Release> {
    const { data } = await api.get<Release>('/releases/random')
    return data
  },

  async getById(id: string): Promise<Release> {
    const { data } = await api.get<Release>(`/releases/${id}`)
    return data
  },

  async update(id: string, request: ReleaseUpdateRequest): Promise<Release> {
    const { data } = await api.patch<Release>(`/releases/${id}`, request)
    return data
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/releases/${id}`)
  },

  async searchCatalog(q: string, limit = 10): Promise<ResolvedMetadata[]> {
    const { data } = await api.get<ResolvedMetadata[]>('/releases/catalog', { params: { q, limit } })
    return data
  }
}
