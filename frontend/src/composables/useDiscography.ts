import { computed, ref } from 'vue'
import { artistsApi } from '@/api/artists'
import { releasesApi } from '@/api/releases'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import { prefillFromCandidate } from '@/utils/candidates'
import { mainDiscography } from '@/utils/artists'
import type { DiscographyEntry, Release, ReleaseRequest, ResolvedMetadata } from '@/types'

/**
 * An artist's discography, and queueing or logging an album from it.
 *
 * Both reuse quick add's path: look the album up for genres, country and cover, then
 * create the release. Queueing shows the add form first, as quick add does. Logging goes
 * straight to the log dialog, which has its own fields for genres and country.
 */
export function useDiscography(artistId: () => string, artistName: () => string) {
  const releases = useReleasesStore()
  const genres = useGenresStore()

  const entries = ref<DiscographyEntry[]>([])
  const loading = ref(false)
  const loaded = ref(false)
  const error = ref('')
  const showAll = ref(false)
  /** The album being looked up or added, so its row can show it. */
  const busyId = ref<string | null>(null)

  const visible = computed(() => (showAll.value ? entries.value : mainDiscography(entries.value)))
  const hiddenCount = computed(() => entries.value.length - mainDiscography(entries.value).length)
  const listenedCount = computed(() => entries.value.filter((e) => e.library?.status === 'LISTENED').length)

  // Queueing: the add form's state.
  const prefill = ref<ResolvedMetadata | null>(null)
  const formVisible = ref(false)
  let formEntryId: string | null = null

  // Logging: the release in the log dialog, and whether it was created just for the dialog.
  const logRelease = ref<Release | null>(null)
  const logVisible = ref(false)
  let createdForLog: { entryId: string; releaseId: string } | null = null

  async function load() {
    loading.value = true
    error.value = ''
    try {
      entries.value = await artistsApi.discography(artistId())
      loaded.value = true
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    } finally {
      loading.value = false
    }
  }

  function entry(id: string) {
    return entries.value.find((e) => e.musicbrainzReleaseGroupId === id)
  }

  function mark(id: string, library: DiscographyEntry['library']) {
    entries.value = entries.value.map((e) => (e.musicbrainzReleaseGroupId === id ? { ...e, library } : e))
  }

  /** The album's full metadata. Browse results carry no artist, so the artist's name fills in. */
  async function details(e: DiscographyEntry): Promise<ResolvedMetadata> {
    let found: ResolvedMetadata | null = null
    try {
      found = await releasesApi.lookup(e.musicbrainzReleaseGroupId)
    } catch {
      // A thinner release beats none: title and year are already known.
    }
    return prefillFromCandidate({ ...e, artist: e.artist ?? artistName() }, found)
  }

  async function queue(e: DiscographyEntry) {
    if (busyId.value || e.library) return
    busyId.value = e.musicbrainzReleaseGroupId
    try {
      prefill.value = await details(e)
      formEntryId = e.musicbrainzReleaseGroupId
      formVisible.value = true
    } finally {
      busyId.value = null
    }
  }

  /** The add form was submitted. Throws when adding fails, for the page to report. */
  async function submitForm(data: ReleaseRequest): Promise<Release> {
    const release = await releases.addRelease(data)
    data.genres?.forEach((g) => genres.addGenre(g))
    if (formEntryId) mark(formEntryId, { releaseId: release.id, status: release.status, rating: release.rating })
    formEntryId = null
    return release
  }

  /** Opens the log dialog, adding the album first if it is not in the library yet. */
  async function log(e: DiscographyEntry) {
    if (busyId.value || e.library?.status === 'LISTENED') return
    busyId.value = e.musicbrainzReleaseGroupId
    try {
      if (e.library) {
        logRelease.value = await releasesApi.getById(e.library.releaseId)
        createdForLog = null
      } else {
        const d = await details(e)
        const release = await releases.addRelease({
          artist: d.artist ?? artistName(),
          title: d.title ?? e.title ?? '',
          releaseYear: d.releaseYear ?? undefined,
          albumArtUrl: d.albumArtUrl ?? undefined,
          country: d.country ?? undefined,
          streamingLinks: d.streamingLinks,
          genres: d.genres,
          musicbrainzReleaseGroupId: d.musicbrainzReleaseGroupId
        })
        mark(e.musicbrainzReleaseGroupId, { releaseId: release.id, status: release.status, rating: release.rating })
        createdForLog = { entryId: e.musicbrainzReleaseGroupId, releaseId: release.id }
        logRelease.value = release
      }
      logVisible.value = true
    } finally {
      busyId.value = null
    }
  }

  function logged(release: Release) {
    const id = entries.value.find((e) => e.library?.releaseId === release.id)?.musicbrainzReleaseGroupId
    if (id) mark(id, { releaseId: release.id, status: release.status, rating: release.rating })
    createdForLog = null
  }

  /**
   * The log dialog closed. If it closed without logging an album that was added only so it
   * could be logged, take it back out: asking to log something is not asking to queue it.
   */
  async function logClosed() {
    const pending = createdForLog
    createdForLog = null
    logRelease.value = null
    if (!pending) return
    try {
      await releases.deleteRelease(pending.releaseId)
      if (entry(pending.entryId)?.library?.releaseId === pending.releaseId) mark(pending.entryId, null)
    } catch {
      // It stays in the queue, which is where the discography now shows it.
    }
  }

  return {
    entries,
    visible,
    hiddenCount,
    listenedCount,
    loading,
    loaded,
    error,
    showAll,
    busyId,
    prefill,
    formVisible,
    logRelease,
    logVisible,
    load,
    queue,
    submitForm,
    log,
    logged,
    logClosed
  }
}
