# Trecker

A personal music library and listening tracker. Discover releases via links (Spotify, YouTube, Tidal, Discord), queue them, listen on your streaming platform of choice, then log them with rating, genres, country, and notes.

Replaces a two-Notion-page workflow with a single purpose-built tool.

**Core flow:** Quick Add → Queue → Mark as Listened → Library

---

## User Guide

### Adding releases

Paste a link or type a search query into the **Quick Add** bar at the top of any page.

- **Spotify album link** — metadata (artist, title, year, artwork, genres) is fetched automatically from Spotify. Country of origin is looked up from MusicBrainz.
- **YouTube link** — title is parsed and used to search Spotify for full metadata.
- **Plain text** (e.g. `Portishead Dummy`) — searched on Spotify directly.
- **Tidal album link** — metadata (artist, title, year, artwork) is fetched via the Tidal API. Country of origin is looked up from MusicBrainz. Requires `TIDAL_CLIENT_ID` and `TIDAL_CLIENT_SECRET`.
- **Any other link** — form opens for manual entry.

After resolution, a pre-filled form opens. Review the data, adjust anything, and click **Add to Queue**.

### Queue

The **Queue** view lists everything you haven't listened to yet.

- Click **Log** on any entry to open the Quick Log modal and mark it as listened.
- Click **Pick one for me** to randomly select a queued release.
- Click the title/artwork to open the full Entry page.

### Quick Log modal

When marking a release as listened, you can optionally fill in:

| Field | Description |
|---|---|
| Rating | 1–5 stars |
| Genres | Pre-filled from Spotify; editable |
| Country | Pre-filled from MusicBrainz; editable |
| Notes | Free-text thoughts |
| Did not finish | Flag for abandoned listens |

Click **Log it** to save and return, or **Log + Details** to save and navigate to the full Entry page for more editing.

### Library

The **Library** view shows all listened releases. Switch between table and grid layout using the toggle in the top-right.

Filter by:
- Free-text search (artist or title)
- Status (Queued / Listened)
- Genre
- Release type (Album, EP, Single, …)
- Country
- Year
- Minimum rating
- Did-not-finish flag

### Stats

The **Stats** view shows:
- **Listening activity** — bar chart of albums logged per month
- **Breakdowns** — doughnut charts by genre, country, and release type
- **Top rated** — your highest-rated releases
- **Year-end list** — ranked list for any year, navigable with the year selector

---

## Developer Guide

### Tech stack

| Layer | Technology |
|---|---|
| Backend | Spring Boot 4.0.0 / Java 25 / Gradle 9.0 (Kotlin DSL) |
| Frontend | Vue 3 + Vite + PrimeVue 4 (Aura theme) |
| State management | Pinia |
| HTTP client | Axios |
| Database | PostgreSQL 17 |
| DB migrations | Liquibase |
| Charts | vue-chartjs + Chart.js |
| Containerisation | Docker Compose |

Authentication: multi-user email/password with HttpOnly cookie-based JWT tokens and refresh token rotation.

### Prerequisites

| Tool | Version |
|---|---|
| Java | 25 (via sdkman: `sdk install java 25.0.2-amzn`) |
| Node.js | 20+ |
| Docker + Docker Compose | any recent version |
| Gradle | not required locally — the wrapper (`./gradlew`) is committed |

### Project structure

```
trecker/
├── backend/                     Spring Boot application
│   ├── Dockerfile               Multi-stage production image
│   ├── build.gradle.kts
│   └── src/main/
│       ├── java/cz/mtulek/trecker/
│       │   ├── config/          CorsConfig, WebClientConfig
│       │   ├── controller/      ReleaseController, GenreController, StatsController
│       │   ├── domain/          Release, Genre, ReleaseStatus, ReleaseType
│       │   ├── dto/             Request/response records + stats DTOs
│       │   ├── exception/       GlobalExceptionHandler, ResourceNotFoundException
│       │   ├── repository/      ReleaseRepository (+ native queries), GenreRepository
│       │   ├── service/         ReleaseService, GenreService, StatsService
│       │   │   └── resolve/     MetadataResolverService, SpotifyService,
│       │   │                    MusicBrainzService, TidalService, YouTubeService
│       │   ├── specification/   ReleaseSpecification (JPA Criteria API)
│       │   └── TreckerApplication.java
│       └── resources/
│           ├── application.yml
│           ├── application-dev.yml
│           ├── application-prod.yml
│           └── db/changelog/    Liquibase master + 3 changesets
├── frontend/                    Vue 3 application
│   ├── Dockerfile               Multi-stage production image (nginx)
│   ├── nginx.conf
│   ├── vite.config.ts
│   └── src/
│       ├── api/                 axios.ts, releases.ts, genres.ts, stats.ts
│       ├── components/
│       │   ├── layout/          AppLayout, AppSidebar
│       │   ├── library/         LibraryFilters, ViewToggle
│       │   ├── queue/           QueueActions
│       │   ├── release/         QuickAddBar, ReleaseCard, ReleaseForm,
│       │   │                    QuickLogModal, GenreTagInput
│       │   └── stats/           ActivityChart, BreakdownChart,
│       │                        TopRatedList, YearEndList
│       ├── router/index.ts
│       ├── stores/              releases.ts, genres.ts, stats.ts (Pinia)
│       ├── types/index.ts
│       └── views/               QueueView, LibraryView, StatsView, EntryView
├── docker-compose.yml           Dev stack (hot reload)
├── docker-compose.prod.yml      Production stack (built images)
├── .env.example                 Template for environment variables
└── README.md
```

