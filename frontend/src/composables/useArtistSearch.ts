import { ref } from 'vue'
import { artistsApi } from '@/api/artists'
import { useArtistsStore } from '@/stores/artists'
import type { Artist, ArtistCandidate } from '@/types'

export type ArtistSearchOutcome = 'found' | 'none' | 'failed'

/**
 * Adding an artist: search, choose from the list, add.
 *
 * Unlike quick add, a single match is not added straight away. Adding an album opens a form
 * to check first; adding an artist has no form, so the list is the one chance to see that
 * MusicBrainz found the right person.
 */
export function useArtistSearch() {
  const store = useArtistsStore()
  const query = ref('')
  const searching = ref(false)
  const candidates = ref<ArtistCandidate[]>([])
  const pickerVisible = ref(false)
  /** The candidate being looked up and added, so its row can show it. */
  const addingId = ref<string | null>(null)
  const error = ref('')

  async function search(text: string): Promise<ArtistSearchOutcome> {
    const trimmed = text.trim()
    if (!trimmed) return 'none'
    query.value = trimmed
    error.value = ''
    candidates.value = []
    searching.value = true
    try {
      candidates.value = await artistsApi.search(trimmed)
      if (candidates.value.length === 0) return 'none'
      pickerVisible.value = true
      return 'found'
    } catch (e: any) {
      error.value = e?.message ?? String(e)
      return 'failed'
    } finally {
      searching.value = false
    }
  }

  /** Adds the chosen artist. Null when adding failed; `error` says why. */
  async function choose(candidate: ArtistCandidate): Promise<Artist | null> {
    if (addingId.value) return null
    addingId.value = candidate.musicbrainzArtistId
    error.value = ''
    try {
      const artist = await store.addArtist({ musicbrainzArtistId: candidate.musicbrainzArtistId })
      pickerVisible.value = false
      return artist
    } catch (e: any) {
      error.value = e?.message ?? String(e)
      return null
    } finally {
      addingId.value = null
    }
  }

  return { query, searching, candidates, pickerVisible, addingId, error, search, choose }
}
