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

- **Artist and album** (`slint spiderland`), or just an album title — searched on
  MusicBrainz. A dash between artist and album (`Slint - Spiderland`) makes the search
  stricter, but is not needed.
- **A Bandcamp or Apple Music link** — the artist and album are read out of the URL and
  searched, and the link is saved.
- **A Spotify, Tidal or YouTube link** — the link is saved, but nothing is looked up
  through it. See below.

Press <kbd>Enter</kbd> or **Add**. If the search finds one album, the form opens with it
filled in: artist, title, the album's original year, country, genres and cover. If it finds
several, up to ten are listed with their covers, type and year. Choose one with the arrow
keys and <kbd>Enter</kbd>, or click it, and the form opens filled in with that album.
**None of these** opens the form without a match. So does a search that finds nothing.

Adjust anything in the form, then **Add to Queue**.

Existing releases in your library appear as suggestions while you type, so adding
something you already have does not create a duplicate.

### Streaming links

Spotify, Tidal and YouTube links are saved but nothing is looked up through them. Sharing
parameters like `si=` are stripped first, since they identify whoever sent you the link.
The reasoning is in [ADR 0003](docs/adr/0003-streaming-services-are-link-only.md).

### Queue

Everything you have not listened to yet, ordered by the date you added it. **Newest first**
next to the title reverses that to oldest first, so you can work through the queue in the
order albums arrived. Trecker remembers the choice, which is also in **Settings**.

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

Genres are chips, here and everywhere else. Type one and press <kbd>Enter</kbd> or a comma,
paste a comma-separated list, or pick from the genres already in your library as they are
suggested. A genre you already have keeps the spelling your library uses.

### Keyboard shortcuts

Press <kbd>?</kbd> for the list, or click the <kbd>?</kbd> in the sidebar.
<kbd>Shift</kbd> <kbd>Shift</kbd> opens quick add from anywhere in the app, in the style of
a JetBrains IDE; <kbd>Ctrl</kbd>+<kbd>K</kbd> does the same for anyone who reaches for that
first.

Navigation follows Gmail: <kbd>G</kbd> then the first letter of where you are going, so
<kbd>G</kbd> <kbd>L</kbd> for Library. A hint appears while the sequence is half-entered.

### Refreshing metadata

Metadata is fetched once when you add a release. If it came back wrong or incomplete, the
refresh button on a release's page fetches it again. It replaces artist, title, year,
country and artwork, adds any genres you do not have, and never touches your rating, notes,
dates or links.

An album typed in by hand or imported has no MusicBrainz id, so refresh cannot know which
album it is. It searches instead and lets you choose from the results, and remembers the
choice. **Info → Match to MusicBrainz** does this for every such album at once: exact
matches on artist, title and year are linked on their own, and the rest wait for you to
pick, one after another.

### Library

Everything you have listened to, as a table or a grid. Filter by free-text search, status,
genre, country, year, rating range and the did-not-finish flag. Countries show as flags
and names, and the country filter offers only the ones you actually have.

### Artists

A backlog of artists to check out, separate from the queue: someone a friend mentioned, a
band from a festival line-up, with no particular album in mind yet.

- **Find** searches MusicBrainz. Choose the right artist from the list, which tells
  same-named artists apart, and they join your list with their country, active years,
  genres and links. Their picture is the cover of their first album, when it has one.
- An artist's page has a **note** for who recommended them or where to start, links that
  open in your browser, and their **discography** of albums and EPs, oldest first. Live
  albums and compilations are hidden until you ask for them.
- Each album shows whether it is already **In queue** or **Listened**. **Queue** opens the
  add form filled in; **Log** opens the log dialog, and closing it without logging leaves
  nothing behind.
- When you are done, choose **Into it**, **Not for me** or **Just checked**. Checked artists
  move to a collapsed section at the bottom of the list, and **Back to the list** undoes it.

The discography is fetched from MusicBrainz each time you open the page, so it needs a
connection; everything else works offline. Why artists work this way:
[ADR 0007](docs/adr/0007-artists-to-check.md).

### Stats

Listening activity by month, breakdowns by genre and country, and a year-end list of your
twenty highest-rated albums for any year, by the year you heard them or the year they came
out.

### Quick add from anywhere

To open quick add while you are in another app, bind a key to Trecker's quick add command
in your desktop's keyboard settings, as a custom shortcut that runs a command. **Settings**
shows the exact command for your copy of Trecker, with a button to copy it. Usually it is:

```bash
trecker --quick-add
```

If Trecker is running, its window comes forward with quick add open. If it is not, the
command starts it first.

Trecker cannot claim a key for the whole desktop by itself: on Wayland, which COSMIC and
most current Linux desktops use, only the desktop can.

### Settings

The cog in the sidebar opens **Settings**, which holds preferences for this computer only.
They are not part of your library and are not exported.

- **When you close the window**: quit, or keep running with an icon in the tray. The tray
  menu has Open, Quick add and Quit. Keeping Trecker running makes the quick add shortcut
  appear instantly instead of starting the app first. Starting Trecker again always brings
  its window back.
- **Queue**: newest or oldest added first. The button at the top of the queue sets the same
  choice.

### Where your data lives

One SQLite file:

| Platform | Path |
|---|---|
| Linux | `~/.local/share/cz.mtulek.trecker/trecker.db` |
| macOS | `~/Library/Application Support/cz.mtulek.trecker/trecker.db` |
| Windows | `%APPDATA%\cz.mtulek.trecker\trecker.db` |

The Info view shows the exact path. Back it up by copying the file while the app is closed.

