# Library export format

A specification for exporting a Trecker library and importing it back, on the same
machine or another one. Written before either side was built, so the two were implemented
against one document rather than against each other.

Both are now built, under **Info → Backup**. The format lives in `src-tauri/src/library/`
and the writing in `repo::releases::import_one`; where this document and the code
disagree, this document is right and the code is a bug.

## What the format has to survive

An export is useful in four situations, and they pull in slightly different directions:

1. **Backup and restore.** Must be lossless and must not duplicate anything on the way
   back in.
2. **Moving to another machine.** Local row ids mean nothing there, so identity has to
   come from the data itself.
3. **Reading it in a spreadsheet.** Wants flat columns and no nesting.
4. **Being read by something else entirely**, years later, possibly without Trecker.

JSON serves 1, 2 and 4. CSV serves 3, and round-trips too, at the cost of some
readability in two columns.

## Identity: how a row is matched on import

Local UUIDs are **not exported**. They are meaningless on another machine, and including
them would invite an importer to trust them.

A release is matched against the existing library in this order, stopping at the first hit:

1. `musicbrainzId`, when both sides have one
2. `artist` and `title`, compared case-insensitively after trimming

`musicbrainzId` is the id of a MusicBrainz **release group**, which is MusicBrainz's word for
an album across all its pressings, reissues and editions. It is not the id of a
*release*, which in their model is one specific pressing. Look it up at
`/ws/2/release-group/<id>`; asking `/ws/2/release/<id>` returns not found.

The field name does not say "release group" on purpose. A CSV header is read by people in a
spreadsheet, and to anyone who has not studied MusicBrainz's data model the term suggests a
group of albums. This document is where the precision lives.

Rule 1 mirrors the precedence `repo::releases::create` already uses to dedupe the catalog,
so import reuses logic that is already tested rather than inventing a second notion of
sameness. Rule 2 exists only here, because guessing that identical text means an identical
album is a judgement the import report can show you and a silent `create()` cannot.

Both rules are matched against the **catalog**, not only against what you currently track.
A release you deleted keeps its catalog row, so re-importing it lands back on that row
rather than creating a second copy of the same album under a new id.

Case-insensitivity is SQLite's `COLLATE NOCASE`, which folds ASCII and nothing else, so
"BJORK" matches "björk" only in its first four letters' worth of luck. That is the same
limit every text sort and the genre lookup already carry, and matching more loosely here
would merge records rather than order them.

There is no `spotifyId` anywhere any more. Nothing had written one since Spotify became
link-only, so the column was removed from the schema along with the dedup step that
checked it.

The second rule is a heuristic and can be wrong: two genuinely different releases with the
same artist and title and no external ids will be treated as one. That is rare, it only
applies to hand-entered rows, and the alternative, refusing to match without an external
id, would duplicate most of a library on re-import. Import reports what it matched so the
mistake is visible rather than silent.

## JSON

The canonical format. One file, one object.

```json
{
  "format": "trecker-library",
  "formatVersion": 1,
  "exportedAt": "2026-09-12T09:20:00Z",
  "appVersion": "0.1.0",
  "releaseCount": 2,
  "releases": [
    {
      "artist": "Slint",
      "title": "Spiderland",
      "releaseYear": 1991,
      "country": "US",
      "albumArtUrl": "https://coverartarchive.org/release/266e8eb6-.../19590732868-500.jpg",
      "musicbrainzId": "…a release group id…",
      "genres": ["alternative rock", "rock"],
      "streamingLinks": { "spotify": "https://open.spotify.com/album/..." },
      "status": "LISTENED",
      "rating": 4.5,
      "didNotFinish": false,
      "dateListened": "2026-09-01T19:30:00Z",
      "notes": "Long-form crescendo. Needs a full sitting.",
      "discoveryLink": null,
      "addedAt": "2026-08-20T10:00:00Z"
    }
  ]
}
```

`format` is a discriminator. Without it, the importer's first job would be guessing
whether an arbitrary JSON file was ever meant for it.

`formatVersion` is an integer, bumped on any breaking change. An importer accepts its own
version and anything below it, migrating as needed, and refuses anything above with a
message naming the version it needs.

`releaseCount` is redundant with the array length on purpose. A mismatch means truncation,
which is worth catching before writing anything.

### Fields

| Field | Type | Notes |
|---|---|---|
| `artist` | string | Required |
| `title` | string | Required |
| `releaseYear` | integer or null | The album's first release year, not the pressing's |
| `country` | string or null | ISO 3166-1 alpha-2, or a free-text area name |
| `albumArtUrl` | string or null | A URL, not the image. See below. |
| `musicbrainzId` | string or null | Match key. A release group id, see below |
| `genres` | array of strings | Sorted, may be empty |
| `streamingLinks` | object | Service name to URL, may be empty |
| `status` | `"QUEUED"` or `"LISTENED"` | |
| `rating` | number or null | 0.5 to 5.0, in half steps |
| `didNotFinish` | boolean | |
| `dateListened` | string or null | RFC3339 UTC |
| `notes` | string or null | |
| `discoveryLink` | string or null | |
| `addedAt` | string | RFC3339 UTC, when *you* added it |

