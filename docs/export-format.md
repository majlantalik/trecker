# Library export format

A specification for exporting a Trecker library and importing it back, on the same
machine or another one. Written before either side is built, so the two are implemented
against one document rather than against each other.

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

This mirrors the precedence `repo::releases::create` already uses to dedupe the catalog,
minus the Spotify step, so import reuses logic that is already tested rather than
inventing a second notion of sameness.

There is no `spotifyId` in this format. The column exists in the schema and the dedup
still checks it, but nothing has written one since Spotify became link-only: the resolver
returns null unconditionally. Exporting a field that is null in every row of every library
would be inviting a future importer to depend on it.

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
      "musicbrainzId": "266e8eb6-244f-450d-b419-7e3cdf815d4c",
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
| `musicbrainzId` | string or null | Match key |
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

Empty cells are null. Booleans are `true` and `false` lowercase.

CSV carries no envelope, so it has no version and no format discriminator. Import
therefore validates it by its header row, and a CSV missing `artist` or `title` is
rejected.

## What is not exported

- **Local row ids**, for the reason above.
- **`spotifyId`**, which is null in every row and has been since Phase 4.
- **The local user id.** A sentinel with one value; sync would assign a real one.
- **Catalog rows you no longer track.** Deleting a release keeps its catalog entry, which
  is an implementation detail of the two-table split, not part of your library.
- **The full-text index**, which is derived and rebuilt by the triggers on insert.
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

Import reports counts of added, overwritten and skipped releases, plus any rows rejected
for failing validation, so a partial import is visible rather than silent.

Genres arriving from a file go through the same case-insensitive find-or-create as any
other path, so importing `Jazz` into a library that has `jazz` does not create a second.

## Versioning

`formatVersion` starts at 1. It changes only when an older importer would read a newer
file wrongly. Adding an optional field does not qualify, because an old importer ignoring
an unknown key loses nothing it knew how to use. Renaming, removing, or changing the
meaning of a field does.
