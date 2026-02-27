import api from './axios'
import type { ActivityDataPoint, BreakdownItem, Release, YearEndEntry } from '@/types'

export const statsApi = {
  async getActivity(): Promise<ActivityDataPoint[]> {
    const { data } = await api.get<ActivityDataPoint[]>('/stats/activity')
    return data
  },

  async getByGenre(): Promise<BreakdownItem[]> {
    const { data } = await api.get<BreakdownItem[]>('/stats/by-genre')
    return data
  },

  async getByCountry(): Promise<BreakdownItem[]> {
    const { data } = await api.get<BreakdownItem[]>('/stats/by-country')
    return data
  },

  async getTopRated(limit = 25): Promise<Release[]> {
    const { data } = await api.get<Release[]>('/stats/top-rated', { params: { limit } })
    return data
  },

  async getYearEnd(year?: number): Promise<YearEndEntry[]> {
    const { data } = await api.get<YearEndEntry[]>('/stats/year-end', { params: { year } })
    return data
  }
}
