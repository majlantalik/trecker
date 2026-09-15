-- Artists to check: a backlog of artists, alongside the queue of albums.
--
-- The same catalog / tracking split as releases (ADR 0002, ADR 0007). `artists` holds what
-- MusicBrainz says about an artist and is keyed by the MusicBrainz artist id, so it dedupes
-- by definition. `user_artists` holds what you think of them.
--
-- The discography is deliberately not stored. It changes as artists release albums, and
-- one MusicBrainz request fetches it whenever the artist's page is opened.

-- ---------------------------------------------------------------- catalog

CREATE TABLE artists (
    id                    TEXT PRIMARY KEY,
    musicbrainz_artist_id TEXT UNIQUE,
    name                  TEXT NOT NULL,
    -- MusicBrainz's note telling same-named artists apart, such as "American metal band".
    disambiguation        TEXT,
    -- Group, Person, Orchestra, Choir, Character or Other, as MusicBrainz names them.
    artist_type           TEXT,
    -- A country code, or an area name such as "England" when there is no code, exactly as
    -- releases.country.
    country               TEXT,
    begin_year            INTEGER,
    end_year              INTEGER,
    -- A cover of one of their albums. MusicBrainz has no artist photos.
    image_url             TEXT,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
);

-- Plain names rather than rows in `genres`. That table feeds the Library's genre filter and
-- the stats, which are about albums you track; an artist's genres must not appear there.
CREATE TABLE artist_genres (
    artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    position  INTEGER NOT NULL,
    PRIMARY KEY (artist_id, name)
);

CREATE TABLE artist_links (
    artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    -- MusicBrainz's relationship type, such as "official homepage" or "bandcamp".
    kind      TEXT NOT NULL,
    url       TEXT NOT NULL,
    PRIMARY KEY (artist_id, url)
);

-- ---------------------------------------------------------------- tracking

CREATE TABLE user_artists (
    id         TEXT PRIMARY KEY,
    artist_id  TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    status     TEXT NOT NULL DEFAULT 'TO_CHECK' CHECK (status IN ('TO_CHECK', 'CHECKED')),
    -- Only for a checked artist, and optional even then.
    verdict    TEXT CHECK (verdict IS NULL OR verdict IN ('LIKED', 'NOT_FOR_ME')),
    note       TEXT,
    checked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (artist_id, user_id),
    CHECK (status = 'CHECKED' OR (verdict IS NULL AND checked_at IS NULL))
);

CREATE INDEX idx_user_artists_user_status ON user_artists(user_id, status);
