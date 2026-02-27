import { defineStore } from 'pinia'
import { ref } from 'vue'
import { genresApi } from '@/api/genres'

export const useGenresStore = defineStore('genres', () => {
  const genres = ref<string[]>([])
  const loading = ref(false)

  async function fetchGenres() {
    if (genres.value.length > 0) return
    loading.value = true
    try {
      genres.value = await genresApi.getAll()
    } catch (e) {
      console.error('Failed to fetch genres', e)
    } finally {
      loading.value = false
    }
  }

  function addGenre(name: string) {
    if (!genres.value.includes(name)) {
      genres.value = [...genres.value, name].sort()
    }
  }

  return { genres, loading, fetchGenres, addGenre }
})
