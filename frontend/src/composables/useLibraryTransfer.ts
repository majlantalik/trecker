import { ref } from 'vue'
import { libraryApi } from '@/api/library'
import { dialogApi } from '@/api/dialog'
import { useReleasesStore } from '@/stores/releases'
import { useGenresStore } from '@/stores/genres'
import type { ExportFormat, ExportSummary, ImportMode, ImportReport } from '@/types'

/**
 * Writing the library to a file and reading one back.
 *
 * The format is specified in `docs/export-format.md` and implemented in Rust. This is only
 * the sequence: pick a path, call the command, say what happened.
 */

/** `trecker-library-2026-09-12.json`, so a folder of exports sorts chronologically. */
export function defaultExportName(format: ExportFormat, now: Date = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  const day = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`
  return `trecker-library-${day}.${format}`
}

export function describeExport(s: ExportSummary): string {
  const releases = s.releaseCount === 1 ? '1 release' : `${s.releaseCount} releases`
  return `Exported ${releases} to ${s.path}`
}

export function describeImport(r: ImportReport): string {
  const parts: string[] = []
  if (r.added) parts.push(`${r.added} added`)
  if (r.overwritten) parts.push(`${r.overwritten} overwritten`)
  if (r.skipped) parts.push(`${r.skipped} already in your library`)
  if (r.rejected.length) parts.push(`${r.rejected.length} rejected`)
  // Every count can be zero at once: importing a file you have already imported, in skip
  // mode, correctly changes nothing, and saying so beats an empty sentence.
  return parts.length ? parts.join(', ') : 'Nothing to import'
}

export function useLibraryTransfer() {
  const releasesStore = useReleasesStore()
  const genresStore = useGenresStore()

  const busy = ref(false)
  const message = ref('')
  const error = ref('')
  const report = ref<ImportReport | null>(null)

  function reset() {
    message.value = ''
    error.value = ''
    report.value = null
  }

  async function run(work: () => Promise<void>) {
    reset()
    busy.value = true
    try {
      await work()
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    } finally {
      busy.value = false
    }
  }

  async function exportLibrary(format: ExportFormat) {
    await run(async () => {
      const path = await dialogApi.saveAs(defaultExportName(format), format)
      // Dismissing the dialog is the ordinary way to change your mind, not a failure.
      if (!path) return
      message.value = describeExport(await libraryApi.export(path, format))
    })
  }

  async function importLibrary(mode: ImportMode) {
    await run(async () => {
      const path = await dialogApi.openFile()
      if (!path) return
      const result = await libraryApi.import(path, mode)
      report.value = result
      message.value = describeImport(result)

      // An import writes releases and genres behind the views' backs. The badge and the
      // genre filter would otherwise keep their pre-import values until a reload.
      await Promise.all([releasesStore.refreshQueuedCount(), genresStore.fetchGenres(true)])
    })
  }

  return { busy, message, error, report, exportLibrary, importLibrary }
}
