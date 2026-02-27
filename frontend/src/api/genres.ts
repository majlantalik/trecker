import api from './axios'

export const genresApi = {
  async getAll(): Promise<string[]> {
    const { data } = await api.get<string[]>('/genres')
    return data
  }
}
