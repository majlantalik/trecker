# Trecker: Tauri 2 + SQLite migration plan

Branch: `worktree-tauri-migration`
Target: local-first desktop app for Windows, macOS and Linux. Mobile deferred.

## Principles

1. **The Vue app is not rewritten.** All 42 frontend files stay. Only `src/api/*.ts` changes.
2. **The catalog / tracking table split survives.** It is the thing that makes sync tractable later. Do not collapse it just because there is one local user.
3. **MusicBrainz is the only resolver.** The streaming services are link-only. See below.
4. **`backend/` is frozen, not deleted.** It becomes the Phase 6 sync server.
5. Each phase ends with a working app. No phase leaves `main` of this branch broken.

## Streaming services: link-only

None of the three can be queried from a shipped binary, because none of them can be
queried without a credential that would have to ship with it.

Spotify Development Mode caps an app at 5 allowlisted users and requires the app owner to
hold a Premium subscription; Extended Quota requires 250,000 monthly active users. Tidal
uses the same client-credentials flow and so has the same problem. A YouTube API key is
extractable from any binary it ships in, and the quota it spends is the developer's.

Bring-your-own-credentials was considered and rejected: asking every user to hold a
Premium subscription and register a developer app before they can add an album is not
reasonable.

What Trecker does instead:
- MusicBrainz for identity and genres, the Cover Art Archive for artwork. Neither needs
  an account, a key or an approval.
- A pasted streaming URL is saved as a link. Sharing and tracking parameters are stripped.
- Bandcamp and Apple Music URLs carry the artist and album in the path, so those are read
  and searched. Spotify and Tidal ids are opaque and yield nothing, correctly.

---

## Target layout

```
trecker/
  frontend/                 # unchanged Vue 3 app
  backend/                  # frozen; Phase 6 sync server
  src-tauri/
    Cargo.toml
    tauri.conf.json
    build.rs
    migrations/             # sqlx migrations, replaces Liquibase
    src/
      main.rs
      lib.rs
      error.rs              # AppError -> serializable to the frontend
      db.rs                 # pool setup, PRAGMAs, migration runner
      domain.rs             # wire types, mirroring frontend/src/types/index.ts
      repo/                 # sqlx queries
        filter.rs           # port of UserReleaseSpecification
        integration.rs      # tests against a real database
      commands.rs           # #[tauri::command] surface
      resolve/              # MusicBrainz + Cover Art Archive
```

### Crates

| Crate | Purpose |
|---|---|
| `tauri` 2 | shell, commands, bundling |
| `sqlx` (sqlite, runtime-tokio, migrate) | DB + migrations |
| `reqwest` (rustls-tls, json, gzip) | resolver HTTP |
| `tokio` | async runtime, timeouts, the rate-limit gate |
| `serde` / `serde_json` | command payloads, provider responses |
| `thiserror` | error enum |
| `tauri-plugin-updater` | auto-update (Phase 5) |

`uuid` and `chrono` were on this list and are not used. Ids are text and timestamps are
RFC3339 text, so generating both is a few lines each. `keyring` and
`tauri-plugin-deep-link` were here for OAuth, which is not happening.

Deliberately **not** using `tauri-plugin-sql`. It exposes SQL to the JS side; we want
typed queries in Rust and a narrow command surface.

---

## Build prerequisites (Linux dev machine)

Rust 1.95 and Node 20 are already present. The webview headers are not:

```
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
  build-essential curl wget file libssl-dev pkg-config
```

Then, from the repo root:

```
npm install          # installs the Tauri CLI at the workspace root
npm run dev          # starts Vite and the Tauri window
npm run build        # production bundle
```

## Phase 0 — Spike (target: one weekend)

Goal: kill the biggest unknown before committing. The unknown is WebKitGTK on Linux,
which is the weakest of the three webviews.

Good news from a pre-scan of the frontend: no `:has()`, no `@container`, no `color-mix`.
The only risky properties are `backdrop-filter` in `AppLayout.vue:41` (already
`-webkit-` prefixed) and `position: sticky` in two places. Risk is lower than feared.

- [ ] `cargo install create-tauri-app`, scaffold `src-tauri/` against existing `frontend/`
- [ ] `tauri.conf.json`: `devUrl` = `http://localhost:5173`, `frontendDist` = `../frontend/dist`,
      `beforeDevCommand` = `npm --prefix ../frontend run dev`
