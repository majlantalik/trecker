export type ReleaseStatus = 'QUEUED' | 'LISTENED'

export interface Release {
  id: string
  artist: string
  title: string
  releaseYear: number | null
  albumArtUrl: string | null
  status: ReleaseStatus
  discoveryLink: string | null
  streamingLinks: Record<string, string>
  country: string | null
  rating: number | null
  didNotFinish: boolean
  dateListened: string | null
  notes: string | null
  createdAt: string
  genres: string[]
}

export interface ReleaseRequest {
  artist: string
  title: string
  releaseYear?: number
  albumArtUrl?: string
  country?: string
  discoveryLink?: string
  streamingLinks?: Record<string, string>
  genres?: string[]
  musicbrainzReleaseGroupId?: string
}

export interface ReleaseUpdateRequest {
  artist?: string
  title?: string
  releaseYear?: number
  albumArtUrl?: string
  status?: ReleaseStatus
  discoveryLink?: string
  streamingLinks?: Record<string, string>
  country?: string
  rating?: number
  didNotFinish?: boolean
  dateListened?: string
  notes?: string
  genres?: string[]
}

export interface ResolveRequest {
  url?: string
  query?: string
}

export interface ResolvedMetadata {
  artist: string | null
  title: string | null
  releaseYear: number | null
  albumArtUrl: string | null
  country: string | null
  streamingLinks: Record<string, string>
  genres: string[]
  musicbrainzReleaseGroupId?: string
}

/** One possible match from a quick add search. Genres, country and cover come later. */
export interface AlbumCandidate {
  musicbrainzReleaseGroupId: string
  artist: string | null
  title: string | null
  releaseYear: number | null
  primaryType: string | null
  secondaryTypes: string[]
  disambiguation: string | null
  albumArtUrl: string
}

export interface PageResponse<T> {
  content: T[]
  totalElements: number
  totalPages: number
  number: number
  size: number
  first: boolean
  last: boolean
}

export interface ReleaseFilterParams {
  status?: ReleaseStatus
  genre?: string
  country?: string
  ratingMin?: number
  ratingMax?: number
  year?: number
  didNotFinish?: boolean
  search?: string
  page?: number
  size?: number
  sort?: string
  direction?: 'ASC' | 'DESC'
}

export interface ActivityDataPoint {
  year: number
  month: number
  count: number
}

export interface BreakdownItem {
  label: string
  count: number
}

export interface YearEndEntry {
  rank: number
  release: Release
}

export interface DbInfo {
  path: string
  sizeBytes: number
  schemaVersion: number
  fts5: boolean
  journalMode: string
  foreignKeys: boolean
  releaseCount: number
  trackedCount: number
}

export type ExportFormat = 'json' | 'csv'
export type ImportMode = 'skip' | 'overwrite'

export interface ExportSummary {
  path: string
  format: ExportFormat
  releaseCount: number
  bytes: number
}

export interface RejectedRow {
  row: number
  artist: string
  title: string
  reason: string
}

export interface ImportReport {
  added: number
  overwritten: number
  skipped: number
  rejected: RejectedRow[]
}

export interface CoverCacheInfo {
  path: string
  count: number
  sizeBytes: number
}

/** What closing the main window does. */
export type CloseAction = 'quit' | 'tray'

/** Preferences for how the app behaves on this machine. Not part of the library. */
export interface Settings {
  closeAction: CloseAction
}
