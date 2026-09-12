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

### Why streaming links are not resolved

Spotify, Tidal and YouTube all require a developer credential to query, and a credential
shipped inside a desktop app is not a credential. The alternative would be asking you to
hold a paid subscription and register your own developer app before adding an album, which
is not a reasonable thing to ask.

So Trecker keeps the link and looks nothing up. Sharing parameters like `si=` are stripped
first, since they identify whoever sent you the link.

MusicBrainz and the Cover Art Archive need no account at all, which is why they do the
work.

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

---

## Developer Guide

### Tech stack

| Layer | Technology |
|---|---|
| Shell | Tauri 2 (system webview, ~4 MB binary) |
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
`build:fast` in 21 s, `build` in 74 s. The release profile is deliberately the slowest
configuration, fat link-time optimisation in one codegen unit, because it halves the
binary. Iterate with `npm run dev`, not with `npm run build`.

> `cargo build --release` on its own produces a binary that tries to reach the dev server.
> Always build through the Tauri CLI.

### Tests

```bash
npm test           # 31 frontend tests
npm run test:rust  # 47 Rust tests
npm run test:net   # 5 tests against the live metadata services
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
│       ├── api/              releases.ts, genres.ts, stats.ts, settings.ts
│       │                     the only place that knows about Tauri
│       ├── components/
│       │   ├── layout/       AppLayout, AppSidebar
│       │   ├── library/      LibraryFilters, ViewToggle
│       │   ├── queue/        QueueActions
│       │   ├── release/      QuickAddBar, ReleaseCard, ReleaseForm,
│       │   │                 QuickLogModal, GenreTagInput
│       │   └── stats/        ActivityChart, BreakdownChart, TopRatedList, YearEndList
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
│       ├── repo/             sqlx queries, filter builder, integration tests
│       └── resolve/          MusicBrainz + Cover Art Archive
├── backend/                  FROZEN Spring Boot app; see TAURI-MIGRATION.md
└── TAURI-MIGRATION.md        the migration record and remaining plan
```

### Configuration

There is none. No `.env`, no API keys, no connection strings. MusicBrainz and the Cover
Art Archive are open, and everything else is local.

### Command surface

Seventeen Tauri commands, mapping 1:1 onto `frontend/src/api/*.ts`:

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

`frontend/src/api/commands.test.ts` asserts every name and argument key, so a rename on
either side of the boundary fails in CI rather than at runtime.

### Data model

A shared catalog plus per-user tracking, kept from the web app because it is what would
make sync tractable later:

- `releases` — deduplicated catalog, keyed by `spotify_id` or `musicbrainz_id`.
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
- [ ] Signed and notarised builds, auto-update
- [ ] Export to CSV / JSON
- [ ] Optional account-based sync between devices
- [x] Keyboard shortcuts (in-app)
- [ ] Global quick-add shortcut (needs a tray icon; see the Wayland caveat in TAURI-MIGRATION.md)
- [ ] Mobile
