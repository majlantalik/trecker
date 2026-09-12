import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { ImportReport } from '@/types'

const saveAs = vi.fn()
const openFile = vi.fn()
const exportLibrary = vi.fn()
const importLibrary = vi.fn()

vi.mock('@/api/dialog', () => ({ dialogApi: { saveAs, openFile } }))
vi.mock('@/api/library', () => ({
  libraryApi: { export: exportLibrary, import: importLibrary }
}))
vi.mock('@/api/releases', () => ({
  releasesApi: { getAll: vi.fn().mockResolvedValue({ totalElements: 0, content: [] }) }
}))
vi.mock('@/api/genres', () => ({ genresApi: { getAll: vi.fn().mockResolvedValue([]) } }))

const { useLibraryTransfer, defaultExportName, describeImport } = await import(
  './useLibraryTransfer'
)

function report(over: Partial<ImportReport> = {}): ImportReport {
  return { added: 0, overwritten: 0, skipped: 0, rejected: [], ...over }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  saveAs.mockResolvedValue('/home/me/library.json')
  openFile.mockResolvedValue('/home/me/library.json')
  exportLibrary.mockResolvedValue({
    path: '/home/me/library.json',
    format: 'json',
    releaseCount: 2,
    bytes: 900
  })
  importLibrary.mockResolvedValue(report({ added: 2 }))
})

describe('defaultExportName', () => {
  it('dates the file so a folder of them sorts chronologically', () => {
    expect(defaultExportName('json', new Date(2026, 8, 12))).toBe('trecker-library-2026-09-12.json')
    expect(defaultExportName('csv', new Date(2026, 0, 5))).toBe('trecker-library-2026-01-05.csv')
  })
})

describe('export', () => {
  it('writes to the path the dialog returned', async () => {
    const t = useLibraryTransfer()
    await t.exportLibrary('csv')
    expect(saveAs).toHaveBeenCalledWith(
      expect.stringMatching(/^trecker-library-\d{4}-\d{2}-\d{2}\.csv$/),
      'csv'
    )
    expect(exportLibrary).toHaveBeenCalledWith('/home/me/library.json', 'csv')
    expect(t.message.value).toContain('2 releases')
    expect(t.error.value).toBe('')
  })

  it('does nothing at all when the dialog is dismissed', async () => {
    // Changing your mind is the ordinary way out of a file picker, not a failure.
    saveAs.mockResolvedValue(null)
    const t = useLibraryTransfer()
    await t.exportLibrary('json')
    expect(exportLibrary).not.toHaveBeenCalled()
    expect(t.message.value).toBe('')
    expect(t.error.value).toBe('')
  })

  it('surfaces a failure instead of claiming success', async () => {
    exportLibrary.mockRejectedValue({ code: 'INTERNAL', message: 'disk is full' })
    const t = useLibraryTransfer()
    await t.exportLibrary('json')
    expect(t.error.value).toBe('disk is full')
    expect(t.message.value).toBe('')
    expect(t.busy.value).toBe(false)
  })
})

describe('import', () => {
  it('passes the chosen mode through', async () => {
    const t = useLibraryTransfer()
    await t.importLibrary('overwrite')
    expect(importLibrary).toHaveBeenCalledWith('/home/me/library.json', 'overwrite')
  })

  it('keeps the rejected rows so they can be shown', async () => {
    const rejected = [{ row: 3, artist: 'Duster', title: 'Stratosphere', reason: 'unknown status' }]
    importLibrary.mockResolvedValue(report({ added: 1, rejected }))
    const t = useLibraryTransfer()
    await t.importLibrary('skip')
    expect(t.report.value?.rejected).toEqual(rejected)
  })

  it('clears the previous result before running again', async () => {
    const t = useLibraryTransfer()
    await t.importLibrary('skip')
    expect(t.message.value).not.toBe('')
    openFile.mockResolvedValue(null)
    await t.importLibrary('skip')
    expect(t.message.value).toBe('')
    expect(t.report.value).toBeNull()
  })
})

describe('describeImport', () => {
  it('names only the counts that happened', () => {
    expect(describeImport(report({ added: 3 }))).toBe('3 added')
    expect(describeImport(report({ added: 1, skipped: 2 }))).toBe(
      '1 added, 2 already in your library'
    )
    expect(describeImport(report({ overwritten: 4, rejected: [
      { row: 1, artist: 'a', title: 'b', reason: 'c' }
    ] }))).toBe('4 overwritten, 1 rejected')
  })

  it('says something when every count is zero', () => {
    // Re-importing a file you have already imported is the common way to land here, and
    // an empty sentence reads as a bug.
    expect(describeImport(report())).toBe('Nothing to import')
  })
})
