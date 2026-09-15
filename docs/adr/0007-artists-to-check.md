# 0007. Artists to check are a second kind of entity

- **Status:** Accepted
- **Date:** 2026-09-14

## Context

The queue holds albums you mean to hear. People also collect **artists** they mean to
explore, with no particular album in mind: a name a friend mentioned, a band from a
festival line-up. Putting a guessed album in the queue for each of them misrepresents what
was saved, and nothing in the library could hold an artist on their own.

MusicBrainz covers artists as well as albums, with no credential:

- an **artist search** returns name, type, country, active years, a note telling same-named
  artists apart, and tags;
- an **artist lookup** with `inc=genres+url-rels` adds genres and links, including official
  sites, Bandcamp and streaming pages;
- a **release group browse** by artist lists up to a hundred albums and EPs per request.

It has **no artist photos**, and the Cover Art Archive holds only album covers.

## Decision

- **Artists are stored as their own entity, with the catalog and tracking split of
  [ADR 0002](0002-sqlite-with-catalog-and-tracking-split.md).** `artists` holds what
  MusicBrainz says and is unique on the MusicBrainz artist id; `user_artists` holds your
  status, verdict and note. Removing an artist deletes only the tracking row.
- **An artist is either to check or checked.** A checked artist keeps an optional verdict,
  "into it" or "not for me", and stays in a collapsed section rather than being deleted,
  because who you already dismissed is worth knowing.
- **The discography is fetched each time the page opens, never stored.** It changes as
  artists release albums, and one request brings it back. Offline, the page shows the saved
  artist and says the discography needs a connection.
- **The discography is matched to your library by release group id,** which releases
  already store ([ADR 0004](0004-release-group-is-album-identity.md)), so an album shows
  "In queue" or "Listened" exactly, not by comparing names.
- **Queueing or logging from the discography reuses the album path:** a release group
  lookup for genres, country and cover, then `releases_create`. Logging creates the release
  and opens the log dialog; closing that dialog without logging removes it again.
- **An artist's picture is the cover of their first studio album,** or failing that their
  first EP, taken from the same browse. It goes through the cover cache like any cover.
- **Artist genres are stored as plain names beside the artist,** not in the `genres` table,
  which feeds the Library filter and the stats and describes albums you track.
- **Links are filtered to a fixed list of relationship types** and dead ones are dropped.
  They open in the default browser through `tauri-plugin-opener`.

## Alternatives considered

- **Queue an artist as a placeholder release.** No new schema, but the queue, the stats and
  the export would all count something that is not an album.
- **Store the discography.** Works offline, and goes stale without a refresh mechanism nobody
  would remember to use.
- **Artist photos from Wikimedia Commons through Wikidata.** Two more requests per artist to
  a third service, and another host for the cover cache to fetch from. From fanart.tv: it
  needs an API key, which [ADR 0003](0003-streaming-services-are-link-only.md) rules out.
- **Store an artist id on every release,** so the library could list everything by an
  artist. Collaborations have several credited artists, so this needs a join table and a
  backfill of every existing release. Deferred; the discography already answers "which of
  their albums do I have".
- **Search artists from quick add.** One box for albums and artists makes Enter ambiguous.
  Artists are added from their own page.

## Consequences

- **Adding an artist costs three MusicBrainz requests,** a search, a lookup and a browse,
  about three seconds behind the rate limit. Opening an artist's page costs one.
- **The discography is noisy.** MusicBrainz lists bootlegs and live recordings as albums,
  and some carry no secondary type. Live albums, compilations and the like are hidden until
  asked for; a bootleg mislabelled as a plain album still shows.
- **Some artists have no picture:** the first album has no cover in the archive. The page
  shows their initial instead.
- **Artists are not in the export yet.** `docs/export-format.md` covers releases only, and
  adding artists means a new top-level list in JSON and a decision about CSV, which cannot
  hold them in the same file.
- **External links open in the browser everywhere,** including the release page's streaming
  links. Before the opener plugin nothing handled `target="_blank"`, and wry connects
  WebKitGTK's new-window signal only when a handler is set, so those links opened nothing.
