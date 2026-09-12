import { ref } from 'vue'
import { releasesApi } from '@/api/releases'
import { prefillFromCandidate, prefillFromText } from '@/utils/candidates'
import type { AlbumCandidate, ResolvedMetadata } from '@/types'

/**
 * Quick add's search: look up what was typed, let the person choose, prefill the form.
 *
 * One match opens the form straight away. Several open a list first. None, or MusicBrainz
 * being unreachable, opens the form with whatever the typed text gives.
 */
export type SearchOutcome = 'one' | 'many' | 'none' | 'failed'

export function useAlbumSearch() {
  const query = ref('')
  const searching = ref(false)
  const candidates = ref<AlbumCandidate[]>([])
  const pickerVisible = ref(false)
  /** The candidate whose details are being fetched, so its row can show it. */
  const choosingId = ref<string | null>(null)
  const prefill = ref<ResolvedMetadata | null>(null)
  const formVisible = ref(false)
  const error = ref('')

  function openForm(metadata: ResolvedMetadata | null) {
    pickerVisible.value = false
    prefill.value = metadata
    formVisible.value = true
  }

  async function search(text: string): Promise<SearchOutcome> {
    const trimmed = text.trim()
    query.value = trimmed
    error.value = ''
    candidates.value = []
    searching.value = true
    try {
      const found = await releasesApi.search(trimmed)
      candidates.value = found
      if (found.length === 1) {
        await choose(found[0])
        return 'one'
      }
      if (found.length > 1) {
        pickerVisible.value = true
        return 'many'
      }
      openForm(prefillFromText(trimmed))
      return 'none'
    } catch (e: any) {
      error.value = e?.message ?? String(e)
      openForm(prefillFromText(trimmed))
      return 'failed'
    } finally {
      searching.value = false
    }
  }

  async function choose(candidate: AlbumCandidate) {
    // A second pick while the first is still loading would open two forms' worth of work.
    if (choosingId.value) return
    choosingId.value = candidate.musicbrainzReleaseGroupId
    let details: ResolvedMetadata | null = null
    try {
      details = await releasesApi.lookup(candidate.musicbrainzReleaseGroupId)
    } catch {
      // The search result still has the artist, title and year. Better a thinner form than
      // no form.
    }
    choosingId.value = null
    openForm(prefillFromCandidate(candidate, details))
  }

  /** "None of these": the list had nothing right, so fill the form in by hand. */
  function chooseNone() {
    openForm(prefillFromText(query.value))
  }

  return {
    query,
    searching,
    candidates,
    pickerVisible,
    choosingId,
    prefill,
    formVisible,
    error,
    search,
    choose,
    chooseNone,
    openForm
  }
}
