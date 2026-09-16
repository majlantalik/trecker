import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { releasesApi } from '@/api/releases'
import { useGenresStore } from '@/stores/genres'
import { exactMatch } from '@/utils/albumMatch'
import type { AlbumCandidate, UnlinkedRelease } from '@/types'

/** An album the run could not link on its own, with the search results to choose from. */
export interface MatchReviewItem {
  release: UnlinkedRelease
  candidates: AlbumCandidate[]
}

/** An album that could not be linked, and why. `existingId` is set for a duplicate. */
export interface MatchProblem {
  release: UnlinkedRelease
  reason: string
  existingId?: string
}

/** A run stops rather than failing every remaining album when MusicBrainz is unreachable. */
export const CONSECUTIVE_FAILURES_TO_STOP = 3

function reasonOf(e: any): string {
  return e?.message ?? String(e)
}

/**
 * Linking every album without a MusicBrainz id, in two passes.
 *
 * The run searches each album and links it only on an exact match (see `exactMatch`).
 * Everything else waits in `toReview` for the person to choose from its search results,
 * which are already fetched, so reviewing costs one lookup per pick. Reviewing can start
 * while the run is still going.
 *
 * A store rather than a composable so the run keeps going when the Info view is left.
 * Nothing about a "none of these" is remembered: the next run asks again.
 */
export const useAlbumMatchStore = defineStore('albumMatch', () => {
  const unlinkedCount = ref<number | null>(null)
  const running = ref(false)
  const stopping = ref(false)
  const finished = ref(false)
  const error = ref('')

  const total = ref(0)
  const processed = ref(0)
  const linked = ref(0)
  const declined = ref(0)
  const toReview = ref<MatchReviewItem[]>([])
  const noMatches = ref<UnlinkedRelease[]>([])
  const duplicates = ref<MatchProblem[]>([])
  const failed = ref<MatchProblem[]>([])
  /** The candidate being linked from the review, so its row can show it. */
  const choosingId = ref<string | null>(null)

  const current = computed(() => toReview.value[0] ?? null)

  async function countUnlinked() {
    try {
      unlinkedCount.value = (await releasesApi.unlinked()).length
    } catch {
      unlinkedCount.value = null
    }
  }

  function reset() {
    error.value = ''
    finished.value = false
    stopping.value = false
    total.value = 0
    processed.value = 0
    linked.value = 0
    declined.value = 0
    toReview.value = []
    noMatches.value = []
    duplicates.value = []
    failed.value = []
  }

  async function link(release: UnlinkedRelease, candidate: AlbumCandidate) {
    try {
      await releasesApi.link(release.id, candidate.musicbrainzReleaseGroupId)
      linked.value++
      if (unlinkedCount.value) unlinkedCount.value--
    } catch (e: any) {
      if (e?.code === 'ALREADY_IN_LIBRARY') {
        duplicates.value.push({ release, reason: reasonOf(e), existingId: e.releaseId })
      } else {
        failed.value.push({ release, reason: reasonOf(e) })
      }
    }
  }

  /** Searches one album and links it on an exact match. False when the search failed. */
  async function matchOne(release: UnlinkedRelease): Promise<boolean> {
    let candidates: AlbumCandidate[]
    try {
      candidates = await releasesApi.search(`${release.artist} - ${release.title}`)
    } catch (e) {
      failed.value.push({ release, reason: reasonOf(e) })
      return false
    }

    if (!candidates.length) {
      noMatches.value.push(release)
      return true
    }
    const exact = exactMatch(release, candidates)
    if (exact) await link(release, exact)
    else toReview.value.push({ release, candidates })
    return true
  }

  async function run() {
    if (running.value) return
    reset()
    running.value = true
    try {
      const albums = await releasesApi.unlinked()
      total.value = albums.length
      let failuresInARow = 0
      for (const release of albums) {
        if (stopping.value) break
        failuresInARow = (await matchOne(release)) ? 0 : failuresInARow + 1
        processed.value++
        if (failuresInARow >= CONSECUTIVE_FAILURES_TO_STOP) {
          error.value = `Stopped after ${failuresInARow} searches in a row failed. Is MusicBrainz reachable?`
          break
        }
      }
    } catch (e) {
      error.value = reasonOf(e)
    } finally {
      running.value = false
      finished.value = true
      // Linking adds MusicBrainz's genres, behind the genres store's back.
      await useGenresStore().fetchGenres(true)
      await countUnlinked()
    }
  }

  function stop() {
    if (running.value) stopping.value = true
  }

  function removeCurrent(item: MatchReviewItem) {
    toReview.value = toReview.value.filter((i) => i !== item)
  }

  async function choose(candidate: AlbumCandidate) {
    const item = current.value
    if (!item || choosingId.value) return
    choosingId.value = candidate.musicbrainzReleaseGroupId
    try {
      await link(item.release, candidate)
    } finally {
      choosingId.value = null
      removeCurrent(item)
    }
    // A run refreshes genres when it ends; a pick made after that has to do its own.
    if (!running.value) await useGenresStore().fetchGenres(true)
  }

  /** "None of these": the album stays as it is, and the next run asks about it again. */
  function decline() {
    const item = current.value
    if (!item || choosingId.value) return
    declined.value++
    removeCurrent(item)
  }

  return {
    unlinkedCount, running, stopping, finished, error,
    total, processed, linked, declined, toReview, noMatches, duplicates, failed, choosingId,
    current, countUnlinked, run, stop, choose, decline
  }
})
