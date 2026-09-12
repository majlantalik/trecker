# Trecker

A personal music library and listening tracker. Queue records you mean to hear, log them
once you have, and see what you actually listened to.

It runs on your machine and stores everything in a single SQLite file you own. No account,
no server, no network required to use it.

**Core flow:** Quick Add → Queue → Mark as Listened → Library

---

## User Guide

### Adding releases

Type into the **Quick Add** bar at the top of any page.

- **Artist and album** (`Slint - Spiderland`) — looked up on MusicBrainz. Artist, title,
  the album's original release year, country and genres come back filled in, with cover
  art from the Cover Art Archive.
- **Just an album title** — the same lookup, less precisely.
- **A Bandcamp or Apple Music link** — the artist and album are read out of the URL and
  searched, and the link is saved.
- **A Spotify, Tidal or YouTube link** — the link is saved, but nothing is looked up
  through it. See below.
- **Anything else** — the form opens for manual entry.

A pre-filled form opens either way. Adjust anything, then **Add to Queue**.

Existing releases in your library appear as suggestions while you type, so adding
something you already have does not create a duplicate.

### Streaming links

Spotify, Tidal and YouTube links are saved but nothing is looked up through them. Sharing
parameters like `si=` are stripped first, since they identify whoever sent you the link.
The reasoning is in [ADR 0003](docs/adr/0003-streaming-services-are-link-only.md).

### Queue

Everything you have not listened to yet.

- **Log** opens the Quick Log modal.
- **Pick one for me** chooses a queued release at random.
- The title opens the full Entry page.

### Quick Log

| Field | Description |
|---|---|
| Rating | half-stars, 0.5 to 5 |
| Genres | pre-filled from MusicBrainz, editable |
| Country | pre-filled, editable |
| Notes | free text |
| Did not finish | for abandoned listens |

**Log it** saves and returns; **Log + Details** saves and opens the Entry page.

### Keyboard shortcuts

Press <kbd>?</kbd> for the list. <kbd>Shift</kbd> <kbd>Shift</kbd> opens quick add from
anywhere, in the style of a JetBrains IDE; <kbd>Ctrl</kbd>+<kbd>K</kbd> does the same for
anyone who reaches for that first.

Navigation follows Gmail: <kbd>G</kbd> then the first letter of where you are going, so
<kbd>G</kbd> <kbd>L</kbd> for Library. A hint appears while the sequence is half-entered.

### Refreshing metadata

Metadata is fetched once when you add a release. If it came back wrong or incomplete, the
refresh button on a release's page fetches it again. It replaces artist, title, year,
country, artwork and genres, and never touches your rating, notes, dates or links.

### Library

Everything you have listened to, as a table or a grid. Filter by free-text search, status,
genre, country, year, rating range and the did-not-finish flag. Countries show as flags
and names, and the country filter offers only the ones you actually have.

### Stats

Listening activity by month, breakdowns by genre and country, your top-rated releases, and
a ranked year-end list for any year.

### Where your data lives

One SQLite file:

| Platform | Path |
|---|---|
| Linux | `~/.local/share/cz.mtulek.trecker/trecker.db` |
| macOS | `~/Library/Application Support/cz.mtulek.trecker/trecker.db` |
| Windows | `%APPDATA%\cz.mtulek.trecker\trecker.db` |

The Info view shows the exact path. Back it up by copying the file while the app is closed.

### Export and import

**Info → Backup** writes your whole library to a file and reads one back.

- **JSON** is the complete record and the one to keep for a backup or a move to another
  machine.
- **CSV** holds the same releases in a shape a spreadsheet can read. It round-trips too.

Neither carries the album artwork itself, only the address it lives at, so a library
restored years later shows no art for any release whose URL has rotted. Refreshing metadata
repairs those.

On the way in, a release is matched by its MusicBrainz id, then by artist and title. You
choose what happens when it matches something you already have: **Keep mine**, the default,
leaves your copy alone, and **Replace mine** takes every field from the file. Importing the
same file twice changes nothing the second time.

