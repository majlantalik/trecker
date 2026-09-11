# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Trecker is a **local-first Tauri 2 desktop app**: a Vue 3 frontend in a native webview,
a Rust core, and a SQLite file on the user's machine. There is no server and no account.

It used to be a Spring Boot web app. `backend/` is still here but **frozen** — see
[TAURI-MIGRATION.md](TAURI-MIGRATION.md) for what moved and why. Do not add features
there; it exists as the starting point for an optional sync server.

## Development environment

Everything runs from the repo root. There is no Docker in the loop any more.

```bash
npm install            # installs the Tauri CLI at the root, once
npm run dev            # Vite + the Tauri window, hot reload
npm run build          # production bundle
npm run build:nobundle # production binary only, much faster
npm test               # frontend tests
cargo test --manifest-path src-tauri/Cargo.toml --lib   # Rust tests
```

**Linux needs the webview headers** before anything will compile:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
  build-essential curl wget file libssl-dev pkg-config
```

Use `cargo check` while iterating; a full `cargo build` writes gigabytes to
`src-tauri/target/`.

### Tests

44 Rust tests and 30 frontend tests. The Rust integration tests in
`src-tauri/src/repo/integration.rs` run against a real temporary SQLite file through the
real migration, so they catch actual SQL errors.

Three network tests hit the live MusicBrainz and Cover Art Archive services and are
excluded from normal runs:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib -- --ignored --test-threads=1
```

`--test-threads=1` is required: the MusicBrainz rate limiter lives on the `Resolver` and
each test builds its own, so parallel runs trip the limit and get a 503.

## Architecture

```
frontend/     Vue 3 + Vite + PrimeVue. Unchanged from the web app except src/api/.
src-tauri/    The Rust core.
  migrations/ sqlx migrations, replacing Liquibase
  src/
    commands.rs   the #[tauri::command] surface, 15 commands
    db.rs         pool, pragmas, migration runner, DbInfo diagnostics
    domain.rs     wire types, mirroring frontend/src/types/index.ts
    error.rs      AppError, serialized to the frontend as { code, message }
    repo/         sqlx queries; filter.rs is the UserReleaseSpecification port
    resolve/      MusicBrainz + Cover Art Archive
backend/      FROZEN. Spring Boot, kept only for a future sync server.
```

### The command boundary

The frontend's only contact with the core is `frontend/src/api/*.ts`, about 170 lines.
Every function there maps 1:1 onto a Rust command, and `frontend/src/api/commands.test.ts`
asserts every command name and argument key, so a rename on either side fails in CI.

The Pinia stores and all views go through that layer and know nothing about Tauri. **Keep
it that way.** If a view starts importing `invoke` directly, the seam has leaked.

### Data model: two-table split

Preserved from the web app even though there is exactly one local user, because it is what
would make sync tractable later:

- `releases` — deduplicated catalog, keyed by `spotify_id` or `musicbrainz_id` (both
  UNIQUE). Content-addressed, so these rows cannot conflict between devices.
- `user_releases` — per-user tracking. Status, rating, notes, `did_not_finish`,
  `date_listened`, `discovery_link`. Carries `user_id` (a local sentinel) and `updated_at`
  purely so sync would not need a schema change.

**Delete removes only the `user_releases` row.** The catalog row stays.

The `id` in every API response is the **catalog release id**, not the tracking row id.
`created_at` comes from the **tracking row**, because it means "when you added this".

### Metadata resolution

`releases_resolve` → `resolve/mod.rs` → MusicBrainz for identity and genres, Cover Art
Archive for artwork, both best-effort.

**Spotify, Tidal and YouTube are link-only and will stay that way.** All three need a
credential that cannot ship in a binary. A pasted streaming URL is saved as a link with
sharing parameters stripped; nothing is queried through it. Bandcamp and Apple Music put
the artist and album in the URL path, so those are read and searched.

## Key gotchas

**Never `cargo build --release` to produce a shippable binary.** It bypasses the Tauri CLI
and leaves `devUrl` embedded, so the binary tries to reach localhost:5173. Use
`npm run build`.

**The Tauri CLI must run from the repo root.** It only searches subfolders, so it cannot
find `src-tauri/` from inside `frontend/`. `beforeDevCommand` and `beforeBuildCommand`
also resolve from the root, not from `src-tauri/`.

**The sort whitelist in `repo/filter.rs` is load-bearing.** `sort` is a client-supplied
string. JPA used to make that safe and raw SQL does not. Anything not on the list is an
error, never a fallback.

**Genre filtering uses `EXISTS`, not a join.** The old `query.distinct(true)` gotcha does
not carry over, and reintroducing a join would bring it back.

**Text sorting uses `COLLATE NOCASE`,** because SQLite's default collation puts every
capital before every lowercase letter.

**`ORDER BY` always needs the `ur.id` tiebreaker.** Without it, rows equal on the sort
column swap between page fetches and a release appears on two pages.

**FTS5 is for catalog autocomplete only.** The library filter box uses substring `LIKE`,
because FTS matches whole tokens and would stop finding "phere" inside "Stratosphere".

**Never pass user input to FTS5 raw.** It has its own query language; `resolve/filter.rs`
quotes every token and appends `*`.

**SQLite pragmas are per-connection.** Checking them from a separate reader tells you
nothing about the app's pool. Use the Settings view's Database section, which reads them
on a pool connection.

**MusicBrainz needs a real `User-Agent` and one request per second.** The limiter is a gate
over the last-request time on a single `Resolver` instance managed by Tauri. Do not
construct a second one.

**`primeicons` is a separate package** and must stay listed in `package.json`.

**`@primevue/themes` 4.5.x** is deprecated upstream but the Aura theme still works; do not
migrate to `@primeuix/themes` yet.

## Database

sqlx migrations in `src-tauri/migrations/`, applied at startup. The file lives in Tauri's
`app_data_dir()`: `~/.local/share/cz.mtulek.trecker/trecker.db` on Linux, `%APPDATA%` on
Windows, `~/Library/Application Support` on macOS.

To add a migration, drop `NNNN_description.sql` into `src-tauri/migrations/`. sqlx picks it
up by filename order; there is no master file to edit.

Current: `0001_initial.sql`.

**Known limitation:** the FTS tokenizer folds accents but not ligatures, so `ros` finds
*Sigur Rós* while `agaetis` does not find *Ágætis byrjun*. That is unicode61 as documented.

## Verifying UI changes

There is no browser to point Playwright at. Build with `npm run build:nobundle`, run
`src-tauri/target/release/trecker`, and read the accessibility tree:

```bash
orca-ide computer get-app-state --app trecker --no-screenshot --json
```

On Wayland, screenshots need a desktop portal grant that is not available to automation,
and keyboard input needs window focus that cannot be taken. The accessibility tree is the
reliable channel — values render into it from `<p>` elements but not from `<div>`s, which
is why the Settings rows are paragraphs.
