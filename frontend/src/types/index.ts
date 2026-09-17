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
/** A tracked album with no MusicBrainz id, as a bulk match searches for it. */
export interface UnlinkedRelease {
  id: string
  artist: string
  title: string
  releaseYear: number | null
  streamingLinks: Record<string, string>
}

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
  /** Only albums with no MusicBrainz id. */
  unlinked?: boolean
  /** Only albums with no cover URL. */
  withoutCover?: boolean
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

/** Which year a year-end list groups by: when you listened, or when the album came out. */
export type YearEndBasis = 'listened' | 'released'

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

/** The queue's order, by the date each album was added. */
export type QueueSort = 'newest' | 'oldest'

/** Preferences for how the app behaves on this machine. Not part of the library. */
export interface Settings {
  closeAction: CloseAction
  queueSort: QueueSort
}

export type ArtistStatus = 'TO_CHECK' | 'CHECKED'
export type ArtistVerdict = 'LIKED' | 'NOT_FOR_ME'

export interface ArtistLink {
  /** MusicBrainz's relationship type, such as "official homepage" or "bandcamp". */
  kind: string
  url: string
}

/** An artist on your list. `id` is the catalog artist id. */
export interface Artist {
  id: string
  musicbrainzArtistId: string | null
  name: string
  disambiguation: string | null
  /** Group, Person, Orchestra, Choir, Character or Other. */
  artistType: string | null
  country: string | null
  beginYear: number | null
  endYear: number | null
  /** The cover of one of their albums; MusicBrainz has no artist photos. */
  imageUrl: string | null
  genres: string[]
  links: ArtistLink[]
  status: ArtistStatus
  verdict: ArtistVerdict | null
  note: string | null
  checkedAt: string | null
  createdAt: string
}

/** One match for an artist search. */
export interface ArtistCandidate {
  musicbrainzArtistId: string
  name: string
  disambiguation: string | null
  artistType: string | null
  country: string | null
  beginYear: number | null
  endYear: number | null
  tags: string[]
}

export interface ArtistAddRequest {
  musicbrainzArtistId: string
  note?: string
}

/**
 * An absent field is left alone. Whenever `status` is sent, `verdict` is taken as given, so
 * leaving it out clears it. An empty `note` removes the note.
 */
export interface ArtistUpdateRequest {
  note?: string
  status?: ArtistStatus
  verdict?: ArtistVerdict | null
}

/** Where an album from a discography already is in your library. */
export interface LibraryMatch {
  releaseId: string
  status: ReleaseStatus
  rating: number | null
}

/** One album or EP of an artist's discography. */
export interface DiscographyEntry extends AlbumCandidate {
  library: LibraryMatch | null
}
