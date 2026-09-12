# 0005. Library export format

- **Status:** Accepted
- **Date:** 2026-09-12

## Context

A local-first app has no server holding a second copy of anyone's data. Moving to another
machine, restoring after a disk failure, or reading the library in a spreadsheet all need
a file format.

The format has to survive four situations that pull in different directions: backup and
restore, moving to another machine where local row ids mean nothing, reading in a
spreadsheet, and being read years later by something other than Trecker.

The field-by-field specification lives in [docs/export-format.md](../export-format.md). It
was written before either side was built, and it is the authority on detail. This record
covers the decisions behind it.

## Decision

- **JSON is canonical and CSV round-trips.** JSON carries an envelope with a `format`
  discriminator, an integer `formatVersion`, and a `releaseCount` that must match the array
  so truncation is caught before anything is written. CSV uses the same field names as
  headers, so the two need no mapping table.
- **No local row ids and no user id are exported.** They mean nothing on another machine,
  and including them would invite an importer to trust them.
- **Identity on import** is the MusicBrainz id first, then artist and title compared
  case-insensitively. The second rule is a heuristic and can be wrong. It lives in import,
  where the report shows what matched, and never in ordinary album creation.
- **Import is additive and idempotent.** It never deletes. Importing the same file twice
  changes nothing the second time.
- **On a match the user chooses skip, the default, or overwrite. There is no merge.**
- **A bad row is rejected and reported, never quietly corrected,** and the remaining rows
  still import.
- **Artwork is exported as a URL, not as image data.**
- **`musicbrainzId` holds a release group id** without saying so in its name. See
  [ADR 0004](0004-release-group-is-album-identity.md).

## Alternatives considered

- **Copy the SQLite file as the backup.** Lossless, but only readable by the same schema
  version of Trecker, not by a spreadsheet, and not mergeable into an existing library.
- **CSV only.** Needs no envelope, which also means no version and no way to detect
  truncation. It would still need two nested fields encoded inside cells.
- **A merge mode.** It needs a rule for every field, and for a rating or a note the only
  honest rule is that one of the two is wrong and only the person can say which.
- **Correct bad values on import,** such as rounding a rating to a half step. It makes an
  invisible edit to someone's data, which is the same objection as merge.
- **Embed artwork.** Tens of megabytes for a modest library, and the Cover Art Archive is a
  better custodian of those images than a backup file.
- **All-or-nothing import.** A single bad row would block the whole library, and the
  report already makes a partial import visible.

## Consequences

- **The format and the database are decoupled.** `src-tauri/src/library/` decides what the
  bytes mean with no SQL, and the repository layer does the writing.
- **Validation runs before any SQL,** which makes the database's constraints unreachable
  through import. A bad row becomes a reported rejection rather than a database error.
- **Two different albums with identical artist and title and no MusicBrainz id import as
  one.** Accepted, because refusing to match without an id would duplicate most of a
  library on re-import.
- **A restored library shows no art for any URL that has since rotted.** Refreshing
  metadata repairs those.
- **Changing the meaning of a field requires a `formatVersion` bump.** Adding an optional
  field does not.
- **Native file dialogs required `tauri-plugin-dialog`,** which added 3.5 MB to the binary.
