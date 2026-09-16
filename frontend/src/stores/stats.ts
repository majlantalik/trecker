import { defineStore } from 'pinia'
import { ref } from 'vue'
import { statsApi } from '@/api/stats'
import type { ActivityDataPoint, BreakdownItem, YearEndBasis, YearEndEntry } from '@/types'

export const useStatsStore = defineStore('stats', () => {
  const activity = ref<ActivityDataPoint[]>([])
  const byGenre = ref<BreakdownItem[]>([])
  const byCountry = ref<BreakdownItem[]>([])
  const yearEnd = ref<YearEndEntry[]>([])
  const selectedYear = ref(new Date().getFullYear())
  const yearEndBy = ref<YearEndBasis>('listened')
  const loading = ref(false)

  async function fetchAll() {
    loading.value = true
    try {
      const [act, genre, country] = await Promise.all([
        statsApi.getActivity(),
        statsApi.getByGenre(),
        statsApi.getByCountry()
      ])
      activity.value = act
      byGenre.value = genre
      byCountry.value = country
    } catch (e) {
      console.error('Failed to fetch stats', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchYearEnd(year?: number, by?: YearEndBasis) {
    if (year) selectedYear.value = year
    if (by) yearEndBy.value = by
    yearEnd.value = await statsApi.getYearEnd(selectedYear.value, yearEndBy.value)
  }

  return {
    activity, byGenre, byCountry, yearEnd, selectedYear, yearEndBy, loading, fetchAll, fetchYearEnd
  }
})
