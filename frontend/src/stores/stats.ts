import { defineStore } from 'pinia'
import { ref } from 'vue'
import { statsApi } from '@/api/stats'
import type { ActivityDataPoint, BreakdownItem, Release, YearEndEntry } from '@/types'

export const useStatsStore = defineStore('stats', () => {
  const activity = ref<ActivityDataPoint[]>([])
  const byGenre = ref<BreakdownItem[]>([])
  const byCountry = ref<BreakdownItem[]>([])
  const topRated = ref<Release[]>([])
  const yearEnd = ref<YearEndEntry[]>([])
  const selectedYear = ref(new Date().getFullYear())
  const loading = ref(false)

  async function fetchAll() {
    loading.value = true
    try {
      const [act, genre, country, top] = await Promise.all([
        statsApi.getActivity(),
        statsApi.getByGenre(),
        statsApi.getByCountry(),
        statsApi.getTopRated()
      ])
      activity.value = act
      byGenre.value = genre
      byCountry.value = country
      topRated.value = top
    } catch (e) {
      console.error('Failed to fetch stats', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchYearEnd(year?: number) {
    if (year) selectedYear.value = year
    yearEnd.value = await statsApi.getYearEnd(selectedYear.value)
  }

  return { activity, byGenre, byCountry, topRated, yearEnd, selectedYear, loading, fetchAll, fetchYearEnd }
})
