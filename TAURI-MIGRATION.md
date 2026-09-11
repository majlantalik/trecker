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
- [ ] Add `@tauri-apps/api`; delete `axios` and `axios-mock-adapter`
- [ ] Rewrite `releases.ts`, `genres.ts`, `stats.ts` bodies
- [ ] Delete `api/axios.ts`, `api/axios.test.ts`, `api/auth.ts`, `api/profile.ts`
- [ ] Delete `stores/auth.ts`, `stores/auth.test.ts`
- [ ] Delete `views/LoginView.vue`, `views/RegisterView.vue`, `views/ProfileView.vue`
- [ ] Strip auth guards and the login/register/profile routes from `router/index.ts`
- [ ] Remove the `fetchMe()` call from `main.ts`
- [ ] Rewrite `ProfileView` later as a local Settings view (Phase 4 needs it for credentials)
- [ ] Keep `stores/releases.test.ts` and the component tests; retarget mocks from axios to `invoke`

Note `stores/releases.ts` (115 lines) and the views should need no changes beyond the
mock retarget. If they do, the seam was leakier than expected — fix that, do not spread it.

**Exit criterion:** app runs on stub commands returning fixtures, with no network layer.

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

## Open decisions

1. Keep `ProfileView` as a local Settings view, or build a fresh one?
2. Ship Spotify at all in v1, given it is on a deprecation path?
3. Is the Postgres data import needed, or start the local DB empty?
