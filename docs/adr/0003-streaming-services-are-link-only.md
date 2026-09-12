# 0003. Streaming services are link-only

- **Status:** Accepted
- **Date:** 2026-09-11

## Context

The web app resolved pasted links and searches through Spotify, Tidal, YouTube and
MusicBrainz in parallel. Spotify supplied artwork and genres, and MusicBrainz supplied
identity. That worked because a server held the API credentials.

A desktop app has no server. Any credential it uses ships inside the binary, and a secret
in a distributed binary is not a secret.

- **Spotify** needs a client secret. Per-user OAuth avoids shipping one, but an app in
  Development Mode is capped at 5 allowlisted users and its owner must hold a Premium
  subscription. Extended Quota requires 250,000 monthly active users.
- **Tidal** uses the same client-credentials flow and has the same problem.
- **YouTube** needs an API key, which anyone can extract from the binary and spend against
  the developer's quota.

## Decision

Spotify, Tidal and YouTube are **link-only** and will stay that way.

- A pasted streaming link is **saved as a link.** Sharing and tracking parameters such as
  `si=` and `utm_` are stripped. Nothing is queried through it.
- **Metadata comes from MusicBrainz** and **artwork from the Cover Art Archive.** Neither
  needs an account, a key or approval.
- **Bandcamp and Apple Music** links carry the artist and album in the URL path, so that
  text is read out and searched on MusicBrainz. Spotify and Tidal URLs carry opaque ids
  and yield nothing, which is the correct outcome.

## Alternatives considered

- **Ship the credentials anyway.** They would be extracted, and the account they belong
  to would be revoked or billed.
- **Bring your own credentials.** Every user would need a Premium subscription and a
  registered developer app before adding an album. Not reasonable to ask.
- **Per-user OAuth against one shared app.** Capped at five users by Spotify's policy.
- **A small proxy server holding the credentials.** Technically sound, and it brings back
  the hosted server that [ADR 0001](0001-local-first-tauri-app.md) exists to remove.

## Consequences

- **A pasted Spotify link produces a saved link and an empty form,** not a filled one. The
  user types or searches the artist and album.
- **MusicBrainz is the backbone,** so its constraints are Trecker's: a descriptive
  `User-Agent` and at most one request per second, enforced by a single rate-limited
  resolver.
- **Coverage follows MusicBrainz.** Obscure or very new releases may be missing or
  untagged where Spotify would have had them.
- **The `spotify_id` column was removed** once nothing wrote to it, and catalog dedup now
  rests on MusicBrainz alone.
- The Info view says plainly which services are looked up and which are link-only, rather
  than offering toggles that would do nothing.
