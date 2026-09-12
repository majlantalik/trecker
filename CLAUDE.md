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
npm run build          # release bundles: .deb and .AppImage
npm run build:deb      # .deb only, skips the 77 MB AppImage
npm run build:fast     # debug .deb, for when you need something installable now
npm run build:nobundle # binary only
npm test               # frontend tests
npm run test:rust      # Rust tests
npm run test:net       # the three network tests, excluded from test:rust
```

### Build times

Measured on 24 cores, one-line Rust change, warm cache:

| | time |
|---|---|
| `npm run dev` rebuild | 5.5 s |
| `npm run build:fast` | 21 s |
| `npm run build` | 74 s |

**Do not reach for `npm run build` while iterating.** The release profile uses fat LTO in
a single codegen unit, which optimises the whole binary at once and so leaves 23 of 24
cores idle. That is deliberate: it halves the binary, 4.2 MB against 8.2 MB for thin LTO,
and release builds happen once per release. `npm run dev` is the loop.

If 74 s ever becomes the bottleneck, a faster linker (`mold`) is the next lever; none is
installed.

**Linux needs the webview headers** before anything will compile:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
  build-essential curl wget file libssl-dev pkg-config
```

Use `cargo check` for a syntax and type pass in about a second; a full `cargo build`
writes gigabytes to `src-tauri/target/`.

### Tests

47 Rust tests and 31 frontend tests. The Rust integration tests in
`src-tauri/src/repo/integration.rs` run against a real temporary SQLite file through the
real migration, so they catch actual SQL errors.

Five further tests hit the live MusicBrainz and Cover Art Archive services and are
excluded from normal runs. `npm run test:net` runs them, and passes `--test-threads=1`,
which is required: the MusicBrainz rate limiter lives on the `Resolver`, each test builds
its own, and parallel runs trip the limit and get a 503.

Run them after touching `resolve/`. Every bug in that module so far was found this way and
by none of the offline tests.

## Architecture

```
frontend/     Vue 3 + Vite + PrimeVue. Unchanged from the web app except src/api/.
src-tauri/    The Rust core.
  migrations/ sqlx migrations, replacing Liquibase
  src/
    commands.rs   the #[tauri::command] surface, 17 commands
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

**Never trust the pressing a search matched.** MusicBrainz search returns one *release*,
and which one is close to arbitrary: its year is that edition's, its country is wherever
that edition was sold, and it often has no cover art even when the album does. The release
*group* is the album, and `mb_release_details` fetches it in the same request that gets
genres and the artist's country. All three of those were live bugs.

**`releases_refresh_metadata` is the repair path.** Metadata is written once at add time
and never revisited, so any improvement here only helps new additions until a user hits
refresh. It rewrites catalog fields only and never touches rating, notes, status, dates or
links.

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
nothing about the app's pool. Use the Info view's Database section, which reads them
on a pool connection.

**MusicBrainz needs a real `User-Agent` and one request per second.** The limiter is a gate
over the last-request time on a single `Resolver` instance managed by Tauri. Do not
construct a second one.

**Country values are not always ISO codes.** MusicBrainz usually gives a two-letter code,
but the field is editable and the resolver falls back to an artist's area name like
"England". `utils/country.ts` degrades to showing the raw string rather than guessing, and
gives no flag to user-assigned codes such as XW.

**Shortcuts live in one catalogue.** `composables/shortcuts.ts` is what the help dialog
renders, so adding a binding without adding it there gives you a shortcut nobody can
discover. A test asserts every entry lands in a group the dialog shows.

**Synthetic keyboard input does not reach the webview** through the OS-level tooling on
this machine, so shortcuts cannot be verified that way. `press-key` reports success and
nothing happens, for modifier and function keys alike. Cover the logic with unit tests and
have a human press the keys.

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
