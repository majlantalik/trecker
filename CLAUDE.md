# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development environment

The primary dev environment is Docker Compose — no local Java/Node required to run the app.

```bash
docker compose up                   # start all services (db, backend, frontend)
docker compose restart frontend     # restart only frontend (after frontend changes)
docker compose logs -f backend      # tail backend logs
docker compose logs -f frontend     # tail frontend logs
```

After making frontend changes: restart the frontend container, then verify at http://localhost:5173 using Playwright MCP.

**Local builds** (without Docker):
```bash
# Backend — must use Java 25
JAVA_HOME=/home/mtulek/.sdkman/candidates/java/25.0.2-amzn ./gradlew build -x test  # from backend/
JAVA_HOME=/home/mtulek/.sdkman/candidates/java/25.0.2-amzn ./gradlew bootRun        # from backend/

# Frontend
npm install && npm run dev    # from frontend/
npm run build                 # production build
```

There are no automated tests yet.

## Data model: two-table split

The core architecture is a **shared catalog / per-user tracking split**:

- `releases` — deduplicated catalog. Keyed by `spotify_id` or `musicbrainz_id` (both UNIQUE). Holds: artist, title, year, artwork, country, genres, streaming links.
- `user_releases` — per-user tracking row. Holds: status (`QUEUED`/`LISTENED`), rating, notes, `did_not_finish`, `date_listened`, `discovery_link`.

Every API response merges both via `ReleaseResponse.from(Release, UserRelease)` — this is the only factory method. There is no single-argument version.

**Delete** removes only the `user_releases` row; the catalog `releases` row stays.

**Deduplication on create**: `ReleaseService.create()` first looks up by `spotify_id`/`musicbrainz_id`. On concurrent inserts it catches `DataIntegrityViolationException` and retries the lookup.

## Backend architecture

Package root: `cz.mtulek.trecker` → `backend/src/main/java/cz/mtulek/trecker/`

| Package | Purpose |
|---|---|
| `domain/` | JPA entities: `Release`, `UserRelease`, `User`, `Genre`, enums |
| `dto/` | Request/response records + stats DTOs |
| `controller/` | `ReleaseController`, `GenreController`, `StatsController`, `AuthController`, `ProfileController` |
| `service/` | Business logic; `resolve/` sub-package for metadata fetching |
| `service/resolve/` | `MetadataResolverService` (dispatcher), `SpotifyService`, `MusicBrainzService`, `TidalService`, `YouTubeService` |
| `specification/` | `UserReleaseSpecification` — JPA Criteria API for filtered queries; roots in `UserRelease`, INNER JOINs to `Release`; always call `query.distinct(true)` when joining genres |
| `security/` | `SecurityConfig`, `JwtAuthFilter`, `JwtService`, `AuthService` |
| `repository/` | Spring Data repos: `ReleaseRepository`, `UserReleaseRepository`, `UserRepository`, `GenreRepository` |

**Metadata resolution flow**: `POST /releases/resolve` → `MetadataResolverService` detects input type (Spotify URL, Tidal URL, YouTube URL, plain text) → dispatches to appropriate service → returns `ResolvedMetadata`. Spotify + MusicBrainz calls run in parallel via `Mono.zip()` with an 8s timeout. MusicBrainz requires `User-Agent` header and has a 1100ms rate-limit delay built in.

**Security**: Stateless JWT. `access_token` cookie (15 min, path=/api) + `refresh_token` cookie (30d, path=/api/auth/refresh). SHA-256 of refresh token stored in DB. `User implements UserDetails` directly — `@AuthenticationPrincipal User user` works in controllers. `DaoAuthenticationProvider` requires `UserDetailsService` passed to its constructor (Spring Security 7 removed the no-arg constructor).

**Profile**: `ProfileController` exposes `GET /profile`, `PATCH /profile` (update `displayName`), and `POST /profile/password` (change password; returns `400` if current password wrong). `UserDto` includes `displayName: string | null`. The auth store's `updateDisplayName()` action updates the sidebar reactively without a page reload.

**CORS**: Configured in `SecurityConfig.corsConfigurationSource()` only. No separate `CorsConfig` bean. `allowCredentials(true)` means wildcard `*` origins are forbidden; use explicit origins in `CORS_ALLOWED_ORIGINS`.

## Frontend architecture

Framework: Vue 3 + Vite + PrimeVue 4 (Aura theme) + Pinia + Axios

| Layer | Files |
|---|---|
| API | `src/api/axios.ts` (base client), `releases.ts`, `genres.ts`, `stats.ts`, `auth.ts`, `profile.ts` |
| Stores | `src/stores/releases.ts`, `genres.ts`, `stats.ts`, `auth.ts` (Pinia) |
| Types | `src/types/index.ts` — single source of truth for all TS interfaces |
| Views | `QueueView`, `LibraryView`, `StatsView`, `EntryView`, `LoginView`, `RegisterView`, `ProfileView` |
| Components | `release/` (QuickAddBar, ReleaseForm, QuickLogModal, GenreTagInput), `stats/` charts |

**Auth flow**: `fetchMe()` is called before `app.mount()` in `main.ts` so the auth store is populated before the router guard runs. The Axios interceptor in `axios.ts` silently calls `/auth/refresh` on any 401, then retries the original request. It uses a dynamic import for the auth store to avoid a circular dependency (auth store → axios → auth store).

**API base URL**: In dev, Vite proxies `/api` → `http://backend:8080` (set in `vite.config.ts`). `VITE_API_BASE_URL` is only used for production image builds.

## Database migrations

Liquibase manages schema. Migrations run automatically on startup. To add one:
1. Create `backend/src/main/resources/db/changelog/changes/NNN-description.sql` using Liquibase formatted-SQL.
2. Add the include entry to `db.changelog-master.yaml`.

Current highest migration: `009-add-display-name-to-users.sql`.

## Key gotchas

- **Gradle 9.0 required**: Spring Boot 4.0.0 + Java 25. The committed wrapper (`./gradlew`) is already Gradle 9.0.
- **`primeicons` is a separate package** — must be listed in `package.json` explicitly.
- **`@primevue/themes` 4.5.x**: deprecated upstream but Aura theme still works; do not migrate to `@primeuix/themes` yet.
- **`UserReleaseSpecification`**: always use `query.distinct(true)` when the spec joins genres to prevent duplicate rows.
- **`ReleaseResponse.from(Release, UserRelease)`**: the only factory — always requires both arguments.

## Environment setup

Copy `.env.example` to `.env`. Required variables:
- `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`
- `JWT_SECRET` — generate with `openssl rand -base64 32` (≥32 bytes)
- `CORS_ALLOWED_ORIGINS` — e.g. `http://localhost:5173`
- `SPOTIFY_CLIENT_ID` / `SPOTIFY_CLIENT_SECRET` — recommended for metadata resolution