Album covers are kept separately, in your system's cache folder, such as
`~/.cache/cz.mtulek.trecker/covers` on Linux. Each one downloads the first time an album is
shown, so the library looks the same offline after that. **Info → Cache** shows how much
space they take and clears them, and they download again as albums are shown. It can also
clear the temporary files kept by the browser engine the app runs in.

### Export and import

**Info → Backup** writes your whole library to a file and reads one back.

- **JSON** is the complete record and the one to keep for a backup or a move to another
  machine.
- **CSV** holds the same releases in a shape a spreadsheet can read. It round-trips too.

Artists to check are not exported yet; a backup covers releases only.

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
| Shell | Tauri 2 (system webview, ~9 MB binary) |
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
npm test           # 248 frontend tests
npm run test:rust  # 177 Rust tests
npm run test:net   # 14 tests against the live metadata services
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
│       ├── api/              releases.ts, artists.ts, genres.ts, stats.ts, info.ts,
│       │                     library.ts, dialog.ts, cache.ts, settings.ts, app.ts
│       │                     the only place that knows about Tauri
│       ├── components/
│       │   ├── artist/       ArtistCard, ArtistAvatar, ArtistPicker
│       │   ├── layout/       AppLayout, AppSidebar
│       │   ├── library/      LibraryFilters, ViewToggle
│       │   ├── queue/        QueueActions
│       │   ├── release/      QuickAddBar, ReleaseCard, ReleaseForm,
│       │   │                 QuickLogModal, GenreTagInput
│       │   └── stats/        ActivityChart, BreakdownChart, YearEndList
│       ├── composables/      keyboard shortcuts, useLibraryTransfer, useCaches
│       ├── stores/           releases.ts, artists.ts, genres.ts, stats.ts (Pinia)
│       ├── types/index.ts    single source of truth for TS interfaces
│       └── views/            QueueView, LibraryView, ArtistsView, ArtistView,
│                             StatsView, EntryView, InfoView, SettingsView
├── src-tauri/                Rust core
│   ├── migrations/           sqlx migrations
│   ├── tauri.conf.json
│   └── src/
│       ├── commands.rs       the #[tauri::command] surface
│       ├── covers.rs         cover cache, served through the cover: protocol
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

Thirty-six Tauri commands, mapping 1:1 onto `frontend/src/api/*.ts`:

| Command | Purpose |
|---|---|
| `releases_resolve` | Resolve a pasted link to one album's metadata |
| `releases_search` | Up to ten albums matching typed text, most likely first |
| `releases_lookup` | Full metadata for the album chosen from a search |
| `releases_create` | Add a release to the queue |
| `releases_list` | List with filtering, sorting, pagination |
| `releases_random` | Pick a random queued release |
| `releases_get` | Fetch one release |
| `releases_update` | Partial update; `status=LISTENED` stamps `dateListened` |
| `releases_delete` | Stop tracking; the catalog row stays |
| `releases_refresh_metadata` | Re-fetch catalog fields for a release linked to MusicBrainz |
| `releases_link` | Link a release to the album chosen from a search, and fetch its metadata |
| `releases_unlinked` | Releases with no MusicBrainz id, for matching them in bulk |
| `releases_search_catalog` | Full-text autocomplete over the catalog |
| `artists_search` | Up to ten artists matching typed text |
| `artists_add` | Look an artist up and put them on your list |
| `artists_list` | Your artists, those to check first |
| `artists_get` | Fetch one artist |
| `artists_update` | Change the note, or mark checked with a verdict |
| `artists_delete` | Take an artist off your list; the catalog row stays |
| `artists_discography` | Their albums and EPs, marked with your library's copies |
| `genres_list` | Every known genre |
| `countries_list` | Countries present in the library, for the filter |
| `stats_activity` | Listening count by year and month |
| `stats_by_genre` | Breakdown by genre |
| `stats_by_country` | Breakdown by country |
| `stats_year_end` | Top twenty for a year, by listen date or release year |
| `info_db` | Database path, size, schema version, pragmas |
| `library_export` | Write the whole library to a JSON or CSV file |
| `library_import` | Read one back, skipping or overwriting what matches |
| `cache_covers_info` | Where cached covers live, how many, and their size |
| `cache_covers_clear` | Delete cached covers; they download again when shown |
| `cache_webview_clear` | Clear the webview's own cache and browser data |
| `settings_get` | This computer's preferences |
| `settings_update` | Apply and save preferences, such as keeping running in the tray |
| `app_take_launch_action` | Whether this launch was started with `--quick-add`, once |
| `app_quick_add_command` | The command to bind to a desktop shortcut |

Covers are not commands. The webview requests them from the `cover:` protocol, which
`covers.rs` serves from disk.

`frontend/src/api/commands.test.ts` asserts every name and argument key, so a rename on
either side of the boundary fails in CI rather than at runtime.

### Data model

A shared catalog plus per-user tracking. Why the split is kept with one user:
[ADR 0002](docs/adr/0002-sqlite-with-catalog-and-tracking-split.md). Why the key is a
release group: [ADR 0004](docs/adr/0004-release-group-is-album-identity.md).

- `releases` — deduplicated catalog, keyed by the album's MusicBrainz release group.
- `user_releases` — status, rating, notes, dates. Deleting one leaves the catalog row.
- `artists` and `user_artists` — the same split for artists to check, keyed by the
  MusicBrainz artist id. Why: [ADR 0007](docs/adr/0007-artists-to-check.md).

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
- [x] Quick add from anywhere, through a desktop shortcut running `trecker --quick-add`
- [x] Keep running in the tray on close, as a setting
- [x] Artists to check, with their discography ([ADR 0007](docs/adr/0007-artists-to-check.md))
- [ ] Artists in the export
- [ ] Add an artist to check from an album's page
- [ ] Mobile