- [ ] One throwaway command returning a hardcoded `Release`, rendered by `ReleaseCard.vue`
- [ ] Build and run on Linux. Verify in order:
      PrimeVue Aura renders; the teal accent palette and gradients are correct;
      `backdrop-filter` on the sticky header; Chart.js canvas at correct DPI;
      custom scrollbar styling; fonts load
- [ ] Screenshot side by side against the browser build

**Exit criterion:** the Linux build is visually acceptable. If it is not, stop and
reconsider Compose Multiplatform before writing any Rust.

### Result: PASSED

Run on WebKitGTK 2.52.6 (WebKit 605.1.15), dev build and production build.

| Check | Chromium baseline | WebKitGTK |
|---|---|---|
| `invoke()` round trip | n/a | 3 releases |
| Syne (display font) | loaded | loaded |
| Outfit (body font) | loaded | loaded |
| `backdrop-filter` | unprefixed | unprefixed |
| `position: sticky` | yes | yes |
| CSS custom properties | `#00e5b0` | `#00e5b0` |
| devicePixelRatio | 1 | 1 |

No fallback fonts, no unsupported properties, no rendering divergence. The Chart.js
canvas, both album-art paths and the half-star widget all appear in the accessibility
tree. **Proceed with Tauri.**

Production binary, frontend embedded: **4.2 MB**.

### Four things learned that change how we build

**`createWebHistory` works.** A deep route resolves against the embedded asset protocol
in a production build, so Tauri does SPA fallback. No need to switch to hash history,
which is the usual advice for Tauri apps. The router stays as it is.

**Never build with plain `cargo build --release`.** That bypasses the Tauri CLI and leaves
`devUrl` embedded, so the binary tries to reach localhost:5173 and shows a connection
error. Always go through `npm run build` at the workspace root.

**The Tauri CLI must run from the directory above `src-tauri`.** It searches subfolders
only, so running it from `frontend/` fails. A root `package.json` now owns the Tauri
scripts, and `beforeDevCommand` / `beforeBuildCommand` resolve from the workspace root,
not from `src-tauri`.

**On Wayland, screenshots need a desktop portal grant.** Automated visual checks should
read the accessibility tree instead, which is why the spike renders its checks as `<p>`
elements. `GDK_BACKEND=x11` forces XWayland if an X11 tool is needed. Note that
`document.title` does not propagate to the native window title on WebKitGTK.

The spike view and its route were removed in Phase 1. To re-run those checks, recover
`frontend/src/views/SpikeView.vue` from commit `8055443`.

### Found while scaffolding

**Fonts are loaded from Google Fonts at runtime.** `frontend/index.html` pulls Syne and
Outfit over the network. In a local-first desktop app that is wrong three times over: the
app falls back to system fonts when offline, Tauri's CSP has to be opened up to allow it,
and a local-first app should not phone a third party on every launch. Fix by vendoring
the two families via `@fontsource/syne` and `@fontsource/outfit` and importing them in
`main.ts`. Do this in Phase 0 so the spike measures the real thing.
- [ ] Vendor Syne and Outfit, drop the `<link>` tags from `index.html`

**`index.html` references `/favicon.svg`, which does not exist.** There is no `public/`
directory. Harmless in the browser, but it should be replaced by the generated icon.

**CSP is currently `null`** in `tauri.conf.json`, which is permissive. Acceptable for the
spike. Tighten it in Phase 5, once the resolver hosts are known.

Also do now, independent of the spike:
- [ ] Check the Spotify developer dashboard for grandfathered user count and client ID count

---

## Phase 1 — Swap the seam

The entire client-to-server contact surface is ~170 real lines across `frontend/src/api/`.
Keep every exported function signature identical. Replace only the bodies.

```ts
// before
async getRandom(): Promise<Release> {
  const { data } = await api.get<Release>('/releases/random')
  return data
}
// after
async getRandom(): Promise<Release> {
  return invoke<Release>('releases_random')
}
```

Command names map 1:1 onto the existing API functions:

| Frontend call | Tauri command |
|---|---|
| `releasesApi.resolve` | `releases_resolve` |
| `releasesApi.create` | `releases_create` |
| `releasesApi.getAll` | `releases_list` |
| `releasesApi.getRandom` | `releases_random` |
| `releasesApi.getById` | `releases_get` |
| `releasesApi.update` | `releases_update` |
| `releasesApi.delete` | `releases_delete` |
| `releasesApi.searchCatalog` | `releases_search_catalog` |
| `genresApi.getAll` | `genres_list` |
| `statsApi.getActivity` | `stats_activity` |
| `statsApi.getByGenre` | `stats_by_genre` |
| `statsApi.getByCountry` | `stats_by_country` |
| `statsApi.getTopRated` | `stats_top_rated` |
| `statsApi.getYearEnd` | `stats_year_end` |

