import { invoke } from '@tauri-apps/api/core'
import type { ExportFormat, ExportSummary, ImportMode, ImportReport } from '@/types'

export const libraryApi = {
  async export(path: string, format: ExportFormat): Promise<ExportSummary> {
    return invoke<ExportSummary>('library_export', { path, format })
  },

  async import(path: string, mode: ImportMode): Promise<ImportReport> {
    return invoke<ImportReport>('library_import', { path, mode })
  }
}