Rows the file gets wrong are listed rather than silently dropped, and the rest still land.
The format is specified in [docs/export-format.md](docs/export-format.md), and the choices
behind it are in [ADR 0005](docs/adr/0005-library-export-format.md).

---

## Developer Guide

The significant architecture decisions, and the alternatives they beat, are recorded in
[docs/adr/](docs/adr/README.md).

### Tech stack

| Layer | Technology |
|---|---|
| Shell | Tauri 2 (system webview, ~8 MB binary) |
| Frontend | Vue 3 + Vite + PrimeVue 4 (Aura theme) |
| State | Pinia |
| Core | Rust |
| Database | SQLite via sqlx, with FTS5 |
| HTTP | reqwest (rustls) |
| Charts | vue-chartjs + Chart.js |

The frontend talks to Rust through Tauri commands. There is no HTTP API and no auth layer.

### Prerequisites

| Tool | Version |
|---|---|
| Rust | 1.77+ (1.95 in use) |
| Node.js | 20+ |

On Linux, the webview development headers:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
  build-essential curl wget file libssl-dev pkg-config
```

### Running

All commands from the repo root.

```bash
npm install            # Tauri CLI, once
npm run dev            # Vite + the Tauri window, hot reload
npm run build          # release bundles: .deb and .AppImage
npm run build:deb      # .deb only, skips the 77 MB AppImage
npm run build:fast     # debug .deb, when you just need something installable
npm run build:nobundle # binary only
```

Measured on 24 cores after a one-line Rust change: `npm run dev` rebuilds in 5.5 s,
`build:fast` in 21 s, `build` in 67 s. The release profile is deliberately the slowest
configuration, fat link-time optimisation in one codegen unit, for the smallest binary.
Iterate with `npm run dev`, not with `npm run build`.

> `cargo build --release` on its own produces a binary that tries to reach the dev server.
> Always build through the Tauri CLI.

### Tests

```bash
npm test           # 84 frontend tests
npm run test:rust  # 109 Rust tests
npm run test:net   # 8 tests against the live metadata services
```

The Rust integration tests run against a real temporary SQLite file through the real
migration. The network tests are excluded from `test:rust` and run single-threaded,
because the MusicBrainz rate limiter is per-`Resolver` and parallel tests trip it.

### Project structure

```
trecker/
├── package.json              workspace root; owns the Tauri scripts
├── frontend/                 Vue 3 application
│   └── src/
│       ├── api/              releases.ts, genres.ts, stats.ts, info.ts,
│       │                     library.ts, dialog.ts
│       │                     the only place that knows about Tauri
│       ├── components/
│       │   ├── layout/       AppLayout, AppSidebar
│       │   ├── library/      LibraryFilters, ViewToggle
│       │   ├── queue/        QueueActions
│       │   ├── release/      QuickAddBar, ReleaseCard, ReleaseForm,
│       │   │                 QuickLogModal, GenreTagInput
│       │   └── stats/        ActivityChart, BreakdownChart, TopRatedList, YearEndList
│       ├── composables/      keyboard shortcuts, useLibraryTransfer
│       ├── stores/           releases.ts, genres.ts, stats.ts (Pinia)
│       ├── types/index.ts    single source of truth for TS interfaces
│       └── views/            QueueView, LibraryView, StatsView, EntryView, InfoView
├── src-tauri/                Rust core
│   ├── migrations/           sqlx migrations
│   ├── tauri.conf.json
│   └── src/
│       ├── commands.rs       the #[tauri::command] surface
│       ├── db.rs             pool, pragmas, migrations, diagnostics
│       ├── domain.rs         wire types
│       ├── error.rs          AppError → { code, message }
│       ├── library/          the export and import file format
│       ├── repo/             sqlx queries, filter builder, integration tests
│       └── resolve/          MusicBrainz + Cover Art Archive
├── backend/                  FROZEN Spring Boot app; see ADR 0006
├── docs/adr/                 architecture decision records
└── docs/export-format.md     the library file format, specified
```

### Configuration

There is none. No `.env`, no API keys, no connection strings. MusicBrainz and the Cover
Art Archive are open, and everything else is local.

### Command surface

Nineteen Tauri commands, mapping 1:1 onto `frontend/src/api/*.ts`:

| Command | Purpose |
|---|---|
| `releases_resolve` | Resolve a URL or search query to metadata |
| `releases_create` | Add a release to the queue |
| `releases_list` | List with filtering, sorting, pagination |
| `releases_random` | Pick a random queued release |
| `releases_get` | Fetch one release |
| `releases_update` | Partial update; `status=LISTENED` stamps `dateListened` |
| `releases_delete` | Stop tracking; the catalog row stays |
| `releases_refresh_metadata` | Re-fetch catalog fields for an existing release |
| `releases_search_catalog` | Full-text autocomplete over the catalog |
| `genres_list` | Every known genre |
| `countries_list` | Countries present in the library, for the filter |
| `stats_activity` | Listening count by year and month |
| `stats_by_genre` | Breakdown by genre |
| `stats_by_country` | Breakdown by country |
| `stats_top_rated` | Top-rated listened releases |
| `stats_year_end` | Ranked list for a year |
| `info_db` | Database path, size, schema version, pragmas |
| `library_export` | Write the whole library to a JSON or CSV file |
| `library_import` | Read one back, skipping or overwriting what matches |

`frontend/src/api/commands.test.ts` asserts every name and argument key, so a rename on
either side of the boundary fails in CI rather than at runtime.

### Data model

A shared catalog plus per-user tracking. Why the split is kept with one user:
[ADR 0002](docs/adr/0002-sqlite-with-catalog-and-tracking-split.md). Why the key is a
release group: [ADR 0004](docs/adr/0004-release-group-is-album-identity.md).

- `releases` — deduplicated catalog, keyed by the album's MusicBrainz release group.
- `user_releases` — status, rating, notes, dates. Deleting one leaves the catalog row.

The `id` in an API response is the catalog release id; `createdAt` is from the tracking
row, meaning when you added it.

### Database migrations

sqlx applies everything in `src-tauri/migrations/` at startup, in filename order. To add
one, create `NNNN_description.sql`. There is no master file to register it in.

### Known issues

- **FTS folds accents but not ligatures.** Searching `ros` finds *Sigur Rós*; `agaetis`
  does not find *Ágætis byrjun*, though `agætis` does. Standard unicode61 behaviour.
- **`primeicons` is a separate package** and must be listed explicitly.
- **`@primevue/themes` 4.5.x** is deprecated upstream; the Aura theme still works.
- **Builds are unsigned.** macOS Gatekeeper and Windows SmartScreen will warn until code
  signing is set up.

---

## Roadmap

- [x] Local-first rewrite: Tauri 2 + SQLite, no server
- [x] MusicBrainz and Cover Art Archive metadata
- [x] Full-text catalog search
- [ ] Auto-update through `tauri-plugin-updater`, with a manifest on GitHub Releases
- [ ] GitHub Actions build for all three platforms
- [ ] macOS signing and notarisation (Apple Developer Program, 99 USD a year)
- [ ] Windows signing (the certificate needs a hardware token, which complicates CI)
- [ ] `.rpm` bundle (needs `rpmbuild` on the build machine; left out so the build does not fail)
- [x] Export and import, CSV and JSON
- [ ] Optional account-based sync between devices ([ADR 0006](docs/adr/0006-sync-deferred-backend-frozen.md))
- [x] Keyboard shortcuts (in-app)
- [ ] Global quick-add shortcut (needs a tray icon, and Wayland restricts global key capture)
- [ ] Mobile
