# 0001. Local-first Tauri app with a Rust core

- **Status:** Accepted
- **Date:** 2026-09-11

## Context

Trecker began as a Spring Boot web app with a Vue frontend and a Postgres database, with
email and password accounts and cookie-based JWTs. It is a personal music library. Its
data is small, private, and only useful to the person who entered it.

Running it meant hosting a server and a database, keeping both alive, and asking every
user for an account. Nothing about the product needs any of that. The one feature that
genuinely needs a network, metadata lookup, talks to public services directly.

The goal became a desktop app for Windows, macOS and Linux that owns its data on disk and
works with no account. Mobile is deferred.

## Decision

Build a local-first desktop app on **Tauri 2**:

- The existing **Vue 3 frontend is kept**, running in the operating system's own webview.
  Only `frontend/src/api/` changes, from HTTP calls to Tauri commands.
- The core is **Rust**, exposed to the frontend as a narrow set of typed commands.
- Data lives in a **local SQLite file**. See [ADR 0002](0002-sqlite-with-catalog-and-tracking-split.md).
- `backend/` is **frozen, not deleted**, as a starting point should sync ever be built.

Adoption was gated on a spike. WebKitGTK is the weakest of the three webviews, so the
Linux build had to render the existing UI acceptably before any Rust was written. It did,
on WebKitGTK 2.52.6, with no rendering divergence from Chromium.

## Alternatives considered

- **Keep the web app.** Rejected because it keeps every cost the product does not need:
  hosting, uptime, accounts, and a server holding people's listening history.
- **Electron.** The same frontend reuse, but it ships a copy of Chromium with every app and
  its backend is Node. Tauri's system webview keeps the binary small, and Rust gives the
  core a type system and a SQL layer checked at compile time.
- **Compose Multiplatform or Flutter.** Both mean rewriting all of the UI. Compose was the
  documented fallback had the WebKitGTK spike failed.
- **Separate native apps per platform.** Three codebases for one person's side project.

## Consequences

- **The whole UI survived.** All 42 frontend files were kept, and the stores and views
  still know nothing about Tauri. `frontend/src/api/` is the only place that imports it.
- **The binary is small.** 4.2 MB at the spike, 7.7 MB once native file dialogs were added.
- **There is no server and no account,** so there is also no multi-user support and no sync.
  Both would have to be built deliberately.
- **Three webviews must be supported,** not one browser. WebKitGTK sets the floor for CSS.
- **Some things became harder to verify.** On this Linux desktop, automation cannot send
  keyboard input to the webview or take screenshots, so shortcuts need a human and UI
  checks read the accessibility tree.
- **Distribution is now a problem Trecker owns:** bundles, code signing and updates. Builds
  are currently unsigned.
- **A global shortcut is harder than an in-app one.** It needs a tray icon, and Wayland
  restricts global key capture.
