import { open, save } from '@tauri-apps/plugin-dialog'
import type { ExportFormat } from '@/types'

/**
 * The native file pickers.
 *
 * Here rather than in a view for the same reason `invoke` is: everything that knows it is
 * running inside Tauri lives under `src/api/`, so the views and stores stay ordinary Vue
 * and the seam stays one directory wide.
 *
 * Both return null when the dialog is dismissed, which is the common case and not an error.
 */

const NAMES: Record<ExportFormat, string> = {
  json: 'Trecker library (JSON)',
  csv: 'Spreadsheet (CSV)'
}

export const dialogApi = {
  /**
   * The filter names only the format being written. The choice was already made by the
   * button that opened this dialog, so offering the other one here would be a second,
   * contradictory control.
   */
  async saveAs(defaultPath: string, format: ExportFormat): Promise<string | null> {
    return save({
      title: 'Export library',
      defaultPath,
      filters: [{ name: NAMES[format], extensions: [format] }]
    })
  },

  /** One filter, both extensions: on the way in, either kind of file is a library. */
  async openFile(): Promise<string | null> {
    const picked = await open({
      title: 'Import library',
      multiple: false,
      directory: false,
      filters: [{ name: 'Trecker library', extensions: ['json', 'csv'] }]
    })
    return typeof picked === 'string' ? picked : null
  }
}
