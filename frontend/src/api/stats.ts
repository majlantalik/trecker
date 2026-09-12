import { invoke } from '@tauri-apps/api/core'
import type { ActivityDataPoint, BreakdownItem, Release, YearEndEntry } from '@/types'

export const statsApi = {
  async getActivity(): Promise<ActivityDataPoint[]> {
    return invoke<ActivityDataPoint[]>('stats_activity')
  },

  async getByGenre(): Promise<BreakdownItem[]> {
    return invoke<BreakdownItem[]>('stats_by_genre')
  },

  async getByCountry(): Promise<BreakdownItem[]> {
    return invoke<BreakdownItem[]>('stats_by_country')
  },

  async getTopRated(limit = 25): Promise<Release[]> {
    return invoke<Release[]>('stats_top_rated', { limit })
  },

  async getYearEnd(year?: number): Promise<YearEndEntry[]> {
    return invoke<YearEndEntry[]>('stats_year_end', { year })
  }
}