### Environment variables

Copy `.env.example` to `.env` and fill in the values before starting any stack.

```bash
cp .env.example .env
```

#### Required for all environments

| Variable | Description |
|---|---|
| `POSTGRES_DB` | PostgreSQL database name |
| `POSTGRES_USER` | PostgreSQL username |
| `POSTGRES_PASSWORD` | PostgreSQL password — change from the default |

#### Port configuration (dev only)

| Variable | Default | Description |
|---|---|---|
| `BACKEND_PORT` | `8080` | Host port for the Spring Boot API |
| `FRONTEND_PORT` | `5173` | Host port for the Vite dev server |

#### API integration

| Variable | Required | Description |
|---|---|---|
| `SPOTIFY_CLIENT_ID` | Recommended | Spotify Web API client ID. Without this, metadata resolution falls back to manual entry. Get credentials at [developer.spotify.com](https://developer.spotify.com/dashboard). |
| `SPOTIFY_CLIENT_SECRET` | Recommended | Spotify Web API client secret. |
| `YOUTUBE_API_KEY` | Optional | Google Data API v3 key for YouTube title parsing. Get a key at [console.cloud.google.com](https://console.cloud.google.com). Enable the **YouTube Data API v3**. |
| `TIDAL_CLIENT_ID` | Optional | Tidal API client ID. Required for Tidal URL resolution. Register an app at [developer.tidal.com](https://developer.tidal.com). |
| `TIDAL_CLIENT_SECRET` | Optional | Tidal API client secret. |
| `TIDAL_COUNTRY_CODE` | Optional | ISO 3166-1 alpha-2 country code for Tidal catalog (default: `US`). |

#### Authentication

| Variable | Default | Description |
|---|---|---|
| `JWT_SECRET` | required | Secret key for signing JWTs. Generate with: `openssl rand -base64 32`. Must be ≥ 32 bytes. |
| `JWT_EXPIRATION_SECONDS` | `900` | Access token lifetime in seconds (default 15 minutes). |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | `30` | Refresh token lifetime in days. |
| `COOKIE_SECURE` | `false` | Set to `true` in production (requires HTTPS). Marks cookies as `Secure`. |

#### CORS / networking

| Variable | Default | Description |
|---|---|---|
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173` | Comma-separated list of allowed frontend origins. Set to your production domain in prod. |
| `VITE_API_BASE_URL` | `http://localhost:8080` | Base URL the frontend uses to reach the API. In the dev Docker stack the Vite proxy handles this internally. In production it is baked into the frontend image at build time. |

> **MusicBrainz** requires no API key. The backend sends a `User-Agent` header as required by their policy. Requests are rate-limited to 1 req/s automatically.

> **Tidal** uses OAuth2 Client Credentials. Without credentials, Tidal URLs open the manual entry form. Genres are not available from the Tidal API and are left empty (supplement via the form or MusicBrainz).

### Running in development

The dev stack mounts source directories and runs hot-reload servers — no images need to be built.

```bash
cp .env.example .env
# fill in POSTGRES_PASSWORD, JWT_SECRET (openssl rand -base64 32), optionally SPOTIFY_CLIENT_ID/SECRET, TIDAL_CLIENT_ID/SECRET

docker compose up
```

| Service | URL |
|---|---|
| Frontend (Vite) | http://localhost:5173 |
| Backend API | http://localhost:8080/api |
| Health check | http://localhost:8080/actuator/health |

The backend starts `./gradlew bootRun` inside the container. First startup downloads Gradle dependencies (~2 min). Subsequent starts use the `gradle_cache` volume and are much faster.

**Building locally** (without Docker):

```bash
# Backend — requires Java 25
JAVA_HOME=/home/mtulek/.sdkman/candidates/java/25.0.2-amzn \
  ./gradlew build -x test          # from backend/

# Frontend
npm install && npm run dev          # from frontend/
```

### Running in production

The prod stack builds optimised images from the multi-stage Dockerfiles.

```bash
cp .env.example .env
# set POSTGRES_PASSWORD, SPOTIFY_*, TIDAL_*, CORS_ALLOWED_ORIGINS, VITE_API_BASE_URL

docker compose -f docker-compose.prod.yml up -d
```

The frontend nginx container exposes port 80 and proxies `/api/*` to the backend internally. The backend is not exposed on any host port.

### Database migrations

Schema is managed by Liquibase. Migrations run automatically on startup.

```
backend/src/main/resources/db/changelog/
├── db.changelog-master.yaml
└── changes/
    ├── 001-create-releases-table.sql
    ├── 002-create-genres-tables.sql
    ├── 003-add-indexes.sql
    ├── 004-create-users-table.sql
    ├── 005-create-refresh-tokens-table.sql
    └── 006-add-user-id-to-releases.sql
```

To add a new migration: create `NNN-description.sql` in `changes/` using the Liquibase formatted-SQL convention, then reference it in `db.changelog-master.yaml`.

### API reference

All endpoints are under `/api`. All endpoints except `/auth/**` require a valid `access_token` HttpOnly cookie.

#### Authentication endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/auth/register` | Register a new account. Body: `{ email, password }`. Returns `201` + sets cookies. |
| `POST` | `/auth/login` | Authenticate. Body: `{ email, password }`. Returns `200` + sets cookies. |
| `POST` | `/auth/logout` | Revoke session. Returns `204` + clears cookies. |
| `GET` | `/auth/me` | Returns current user `{ id, email }`. Returns `401` if unauthenticated. |
| `POST` | `/auth/refresh` | Rotate refresh token (reads `refresh_token` cookie). Returns `200` + sets new cookies. |

#### Release endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/releases/resolve` | Resolve a URL or search query to metadata |
| `POST` | `/releases` | Add a release to the queue |
| `GET` | `/releases` | List releases with filtering, sorting, pagination |
| `GET` | `/releases/random` | Pick a random queued release |
| `GET` | `/releases/{id}` | Get a single release |
| `PATCH` | `/releases/{id}` | Partial update; setting `status=LISTENED` auto-sets `dateListened` |
| `DELETE` | `/releases/{id}` | Delete a release |
| `GET` | `/genres` | List all known genres |
| `GET` | `/stats/activity` | Listening count by year/month |
| `GET` | `/stats/by-genre` | Count breakdown by genre |
| `GET` | `/stats/by-country` | Count breakdown by country |
| `GET` | `/stats/by-type` | Count breakdown by release type |
| `GET` | `/stats/top-rated` | Top-rated listened releases (`?limit=25`) |
| `GET` | `/stats/year-end` | Ranked list for a given year (`?year=2024`) |

#### `GET /releases` query parameters

| Parameter | Type | Description |
|---|---|---|
| `status` | `QUEUED` \| `LISTENED` | Filter by status |
| `genre` | string | Filter by genre name (exact, case-insensitive) |
| `country` | string | Filter by country (case-insensitive) |
| `releaseType` | string | Filter by type (e.g. `ALBUM`, `EP`) |
| `year` | integer | Filter by release year |
| `ratingMin` | 1–5 | Minimum rating |
| `ratingMax` | 1–5 | Maximum rating |
| `didNotFinish` | boolean | Filter by DNF flag |
| `search` | string | Free-text search on artist and title |
| `sort` | string | Field to sort by (default: `createdAt`) |
| `direction` | `ASC` \| `DESC` | Sort direction (default: `DESC`) |
| `page` | integer | Zero-based page number (default: `0`) |
| `size` | integer | Page size (default: `20`) |

### Known issues / gotchas

- **Spring Boot 4.0.0 requires Gradle 9.0+.** Gradle 8.14's bundled Kotlin DSL cannot parse Java 25 version strings, causing a build failure. The committed wrapper uses Gradle 9.0.
- **`primeicons` is a separate package.** It is not bundled with `primevue` and must be listed explicitly in `package.json`.
- **`@primevue/themes` 4.5.x** is deprecated upstream (migrate to `@primeuix/themes` when ready). The Aura theme still works from the current package.

---

## Roadmap

- [x] Tidal API integration (`openapi.tidal.com/v2`)
- [x] Authentication (Spring Security + JWT, HttpOnly cookies, refresh token rotation)
- [ ] Profile section (change email / password, account settings)
- [ ] Notion CSV import
- [ ] Export to CSV / JSON
- [ ] Keyboard shortcuts
