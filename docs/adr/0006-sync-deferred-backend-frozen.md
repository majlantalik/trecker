# 0006. Sync is deferred, and the old backend is frozen

- **Status:** Accepted
- **Date:** 2026-09-11

## Context

[ADR 0001](0001-local-first-tauri-app.md) removed the server, and with it the only thing
that kept a library in step across a person's devices. A desktop library on one machine
covers the use Trecker was built for. Sync between devices is plausible later, possibly as
a paid feature, but nothing yet shows it is wanted.

The Spring Boot app in `backend/` already has the pieces a sync server would need:
email and password accounts with cookie-based JWTs, a Postgres schema, and the same
catalog and tracking model the desktop app kept.

## Decision

- **Sync is not built** until the local app has shown it is worth building.
- **`backend/` is frozen, not deleted.** No features are added to it. It stays as the
  starting point for a sync server.
- **The desktop schema keeps sync possible at no cost now.** Catalog rows are
  content-addressed and tracking rows carry `user_id` and `updated_at`. See
  [ADR 0002](0002-sqlite-with-catalog-and-tracking-split.md).

If sync is built, the plan is:

- **Unfreeze `backend/`,** remove every controller except authentication, and add a sync
  endpoint.
- **Catalog rows need no conflict resolution.** They are keyed by MusicBrainz release group,
  so the same album is the same row on every device.
- **Tracking rows resolve last-write-wins per field,** using `updated_at`. There is one
  writer per device, so field-level last-write-wins covers it.

## Alternatives considered

- **Build sync now.** It brings back a hosted server, accounts and uptime before anyone has
  asked for them, which is the cost ADR 0001 removed.
- **Delete `backend/`.** Cleaner, and it throws away working authentication and a schema
  that a sync server would otherwise rebuild from nothing. Git would keep it, but a frozen
  directory is easier to find than a commit.
- **Sync through a file-sharing service,** such as the SQLite file in a synced folder.
  A sync service copies whole files, so two devices writing between syncs leaves one
  device's changes discarded or a conflict copy to sort out by hand. With WAL mode the
  database and its write-ahead log can also be synced out of step.
- **Export and import as manual sync.** Already possible through
  [ADR 0005](0005-library-export-format.md), and adequate for moving machines. It is not
  sync: import never deletes and offers no merge, by design.

## Consequences

- **A library lives on one device.** Moving it means an export and an import.
- **`backend/` drifts from the desktop app.** It still has the `spotify_id` column and the
  Spotify resolver, and it keys the catalog by MusicBrainz *release*, not release group.
  Unfreezing it starts with bringing its schema in line with the desktop migration.
- **Nothing in the desktop schema has to change to add sync.**
- **The repository carries a directory nobody runs,** which is a real cost to anyone reading
  the tree. `CLAUDE.md` marks it frozen so it is not mistaken for live code.
