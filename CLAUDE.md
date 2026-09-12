# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Trecker is a **local-first Tauri 2 desktop app**: a Vue 3 frontend in a native webview,
a Rust core, and a SQLite file on the user's machine. There is no server and no account.
Why: [ADR 0001](docs/adr/0001-local-first-tauri-app.md).

It used to be a Spring Boot web app. `backend/` is still here but **frozen**. Do not add
features there; it exists as the starting point for an optional sync server. Why:
[ADR 0006](docs/adr/0006-sync-deferred-backend-frozen.md).

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
npm run test:net       # the eleven network tests, excluded from test:rust
```

### Build times

Measured on 24 cores, one-line Rust change, warm cache:

| | time |
|---|---|
| `npm run dev` rebuild | 5.5 s |
| `npm run build:fast` | 21 s |
| `npm run build` | 67 s |

**Do not reach for `npm run build` while iterating.** The release profile uses fat LTO in
a single codegen unit, which optimises the whole binary at once and so leaves 23 of 24
cores idle. That is deliberate: measured at 7.7 MB against 10.0 MB for thin LTO across 16
units, before the tray and single-instance support took it to 8.8 MB, and release builds
happen once per release. `npm run dev` is the loop.

If 67 s ever becomes the bottleneck, a faster linker (`mold`) is the next lever; none is
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

147 Rust tests and 141 frontend tests. The Rust integration tests in
`src-tauri/src/repo/integration.rs` run against a real temporary SQLite file through the
real migration, so they catch actual SQL errors.

Eleven further tests hit the live MusicBrainz and Cover Art Archive services and are
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
    commands.rs   the #[tauri::command] surface, 28 commands
    covers.rs     the on-disk cover cache and the cover: protocol
    desktop.rs    --quick-add, the tray, and what closing the window does
    settings.rs   this computer's preferences, as a JSON file
    db.rs         pool, pragmas, migration runner, DbInfo diagnostics
    domain.rs     wire types, mirroring frontend/src/types/index.ts
    error.rs      AppError, serialized to the frontend as { code, message }
    library/      the export and import file format; no SQL, no Tauri
    repo/         sqlx queries; filter.rs is the UserReleaseSpecification port
    resolve/      MusicBrainz + Cover Art Archive
backend/      FROZEN. Spring Boot, kept only for a future sync server.
docs/adr/     architecture decision records
```

**The reasons live in [docs/adr/](docs/adr/README.md), not here.** This file states the
rules that follow from those decisions. Do not copy an ADR's reasoning into this file;
link to it. Before reversing one of those decisions, read its ADR. To overturn it, write a
new ADR and mark the old one superseded rather than editing it.

### The command boundary

The frontend's only contact with the core is `frontend/src/api/*.ts`, about 170 lines.
Every function there maps 1:1 onto a Rust command, and `frontend/src/api/commands.test.ts`
asserts every command name and argument key, so a rename on either side fails in CI.

The Pinia stores and all views go through that layer and know nothing about Tauri. **Keep
it that way.** If a view starts importing `invoke` directly, the seam has leaked.

### Data model: two-table split

Do not collapse these into one table. Why: [ADR 0002](docs/adr/0002-sqlite-with-catalog-and-tracking-split.md).

- `releases` — deduplicated catalog, keyed by `musicbrainz_release_group_id` (UNIQUE).
  `create()` never merges a release that has no id with another one; text matching
  belongs to import only.
- `user_releases` — per-user tracking. Status, rating, notes, `did_not_finish`,
  `date_listened`, `discovery_link`. Keep writing `user_id` (a local sentinel) and
  `updated_at` even though nothing reads them yet.

**Delete removes only the `user_releases` row.** The catalog row stays.

The `id` in every API response is the **catalog release id**, not the tracking row id.
`created_at` comes from the **tracking row**, because it means "when you added this".

### Metadata resolution

Typed text goes through `releases_search`, which returns up to ten candidates from one
search request, and then `releases_lookup` for the one chosen, which adds genres, country
and cover. A pasted link goes through `releases_resolve`, which picks one album itself.
MusicBrainz supplies identity and genres, the Cover Art Archive the artwork, both
best-effort.

**Search results stay thin.** Genres, country and the real cover each need a lookup, and at
one MusicBrainz request per second a list of ten would take ten seconds. Only the chosen
album is looked up. The list's covers come from an archive address that needs no request
and often 404s, so `prefillFromCandidate` never saves it.

**How typed text is searched** is in `search_query`. A spaced dash searches artist and title
separately. Anything else scores an exact title highest and otherwise requires every word
in the artist or the title. When that finds nothing, typically because of an extra word
such as a year, `loose_query` retries with the words alone. Change these only against live
results; `npm run test:net` covers the cases that shaped them.

**Spotify, Tidal and YouTube are link-only.** Do not add lookups through them or any
credential for them. A pasted streaming URL is saved as a link with sharing parameters
stripped. Bandcamp and Apple Music URLs are read for artist and album, then searched.
Why: [ADR 0003](docs/adr/0003-streaming-services-are-link-only.md).

**Use the release group, never a release.** Search, lookup, cover art and the stored id
all go through the group. Why: [ADR 0004](docs/adr/0004-release-group-is-album-identity.md).
The rules that follow:

- **Country is the credited artist's,** then their area name, then nothing. Never fall
  back to a pressing's country.
- **`rank_release_groups` only reorders hits tied on score.** Never filter by type, or EPs
  and singles become unfindable.
- **Refresh looks the stored group id up directly.** Only a release with no id is refreshed
  by searching its artist and title.
- **"Release group" never reaches the interface.** The UI says album and the export field
  stays `musicbrainzId`. Code, schema and developer docs use the precise term.