Tasks:
- [x] Add `@tauri-apps/api`; delete `axios` and `axios-mock-adapter`
- [x] Rewrite `releases.ts`, `genres.ts`, `stats.ts` bodies
- [x] Delete `api/axios.ts`, `api/axios.test.ts`, `api/auth.ts`, `api/profile.ts`
- [x] Delete `stores/auth.ts`, `stores/auth.test.ts`
- [x] Delete `views/LoginView.vue`, `views/RegisterView.vue`
- [x] Replace `views/ProfileView.vue` with `views/SettingsView.vue`, reusing the section
      styling, at `/settings`
- [x] Strip auth guards and the login/register routes from `router/index.ts`, and remove
      the `/spike` route and `SpikeView.vue`
- [x] Remove the `fetchMe()` call from `main.ts`
- [x] Keep `stores/releases.test.ts` and the component tests; add `api/commands.test.ts`

**Exit criterion: MET.** The app runs with no network layer. Verified against a production
build by driving the real window: the queue lists fixtures, Stats renders all three charts
with correct aggregation and ordering, Settings renders, and deleting a release removes it
and drops both the heading and the sidebar badge from 4 to 3.

30 frontend tests pass, typecheck is clean, and the Rust side compiles without warnings.

### What the seam swap actually cost

The prediction held. `stores/releases.ts`, `stores/genres.ts` and `stores/stats.ts` needed
**zero changes** — they only ever touched the API layer. None of the seven views changed
either, beyond the two that were deleted and the one that was replaced.

What did change, beyond the API bodies:

- `App.vue` lost its `route.meta.public` branch, so there is one layout, not two.
- `AppSidebar.vue` lost the avatar, the user name and the sign-out button. The footer slot
  now links to Settings, reusing the same styling.
- `main.ts` mounts synchronously. There is no session to resolve first.

### Deviations from the plan as written

**The store is not a stub.** The plan said fixtures; `store.rs` is a real in-memory store
with working create, update, delete, filter, sort and pagination, seeded with ten releases.
Stubs would not have caught anything. This did: it proved the filter contract and the
full write round trip. Phase 3 replaces this module with sqlx and the command signatures
do not move.

**`ProfileView.vue` was replaced rather than gutted.** Reusing its section CSS was worth
it, but nothing else in the file survived the removal of accounts, so editing it in place
would have been a rewrite pretending to be an edit. The route moved `/profile` →
`/settings` to match.

**The sort whitelist landed early.** `SORTABLE` in `store.rs` already rejects unknown sort
fields. That is the Phase 3 injection fix, put in now so the contract is correct before
the SQL exists rather than after.

**`api/commands.test.ts` replaces `api/axios.test.ts`.** The deleted test covered the 401
refresh interceptor, which no longer exists. The new one asserts every command name and
argument key, so a rename on either side of the boundary fails in CI instead of at runtime.

---

## Phase 2 — Schema

Port the 10 Liquibase migrations in `backend/src/main/resources/db/changelog/changes/`
to `src-tauri/migrations/`. Do not port them one for one. Collapse 001 through 010 into a
single `0001_initial.sql` that produces the *current* schema. History has no value here.

Postgres to SQLite conversions:

| Postgres | SQLite |
|---|---|
| `UUID` | `TEXT` (36 char), not BLOB, so it stays greppable |
| `TIMESTAMPTZ` | `TEXT` RFC3339 via `chrono::DateTime<Utc>` |
| `VARCHAR(n)` | `TEXT` (length is not enforced; keep CHECK where it matters) |
| status enum | `TEXT NOT NULL CHECK (status IN ('QUEUED','LISTENED'))` |
| `BOOLEAN` | `INTEGER NOT NULL DEFAULT 0` |
| `DOUBLE PRECISION` rating | `REAL` |

Tables to drop: `users`, `refresh_tokens`.

Tables to keep exactly as they are: `releases`, `user_releases`, `genres`,
`release_genres`, `release_streaming_links`. The element collection is already a child
table, so it needs no restructuring.

**Keep `user_releases.user_id`.** Default it to a constant local UUID written at first
launch. This costs nothing now and is what makes Phase 6 possible.

