import { defineStore } from 'pinia'
import { ref } from 'vue'
import { genresApi } from '@/api/genres'

export const useGenresStore = defineStore('genres', () => {
  const genres = ref<string[]>([])
  const loading = ref(false)

  // `force` is for the paths that write genres behind this store's back: an import can
  // add a dozen at once, and the cached list would keep its pre-import contents.
  async function fetchGenres(force = false) {
    if (genres.value.length > 0 && !force) return
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
