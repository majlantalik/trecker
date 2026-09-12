# 0002. SQLite, keeping the catalog and tracking split

- **Status:** Accepted
- **Date:** 2026-09-11

## Context

[ADR 0001](0001-local-first-tauri-app.md) moved Trecker onto the user's machine, so the
Postgres database had to be replaced by something embedded.

The web app's schema had two tables where one might be expected. `releases` was a shared,
deduplicated catalog of albums. `user_releases` held each person's tracking: status,
rating, notes, dates. With accounts gone there is exactly one user, which makes the split
look like leftover multi-user machinery.

It is also the thing that would make sync between devices tractable, if that is ever
built.

## Decision

- **One SQLite file** in the platform's app data directory, opened through `sqlx` from
  Rust, with typed queries and migrations applied at startup.
- **The catalog and tracking split is kept.** Catalog rows are content-addressed by a
  MusicBrainz id, so two devices adding the same album produce the same row and cannot
  conflict. Only tracking rows would ever need conflict resolution.
- **Tracking rows keep a `user_id`,** set to a fixed local sentinel, and gain an
  `updated_at` column. Neither does anything today. Both mean sync would not need a schema
  change.
- **Deleting a release removes only its tracking row.** The catalog row stays.
- The connection runs in **WAL mode with foreign keys on**, and catalog search uses an
  **FTS5** index kept in step by triggers.

## Alternatives considered

- **Collapse into a single table.** Simpler queries today, and a schema migration plus a
  conflict model for every column the day sync is wanted.
- **`tauri-plugin-sql`.** It exposes SQL to the JavaScript side. Trecker wants its queries
  typed and in Rust behind a narrow command surface, so the webview can ask for releases
  but cannot run arbitrary SQL.
- **A JSON file.** No filtering, sorting or pagination without loading everything, no
  transactions, and no search index.
- **Import the Postgres data.** Rejected because it only ever held throwaway test rows on
  one machine.

## Consequences

- **Every API response uses the catalog id as `id`,** because that is what the frontend
  holds and sends back. `createdAt` comes from the tracking row, since it means when you
  added the album.
- **Queries that JPA used to generate are now handwritten SQL.** That moved several safety
  properties into code: the sort field whitelist, genre filtering with `EXISTS`, and a
  deterministic tiebreaker in every `ORDER BY`. `CLAUDE.md` lists them.
- **Pragmas are per connection,** so they are set on the pool, and the Info view checks
  them on a pool connection.
- **The whole library is one file** a person can see and copy.
- **Editing the initial migration is only possible while a single install exists.** sqlx
  checksums applied migrations, so it has been done twice while nothing had shipped. After
  the first release every schema change must be a new migration.