New:
- [x] FTS5 virtual table over `releases(artist, title)` with insert/update/delete triggers
- [x] Connection PRAGMAs on every connection: `journal_mode=WAL`, `foreign_keys=ON`,
      `busy_timeout=5000`, `synchronous=NORMAL`
- [x] DB file at Tauri's `app_data_dir()`, created on first run, migrations applied at startup

Data migration from the existing Postgres instance:
- [x] **Not needed.** The Postgres database only ever held throwaway test rows on one
      machine and was never deployed. The local database starts empty. An import script
      was written and then deleted in commit `b0ad9a5`'s successor; recover it from there
      if a populated database ever turns up.

**Exit criterion: MET.** The database is created, migrated and opened on first launch.

### Verified

Read back from inside the running app, on the pool's own connection, which is what
matters because SQLite pragmas are per-connection and a separate reader would have told
us nothing:

| | |
|---|---|
| Location | `~/.local/share/cz.mtulek.trecker/trecker.db` |
| Schema version | 1 |
| Journal mode | WAL |
| Foreign keys | On |
| Full-text search | FTS5 present |

Exercised directly against the file: all three FTS triggers (insert indexes, update
re-indexes, delete removes), prefix matching, `ON DELETE CASCADE` from a catalog row to
its tracking row, and the status CHECK rejecting a value outside the enum.

### Known limitation: ligatures do not fold

`remove_diacritics 2` folds accents, so searching `ros` finds *Sigur Rós*. It does not
decompose ligatures, so `agaetis` does not find *Ágætis byrjun*, while `agætis` does.
That is unicode61 behaving as documented, not a misconfiguration. It affects Nordic and
German titles (æ, ø, ß). Fixing it means either a custom tokenizer registered from Rust
or an ASCII-folded shadow column maintained at write time. Deferred, not forgotten.

### Deviations from the plan as written

**`updated_at` is on `user_releases` from the start**, per the Phase 6 note. Adding a
`NOT NULL` column later to a populated table is meaningfully worse than carrying it now.

**A `settings_db_info` command was added**, outside the fourteen. It is diagnostics, not
part of the release API, and it is how the table above was read. It also makes the
Settings view honest about where the data actually lives.

**The import script was written, then dropped.** It went through Postgres directly rather
than the REST API, and handled the multi-account case. All of that turned out to be
solving a problem that does not exist, so it is gone. Recorded here only so the deletion
does not look like an oversight.

---

## Phase 3 — Rust core

Order matters: repos before commands, commands before deleting the stubs.

- [x] `domain.rs` structs with `serde::Serialize`, `#[serde(rename_all = "camelCase")]`
- [x] `repo/` sqlx queries
- [x] `repo/filter.rs`: port `UserReleaseSpecification` (74 lines) to a `QueryBuilder<Sqlite>`

Two things to get right in the filter port:

**Search ports verbatim.** The Java already does `cb.lower(field) LIKE lower(pattern)`,
so `LOWER(artist) LIKE ?` works in SQLite unchanged. No ILIKE problem. Same for the
country and genre comparisons.

**Sorting is now an injection risk.** `ReleaseFilterParams.sort` is a client-supplied
string. JPA made that safe. Raw SQL does not. Whitelist the sortable columns to an enum
and reject anything else.

- [x] Preserve the `PageResponse<T>` shape, which means a `COUNT(*)` query alongside the page
- [x] Port `ReleaseService.create()` dedup
- [x] Port `StatsService` (87 lines) to five SQL aggregate queries
- [x] `error.rs`: one `AppError` enum, serialized to the frontend as `{ code, message }`
- [x] Replace the Phase 1 store; `store.rs` is deleted

**Exit criterion: MET.** Every command reads and writes SQLite. The in-memory store is gone.

### Verified

**28 Rust tests**, run against a real SQLite file through the real migration, covering
dedup on both external ids, idempotent re-tracking, genre find-or-create and
case-insensitive dedup, cascade behaviour on delete, every filter predicate, the sort
whitelist, pagination arithmetic across three pages, FTS behaviour including operator
injection, the year-end boundary, and reopening the database.

**Through the running app**, on a production build: the empty states render on a fresh
database; all five stats commands return and draw; after inserting three releases the
queue reads them back with their streaming links attached; Top Rated orders 5.0 above
4.5; and deleting through the UI takes tracking rows from 3 to 2 while catalog rows stay
at 3, leaving exactly the deleted release orphaned in the catalog.

