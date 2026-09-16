import { invoke } from '@tauri-apps/api/core'
import type { ActivityDataPoint, BreakdownItem, YearEndBasis, YearEndEntry } from '@/types'

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

  async getYearEnd(year?: number, by: YearEndBasis = 'listened'): Promise<YearEndEntry[]> {
    return invoke<YearEndEntry[]>('stats_year_end', { year, by })
  }
}
