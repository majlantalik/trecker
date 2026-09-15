import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { artistsApi } from '@/api/artists'
import type { Artist, ArtistAddRequest, ArtistUpdateRequest } from '@/types'

/**
 * Your list of artists to check. Small enough to hold whole: a backlog of artists runs to
 * dozens, not the thousands a library of albums can.
 */
export const useArtistsStore = defineStore('artists', () => {
  const artists = ref<Artist[]>([])
  const loaded = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const toCheck = computed(() => artists.value.filter((a) => a.status === 'TO_CHECK'))
  const checked = computed(() => artists.value.filter((a) => a.status === 'CHECKED'))

  async function fetchArtists() {
    loading.value = true
    error.value = null
    try {
      artists.value = await artistsApi.list()
      loaded.value = true
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    } finally {
      loading.value = false
    }
  }

  /** Replaces an artist in the list, or puts a new one at the top of those to check. */
  function upsert(artist: Artist) {
    const at = artists.value.findIndex((a) => a.id === artist.id)
    if (at === -1) {
      artists.value = [artist, ...artists.value]
    } else {
      artists.value = artists.value.map((a) => (a.id === artist.id ? artist : a))
    }
  }

  async function addArtist(request: ArtistAddRequest) {
    const artist = await artistsApi.add(request)
    upsert(artist)
    return artist
  }

  async function updateArtist(id: string, request: ArtistUpdateRequest) {
    const before = artists.value.find((a) => a.id === id)
    const artist = await artistsApi.update(id, request)
    const moved = before && before.status !== artist.status
    artists.value = artists.value.map((a) => (a.id === id ? artist : a))
    if (moved) await fetchArtists()
    return artist
  }

  async function deleteArtist(id: string) {
    await artistsApi.delete(id)
    artists.value = artists.value.filter((a) => a.id !== id)
  }

  return {
    artists,
    loaded,
    loading,
    error,
    toCheck,
    checked,
    fetchArtists,
    addArtist,
    updateArtist,
    deleteArtist
  }
})