30 frontend tests pass and the typecheck is clean.

### Deviations from the plan as written

**`sqlx::query!` macros were not used.** They need a live database at compile time
(`DATABASE_URL` or a checked-in `.sqlx` cache) and they cannot check a query that is
assembled at runtime, which every filtered query here is. Runtime-checked `query()` with
`try_get` is used throughout instead. The integration tests cover what the macro would
have: a wrong column name fails a test rather than a build.

**The genre filter uses `EXISTS`, not a join.** The Java joined to `genres` and then
needed `query.distinct(true)` to undo the row multiplication it had just caused. `EXISTS`
asks the same question without ever producing duplicates, so the long-standing gotcha
about remembering `distinct` does not carry over. There is a test asserting a release
with three genres appears once.

**Text sorting is now case-insensitive.** `COLLATE NOCASE` on artist and title. SQLite's
default collation orders every capital before every lowercase letter, so a library sorted
by artist would have put "zZ Top" before "beach House". This is a deliberate change from
the Postgres behaviour, not an accident of the port.

**Sorting gained a tiebreaker.** `ORDER BY <column>, ur.id`. Without it, rows equal on the
sort column can change places between page fetches, so one release shows up on two pages
and another never appears. The Java had the same latent bug.

**FTS5 is used for catalog search only.** The library filter box keeps substring `LIKE`,
matching the Java exactly, because FTS matches whole tokens and prefixes: it would stop
finding "phere" inside "Stratosphere". Prefix semantics are right for autocomplete and
wrong for a filter that narrows as you type. Both behaviours are tested.

**User input is never passed to FTS5 raw.** FTS5 has its own query language, so `"`, `*`,
`:`, `-`, `AND`, `OR` and `NOT` are either syntax errors or silently mean something else.
Every token is quoted and given a trailing `*`. Tested with deliberately hostile input.

**`chrono` and `uuid` were not added.** The schema stores timestamps as RFC3339 text and
ids as text; generating both is a few lines each, already written and tested for the
Phase 1 store. Two fewer dependencies in a binary that has to be signed and shipped.

### Not verified

**Quick Add could not be driven from the UI.** Keyboard input needs the window focused,
which the OS-level tooling could not achieve under Wayland, and the autocomplete widget
rejects direct value writes. So the create path is proven by integration tests and by
inserting rows directly, but not by typing into the app. Worth a manual check: type an
artist and title, click Add, and confirm the release lands in the queue.

---

## Phase 4 — Resolvers

Reordered so the free, unauthenticated provider lands first.

- [x] `resolve/musicbrainz.rs` (from 279 Java lines). `reqwest` with the required
      `User-Agent` and a real one-per-second gate.
- [x] `resolve/coverart.rs` for Cover Art Archive, including retry on transient failures
- [x] `resolve/mod.rs` (from `MetadataResolverService`, 210 lines)
- [x] Enrichment degrades instead of failing the resolve

**Streaming providers are not implemented and will not be.** Decision recorded below.

**Exit criterion: MET.** Verified against the live services:

```
artist:         Slint
title:          Spiderland
release_year:   1991
album_art_url:  coverartarchive.org/release/266e8eb6-.../19590732868-500.jpg
country:        US
genres:         alternative rock, rock
musicbrainz_id: 266e8eb6-244f-450d-b419-7e3cdf815d4c
```

44 Rust tests pass, plus 3 network tests excluded from normal runs. Run those with
`cargo test --lib -- --ignored --test-threads=1`; the single thread is required because
the rate limiter is per-Resolver and three parallel tests get a 503.

### Why the streaming providers are gone

All three fail the same way: the credential cannot travel in a shipped binary.

| | Needs | Why it cannot ship |
|---|---|---|
| Spotify | client secret, or per-user OAuth | Dev Mode caps at 5 allowlisted users and the owner must hold Premium |
| Tidal | client secret | same client-credentials flow, same problem |
| YouTube | API key | extractable from the binary, and the quota is the developer's to lose |

Bring-your-own-credentials was considered and rejected. It would require every user to
hold a Spotify Premium subscription, create a developer app and accept the developer
terms, to add an album. That is not a reasonable thing to ask.

So a pasted streaming URL is **kept as a link, not resolved**. The link is what the user
wanted to save anyway. Sharing parameters (`si=`, `utm_*`) are stripped, because they
identify whoever shared it and do not belong in a library.