### Album covers

**Never bind a stored `albumArtUrl` to an image directly. Use `coverSrc()` from
`@/api/cache`.** It routes the URL through the `cover:` protocol, which `covers.rs` answers
from `app_cache_dir()/covers`, downloading on the first request. `img-src` has no `https:`,
so a direct binding does not load at all. `frontend/src/api/covers.guard.test.ts` scans
every `.vue` file for one.

- **The stored URL stays the source of truth.** Exports carry it and refresh replaces it.
  The cache is derived and safe to delete at any time, which the Info view does.
- **Covers are fetched at 250px.** The resolver prefers the Cover Art Archive's 250px
  thumbnail, and the cache rewrites stored 500px and 1200px archive URLs to 250px, falling
  back to the stored URL.
- **Only `https` URLs are fetched, and only images are kept.** The type is sniffed from the
  bytes, so an error page served with status 200 is never cached. Files over 5 MB are
  refused.
- **Cache file names are FNV-1a hashes** of `250:` plus the URL. Do not swap in std's
  `DefaultHasher`: its algorithm may change between Rust releases and orphan the cache.

### Desktop integration

**Quick add from outside the app is a command, not a global shortcut.** A person binds
`trecker --quick-add` to a key in their desktop's own settings. An app cannot grab a key on
Wayland, COSMIC's desktop portal has no GlobalShortcuts interface, and Tauri's
global-shortcut plugin only works through X11. `desktop.rs` has the detail. Do not add that
plugin expecting it to work here.

**`tauri-plugin-single-instance` must stay the first plugin registered.** A second launch
hands its arguments to the running app and exits. It keys on the app identifier, so while
`npm run dev` is running, launching a release build hands off to the dev app instead of
starting. Quit the dev app before testing a release binary.

**A quick add request reaches the page two ways.** The launch that starts the app stores it
in `LaunchAction`, and the page takes it once on mount, because an event emitted during
startup would arrive before anything listens. Later launches and the tray menu emit
`quick-add`. `useLaunchActions` handles both.

**Closing hides the window only when a tray icon exists.** If creating the tray failed, a
hidden window would leave an app running with no visible way back, so closing quits.
`settings_update` applies the tray before saving, so a tray the desktop cannot show is an
error, not a saved choice that does nothing.

**Settings live in `app_config_dir()/settings.json`, not in the database.** They describe
this computer, not the library, so an export must not carry them.

### Export and import

**The format is specified in [docs/export-format.md](docs/export-format.md)**, and it is
the authority: where the document and `src-tauri/src/library/` disagree, the code is wrong.
Why the format is shaped as it is: [ADR 0005](docs/adr/0005-library-export-format.md).

`library/` decides what the bytes mean and touches neither SQL nor Tauri, so every decision
about the format is testable without a database. `repo::releases::import_one` does the
writing. The UI is one section of the Info view, driven by `useLibraryTransfer`.

**Validate before any SQL runs.** Status, rating range and timestamp shapes are checked
in `library/`, so a bad row is a reported rejection and never a database error. The import
loop's error arm is for surprises, not the normal path.

**Native file dialogs come from `tauri-plugin-dialog`, and Rust does the file I/O.** The
webview gets a path from the dialog and nothing else; do not grant it filesystem
permissions.

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

**`@/api/` is the only place that imports Tauri.** `dialog.ts` wraps the file pickers for
the same reason the others wrap `invoke`: the views and stores stay ordinary Vue and the
seam stays one directory wide.

**The content security policy is in `tauri.conf.json`, which cannot hold comments.**
Nothing the webview loads comes from the network. `img-src` allows `cover:` and
`http://cover.localhost`, the Windows form of the same scheme, and no remote host, so every
cover goes through the cache. Scripts and connections stay locked to the app.
`style-src 'unsafe-inline'` is required by PrimeVue.

**Clearing webview data wipes browser storage too.** The Info view's button calls
`clear_all_browsing_data`, which removes the HTTP cache along with cookies, `localStorage`
and IndexedDB. That is harmless only because Trecker uses none of them. If anything starts
keeping state in browser storage, change that button before shipping.

**`createWebHistory` works under Tauri.** A deep route resolves in a production build, so
the common advice to switch Tauri apps to hash history does not apply here.

**Fonts are bundled, never fetched.** Syne and Outfit come from `@fontsource` imports in
`main.ts`. Do not add a Google Fonts `<link>`: the app would fall back to system fonts
offline, the policy would have to open up, and a local-first app would contact a third
party on every launch.

**Queries use runtime `sqlx::query()`, not the `query!` macros.** The macros need a live
database or a checked-in `.sqlx` cache at build time, and cannot check the filter queries,
which are assembled at runtime. The integration tests catch a wrong column name instead.

**Restyle PrimeVue through its CSS variables, not plain declarations.** PrimeVue injects its
theme at runtime, after the app's stylesheet, so a one-class rule such as
`.my-dialog { background: ... }` loses to `.p-dialog` and never applies, with no error. Set
the variable the theme reads instead, such as `--p-dialog-background`, on the component's
root; `.tk-dialog` in `App.vue` does this for dialogs, so give every dialog that class.
`components/dialogs.guard.test.ts` fails for one that lacks it. The
name of any token's variable comes from `dt()` in `@primeuix/styled`, and the tokens are
listed in `@primeuix/themes/dist/aura/<component>`.

**A dialog focuses its content's `[autofocus]` element when it finishes opening, or else its
close button.** Focusing something yourself in `@show` is overridden a moment later. Mark the
element instead; `AlbumPicker` passes `autofocus` to its list through `pt`.

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
