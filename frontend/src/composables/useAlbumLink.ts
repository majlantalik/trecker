import { ref } from 'vue'
import { releasesApi } from '@/api/releases'
import type { AlbumCandidate, Release } from '@/types'

/**
 * Linking an album that has no MusicBrainz id: search its artist and title, let the person
 * choose, then store the choice and its metadata with `releases_link`.
 *
 * The list opens even for a single match. The album being linked was typed in or imported,
 * and for those the one confident hit is exactly the case that is most often a different
 * record, so nothing is chosen on the person's behalf.
 */
export type LinkSearchOutcome = 'found' | 'none' | 'failed'

export function useAlbumLink() {
  const query = ref('')
  const searching = ref(false)
  const candidates = ref<AlbumCandidate[]>([])
  const pickerVisible = ref(false)
  /** The candidate being linked, so its row can show it and a second pick is ignored. */
  const choosingId = ref<string | null>(null)
  const error = ref('')

  async function search(release: Pick<Release, 'artist' | 'title'>): Promise<LinkSearchOutcome> {
    // A spaced dash makes the resolver search artist and title separately.
    query.value = `${release.artist} - ${release.title}`
    error.value = ''
    candidates.value = []
    searching.value = true
    try {
      candidates.value = await releasesApi.search(query.value)
      if (!candidates.value.length) return 'none'
      pickerVisible.value = true
      return 'found'
    } catch (e: any) {
      error.value = e?.message ?? String(e)
      return 'failed'
    } finally {
      searching.value = false
    }
  }

  /**
   * Links the release to the chosen album and returns it updated, or null when a pick is
   * already in flight. Errors are rethrown for the caller, with the list closed either way,
   * so a follow-up dialog is not stacked on top of it.
   */
  async function choose(releaseId: string, candidate: AlbumCandidate): Promise<Release | null> {
    if (choosingId.value) return null
    choosingId.value = candidate.musicbrainzReleaseGroupId
    try {
      return await releasesApi.link(releaseId, candidate.musicbrainzReleaseGroupId)
    } finally {
      choosingId.value = null
      pickerVisible.value = false
    }
  }

  return { query, searching, candidates, pickerVisible, choosingId, error, search, choose }
}