Bandcamp and Apple Music put the artist and album in the URL path, so those *are* read and
searched. Spotify and Tidal use opaque ids, which yield nothing, and that is the right
outcome: an empty form with the link saved beats a confident match on a random string.

### Three things the live run caught that tests could not

**The year was wrong.** A MusicBrainz release search returns one *pressing*. Searching
"Slint - Spiderland" landed on a 2014 reissue and reported 2014. The release group carries
`first-release-date`, which is the album's own date, and it now wins. The Java had this
bug too.

**The country was `XW`.** That is MusicBrainz for "worldwide", a correct answer to where
a record was released and a useless one for a By Country chart. `XW`, `XE` and friends are
now treated as absent so the artist-country lookup fills in, which turned `XW` into `US`.

**503 was not being retried.** MusicBrainz answers "currently busy" under load, and both
the Java and the first cut of this port retried only transport failures, so a busy server
looked exactly like a release that does not exist. 429, 500, 502, 503 and 504 now back off
and retry, honouring `Retry-After`; 404 still does not, because retrying it burns the
one-per-second budget three times to learn the same thing.

### Other deviations from the Java

**The rate limiter is a gate, not a sleep.** The Java slept 1100ms before every call. That
delayed the first request of the day for no reason, and did not actually stop two
concurrent calls firing together. A mutex over the last-request time serialises requests
and skips the wait when enough time has already passed.

**Lucene input is escaped.** The Java interpolated the artist and title straight into the
query, so an album with a quote, a colon or a slash in its name produced a malformed
search and silently no results. "AC/DC" was enough to trigger it.

**Genres come from MusicBrainz.** They used to come from Spotify, so they would otherwise
have been lost along with the By Genre chart. Taken from the release, falling back to the
release group, ranked by vote count and capped at five.

**No provider trait.** With one provider there is nothing to abstract over. The enrichment
steps already degrade independently, which is what the trait was for.

---

## Phase 5 — Ship

- [ ] Bundle targets: `.msi` and `.exe` (Windows), `.dmg` (macOS), `.deb` and `.AppImage` (Linux)
- [ ] `tauri-plugin-updater` with a static JSON manifest on GitHub Releases
- [ ] macOS signing and notarization (Apple Developer Program, annual fee)
- [ ] Windows signing (certificate with hardware token; this complicates CI)
- [ ] GitHub Actions matrix build

Budget real time for signing. It is the most commonly underestimated phase.

---

## Phase 6 — Sync (only if the local app earns it)

Unfreeze `backend/`. Strip every controller except auth, and add a sync endpoint.
The JWT work, the Postgres schema and the catalog model all survive intact.

Conflict model, which the existing data model already gives us for free:
- Catalog rows are content-addressed by `spotify_id` / `musicbrainz_id`. They dedupe by
  definition and cannot conflict.
- Only `user_releases` needs resolution, and it is one writer per device.
  Last-write-wins per field on an `updated_at` column covers it.
- [ ] Add `updated_at` to `user_releases` in Phase 2 so this is available later at no cost

---

## Stale docs to fix on the way through

- `CLAUDE.md` says "no automated tests yet". There are vitest suites in `api/`, `stores/`
  and `components/release/`, plus Playwright as a dependency.
- `CLAUDE.md` says the highest migration is `009`. It is `010-rating-decimal.sql`.

## Decisions taken

- **No Spotify, Tidal or YouTube lookups.** Their credentials cannot ship in a binary and
  bring-your-own would require every user to hold a paid subscription and register a
  developer app. Their links are saved; nothing is queried through them.

- **`ProfileView` is rebuilt as a local Settings view.** It keeps its route and its panel
  styling, and loses the account fields. It gains, in Phase 4, the per-provider metadata
  toggles and the OAuth connect buttons. `displayName`, email and password change all go.
  This means `ProfileView.vue` is *not* deleted in Phase 1 as originally written. It is
  gutted and refilled. The Phase 1 checklist has been amended accordingly.

- **The local database starts empty. No Postgres import.** The Postgres instance only ever
  held throwaway test rows, on one machine, and was never deployed. Nothing is worth
  carrying across.

- **The app starts genuinely empty. No seed data.** All four stats components, the queue
  and the library already had empty states, so a fresh database reads as deliberate rather
  than broken. Seed rows in a personal library are clutter you have to delete.

## Open decisions

1. Ship Spotify at all in v1, given it is on a deprecation path?
