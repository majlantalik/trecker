# 0004. The MusicBrainz release group is an album's identity

- **Status:** Accepted
- **Date:** 2026-09-12

## Context

MusicBrainz models an album as a **release group** holding many **releases**. A release is
one specific pressing: a regional edition, a reissue, a format, a clean or explicit
version. City of Evil is one release group containing 14 releases.

Trecker originally searched releases and stored the matched release's id as the catalog's
dedup key. Which pressing a search returns is close to arbitrary, and every metadata bug
the resolver had came from trusting it:

- the **year** was that edition's, so Spiderland could come back as a 2014 reissue;
- the **country** was wherever that edition was sold, so an American band showed a
  Canadian flag;
- the pressing often had **no genres or cover art** of its own when the album plainly did;
- **adding one album twice** could match two different pressings, which dedup saw as two
  albums.

Year, genres and artwork had already been patched by borrowing them from the pressing's
release group. Identity had not.

## Decision

Everything comes from the **release group**, never a release.

- Search queries release groups. The Cover Art Archive is asked for the group's cover.
- The catalog's unique key is **`musicbrainz_release_group_id`.**
- **Country is the credited artist's,** falling back to their area name, such as England.
  A release group has no country, since where a record was sold is a property of a
  pressing. There is no fallback to a pressing's country.
- **Refresh looks the group up by its stored id** instead of searching again. Only a
  release typed in by hand is refreshed by searching its artist and title.
- **Search ties are broken toward the album.** MusicBrainz scores text alone, so every exact
  title match ties at 100 in no useful order. Among tied hits, a plain album beats one with
  a secondary type such as live or compilation, which beats every other type. The group
  with the most releases wins what remains. Ties are reordered, never filtered.
- **"Release group" never reaches the interface.** The UI says album, and the export field
  stays `musicbrainzId`. Code, schema and developer docs use the precise term.

## Alternatives considered

- **Keep release ids and patch fields from the group.** This was the previous state. It
  fixes display fields and leaves dedup keyed on something arbitrary.
- **Store both ids.** A release id only matters if Trecker tracks which edition you own,
  such as a particular vinyl pressing. It does not, and a nullable column can be added if
  it ever does.
- **Filter search to albums only.** Simpler, and it would make EPs and singles impossible
  to find.
- **Show "release group" in the UI for precision.** Rejected because to anyone who has not
  read MusicBrainz's model it suggests a group of albums.

## Consequences

- **Dedup now means one row per album,** whichever pressing a search would have landed on.
- **Search results improved beyond the original bugs.** "The Weeknd - After Hours" had
  ranked the title-track single first, and "Metallica - Metallica" had ranked a live
  bootleg, an interview disc and a compilation above the 1991 album. Both now resolve to
  the album.
- **Refresh can no longer drift** onto a different album with a similar name.
- **Some country fields are blank** where they would previously have shown a pressing's
  country. That is intended.
- **The request count is unchanged:** a group search and a group lookup replaced a release
  search and a release lookup.
- **The export's `musicbrainzId` holds a release group id,** which its name does not say.
  `docs/export-format.md` documents this, and anyone querying it as a release id gets a
  not-found.
- The change was made by editing the initial migration, which was only possible because
  one install existed. See [ADR 0002](0002-sqlite-with-catalog-and-tracking-split.md).
