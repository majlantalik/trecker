# Trecker: Tauri 2 + SQLite migration plan

Branch: `worktree-tauri-migration`
Target: local-first desktop app for Windows, macOS and Linux. Mobile deferred.

## Principles

1. **The Vue app is not rewritten.** All 42 frontend files stay. Only `src/api/*.ts` changes.
2. **The catalog / tracking table split survives.** It is the thing that makes sync tractable later. Do not collapse it just because there is one local user.
3. **MusicBrainz is the primary resolver.** Spotify becomes optional enrichment. See "Spotify position" below.
4. **`backend/` is frozen, not deleted.** It becomes the Phase 6 sync server.
5. Each phase ends with a working app. No phase leaves `main` of this branch broken.

## Spotify position

Spotify cannot be the backbone. Development Mode is capped at 5 allowlisted users,
requires the app owner to hold Premium, and Spotify has stated it is moving away from
the Client Credentials flow for metadata endpoints, which is our only use of it.
Extended Quota requires 250k monthly active users.

Consequences baked into this plan:
- MusicBrainz + Cover Art Archive are the default path and need no credentials at all.
- Spotify, Tidal and YouTube ship disabled, enabled per user in settings.
- No client secret ever ships in the binary. Authorization Code + PKCE only.
- Resolver architecture must degrade cleanly when a provider is off or fails.

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
      db/mod.rs             # pool setup, PRAGMAs, migration runner
      domain/               # Release, UserRelease, Genre structs
      repo/                 # sqlx queries
      query/filter.rs       # port of UserReleaseSpecification
      commands/             # #[tauri::command] surface
      resolve/              # metadata providers
```

### Crates

| Crate | Purpose |
|---|---|
| `tauri` 2 | shell, commands, bundling |
| `sqlx` (sqlite, runtime-tokio, tls-rustls, uuid, chrono, migrate) | DB + migrations |
| `reqwest` (rustls-tls, json) | resolver HTTP |
| `tokio` | async runtime, `try_join!`, `timeout` |
| `serde` / `serde_json` | command payloads |
| `uuid` (v4, serde) | IDs, stored as TEXT |
| `chrono` (serde) | timestamps, stored as RFC3339 TEXT |
| `thiserror` | error enum |
| `keyring` | OAuth tokens in the OS keychain (Phase 4) |
| `tauri-plugin-deep-link` | OAuth callback (Phase 4) |
| `tauri-plugin-updater` | auto-update (Phase 5) |

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
- [ ] FTS5 virtual table over `releases(artist, title)` with insert/update/delete triggers
- [ ] Connection PRAGMAs on every connection: `journal_mode=WAL`, `foreign_keys=ON`,
      `busy_timeout=5000`, `synchronous=NORMAL`
- [ ] DB file at Tauri's `app_data_dir()`, created on first run, migrations applied at startup

Data migration from the existing Postgres instance:
- [ ] `scripts/export-to-sqlite.ts` that calls the running backend's
      `GET /api/releases?size=10000` with a valid session and emits SQLite INSERTs.
      Going through the API reuses the `ReleaseResponse` shape and avoids schema drift.
      One-time, throwaway, does not need to be pretty.

---

## Phase 3 — Rust core

Order matters: repos before commands, commands before deleting the stubs.

- [ ] `domain/` structs with `serde::Serialize`. Field names must serialize to exactly what
      `frontend/src/types/index.ts` expects. Use `#[serde(rename_all = "camelCase")]`.
- [ ] `repo/` sqlx queries. Use the `query!` macros so the schema is checked at compile time.
- [ ] `query/filter.rs`: port `UserReleaseSpecification` (74 lines) to a `QueryBuilder<Sqlite>`.
      This is the fiddliest translation in the project.

Two things to get right in the filter port:

**Search ports verbatim.** The Java already does `cb.lower(field) LIKE lower(pattern)`,
so `LOWER(artist) LIKE ?` works in SQLite unchanged. No ILIKE problem. Same for the
country and genre comparisons.

**Sorting is now an injection risk.** `ReleaseFilterParams.sort` is a client-supplied
string. JPA made that safe. Raw SQL does not. Whitelist the sortable columns to an enum
and reject anything else.

- [ ] Preserve the `PageResponse<T>` shape, which means a `COUNT(*)` query alongside the page
- [ ] Port `ReleaseService.create()` dedup. This gets *simpler*: single-process SQLite means
      `INSERT ... ON CONFLICT DO NOTHING RETURNING id` replaces catching
      `DataIntegrityViolationException` and retrying the lookup.
- [ ] Port `StatsService` (87 lines) to five SQL aggregate queries
- [ ] `error.rs`: one `AppError` enum, serialized to the frontend as `{ code, message }`
- [ ] Replace the Phase 1 stubs command by command, verifying each view as you go

---

## Phase 4 — Resolvers

Reordered so the free, unauthenticated provider lands first.

- [ ] `resolve/musicbrainz.rs` (from 279 Java lines). `reqwest` with the required
      `User-Agent`. Enforce the 1 request/second limit with a shared
      `tokio::time::Interval`, not a sleep per call.
- [ ] `resolve/coverart.rs` for Cover Art Archive, including the retry-on-transient
      behaviour from commit `f640c1c`
- [ ] `resolve/mod.rs` (from `MetadataResolverService`, 210 lines). Input-type detection
      stays as is. `Mono.zip()` becomes `tokio::try_join!`; the 8 second budget becomes
      `tokio::time::timeout`.
- [ ] Provider trait so a disabled or failing provider degrades instead of failing the resolve

Then, behind a settings toggle and only if wanted:
- [ ] Settings view and a `settings` table for per-provider enablement
- [ ] Authorization Code + PKCE with `tauri-plugin-deep-link` for the callback
- [ ] OAuth tokens in the OS keychain via `keyring`, never in the SQLite file
- [ ] Spotify: expect the 5-user cap and the Client Credentials deprecation. Treat any
      Spotify work as disposable.

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

- **`ProfileView` is rebuilt as a local Settings view.** It keeps its route and its panel
  styling, and loses the account fields. It gains, in Phase 4, the per-provider metadata
  toggles and the OAuth connect buttons. `displayName`, email and password change all go.
  This means `ProfileView.vue` is *not* deleted in Phase 1 as originally written. It is
  gutted and refilled. The Phase 1 checklist has been amended accordingly.

## Open decisions

1. Ship Spotify at all in v1, given it is on a deprecation path?
2. Is the Postgres data import needed, or start the local DB empty?