`addedAt` is the tracking row's `created_at`. Renamed because "created" in a file a human
may open reads as though it were about the album rather than about the record of it.

Every timestamp is RFC3339 in UTC, exactly as stored, so string ordering is chronological
ordering.

## CSV

One row per release, a header row, `,` separated, RFC 4180 quoting, UTF-8 with no byte
order mark. Headers are the JSON field names, unchanged, so the two formats do not need a
mapping table between them.

Two columns cannot be flat:

- **`genres`** is joined with `; `. Chosen over JSON because genres are the column people
  actually want to read and filter in a spreadsheet. A genre containing `;` would break
  the round trip; none of MusicBrainz's do, and import splits on `;` and trims.
- **`streamingLinks`** is a JSON object in a single quoted cell. A delimited encoding
  would need escaping inside URLs, which already contain every delimiter worth choosing.
  It is unreadable in a spreadsheet and that is accepted: nobody reads streaming links in
  a spreadsheet.

Empty cells are null. Booleans are `true` and `false` lowercase. Records are separated by
`\n`; RFC 4180 asks for CRLF and reading accepts either.

CSV carries no envelope, so it has no version and no format discriminator. Import
therefore validates it by its header row, and a CSV missing `artist` or `title` is
rejected.

### Reading one a person typed

The point of CSV is that you can write one by hand or export it from something else, so
reading is deliberately more generous than writing:

- Headers are matched ignoring case and surrounding spaces, and unknown columns are
  ignored rather than refused.
- A leading byte order mark is stripped. Excel writes one.
- Everything but `artist` and `title` may be missing entirely. A missing `status` reads as
  `QUEUED`, the state every release passes through.
- `didNotFinish` accepts `1`, `0`, `yes` and `no` alongside `true` and `false`.
- A bare `YYYY-MM-DD` in either date column becomes midnight UTC, because everything
  downstream sorts and groups these as strings and two shapes in one column would quietly
  break the activity chart.
- Genres are trimmed, dropped when empty, and deduplicated case-insensitively, matching
  what the catalog would hold anyway.

## What is not exported

- **Local row ids**, for the reason above.
- **The local user id.** A sentinel with one value; sync would assign a real one.
- **Catalog rows you no longer track.** Deleting a release keeps its catalog entry, which
  is an implementation detail of the two-table split, not part of your library.
- **The full-text index**, which is derived and rebuilt by the triggers on insert.
- **Artists to check.** Not yet part of the format. Adding them is a version change: a new
  top-level list in JSON, and a separate file or no support in CSV, which holds one kind of
  row. See [ADR 0007](adr/0007-artists-to-check.md).
- **Album artwork itself.** Only the URL. Artwork is tens of megabytes for a modest
  library and the Cover Art Archive is a better custodian of it than a backup file. The
  consequence is that a restored library shows no art for any release whose URL has since
  rotted; refreshing metadata repairs those.

## Import behaviour

Import is **idempotent**: running the same file twice changes nothing the second time.

When an incoming release matches an existing one, the caller chooses:

- **Skip** (default). The existing release is left alone. Safe for merging a friend's
  export into your library, or re-running an import you are not sure completed.
- **Overwrite.** Every field from the file replaces what is there, including a null
  clearing a value.

There is deliberately no "merge" mode. Merging needs a rule for every field, and the
honest rule for a rating or a note is that there is no rule: one of them is simply wrong,
and only the person can say which.

Overwrite replaces the shared catalog entry too. Adding does not: when a release is new to
you but its catalog row already exists, that row is left exactly as it is, for the same
reason `create()` leaves a matched one alone. The catalog is content-addressed shared data,
and an import that is adding *tracking* has no business rewriting it.

Import reports counts of added, overwritten and skipped releases, plus any rows rejected
for failing validation, so a partial import is visible rather than silent. Each release is
written in its own transaction: a row that cannot be stored joins the rejected list instead
of abandoning the rows after it.

A row is rejected, rather than quietly corrected, when its artist or title is blank, its
status is a word other than `QUEUED` or `LISTENED`, its rating is outside 0.5 to 5.0 or not
on a half step, or a date is neither a plain date nor an RFC3339 timestamp. Silently
rounding someone's rating would be the same invisible edit the format refuses to make when
it declines to offer a merge mode.

Three failures stop the whole file before anything is written, because each one means the
file is not what it claims to be: a missing or wrong `format`, a `formatVersion` newer than
this build reads, and a `releaseCount` that disagrees with the array.

Genres arriving from a file go through the same case-insensitive find-or-create as any
other path, so importing `Jazz` into a library that has `jazz` does not create a second.

## Versioning

`formatVersion` starts at 1. It changes only when an older importer would read a newer
file wrongly. Adding an optional field does not qualify, because an old importer ignoring
an unknown key loses nothing it knew how to use. Renaming, removing, or changing the
meaning of a field does.
