-- Initial schema.
--
-- This is the Postgres schema as it stands after Liquibase changesets 001 through 010,
-- collapsed into one file. The ten-step history has no value here: there is no deployed
-- SQLite database to migrate forward from, so this reproduces the end state directly.
--
-- Type mapping from Postgres:
--   UUID           -> TEXT (36 chars, not BLOB, so the file stays greppable)
--   TIMESTAMPTZ    -> TEXT, RFC3339 / ISO-8601 in UTC
--   VARCHAR(n)     -> TEXT (SQLite does not enforce length; the limits were never load-bearing)
--   BOOLEAN        -> INTEGER 0/1 with a CHECK
--   DECIMAL(3,1)   -> REAL
--   SERIAL         -> INTEGER PRIMARY KEY (rowid alias, auto-assigns)
--
-- Dropped entirely: users, refresh_tokens. A local app has no accounts.

-- ---------------------------------------------------------------- catalog

CREATE TABLE releases (
    id             TEXT PRIMARY KEY,
    artist         TEXT NOT NULL,
    title          TEXT NOT NULL,
    release_year   INTEGER,
    album_art_url  TEXT,
    country        TEXT,
    spotify_id     TEXT UNIQUE,
    musicbrainz_id TEXT UNIQUE,
    created_at     TEXT NOT NULL
);

CREATE INDEX idx_releases_artist       ON releases(artist);
CREATE INDEX idx_releases_country      ON releases(country);
CREATE INDEX idx_releases_release_year ON releases(release_year);

CREATE TABLE genres (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE release_genres (
    release_id TEXT    NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    genre_id   INTEGER NOT NULL REFERENCES genres(id)   ON DELETE CASCADE,
    PRIMARY KEY (release_id, genre_id)
);

CREATE INDEX idx_release_genres_genre_id ON release_genres(genre_id);

-- Was an @ElementCollection Map<service, url> on the JPA side. It was already a child
-- table in Postgres, so it needs no restructuring.
CREATE TABLE release_streaming_links (
    release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    service    TEXT NOT NULL,
    url        TEXT NOT NULL,
    PRIMARY KEY (release_id, service)
);

-- ---------------------------------------------------------------- tracking

-- The catalog / tracking split is kept even though there is exactly one local user.
-- Catalog rows are content-addressed by spotify_id or musicbrainz_id, so they dedupe by
-- definition and can never conflict; only this table needs conflict resolution. That is
-- what makes the Phase 6 sync tractable, and it costs nothing to preserve now.
CREATE TABLE user_releases (
    id             TEXT PRIMARY KEY,
    release_id     TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    user_id        TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    status         TEXT NOT NULL DEFAULT 'QUEUED' CHECK (status IN ('QUEUED', 'LISTENED')),
    rating         REAL CHECK (rating IS NULL OR (rating >= 0.5 AND rating <= 5.0)),
    did_not_finish INTEGER NOT NULL DEFAULT 0 CHECK (did_not_finish IN (0, 1)),
    date_listened  TEXT,
    notes          TEXT,
    discovery_link TEXT,
    created_at     TEXT NOT NULL,
    -- Not in the Postgres schema. Added now because last-write-wins per field needs it,
    -- and adding a column later to a table full of rows is worse than carrying it early.
    updated_at     TEXT NOT NULL,
    UNIQUE (release_id, user_id)
);

CREATE INDEX idx_user_releases_user_id       ON user_releases(user_id);
CREATE INDEX idx_user_releases_user_status   ON user_releases(user_id, status);
CREATE INDEX idx_user_releases_user_listened ON user_releases(user_id, date_listened);
CREATE INDEX idx_user_releases_release_id    ON user_releases(release_id);

-- ---------------------------------------------------------------- search

-- New capability, not a port. The Postgres version did case-insensitive LIKE scans over
-- artist and title; FTS5 gives prefix and token matching over the same two columns.
-- External-content table: the index stores no copy of the text, it points back at
-- releases by rowid, and the triggers below keep it in step.
CREATE VIRTUAL TABLE releases_fts USING fts5(
    artist,
    title,
    content='releases',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER releases_fts_insert AFTER INSERT ON releases BEGIN
    INSERT INTO releases_fts(rowid, artist, title)
    VALUES (new.rowid, new.artist, new.title);
END;

CREATE TRIGGER releases_fts_delete AFTER DELETE ON releases BEGIN
    INSERT INTO releases_fts(releases_fts, rowid, artist, title)
    VALUES ('delete', old.rowid, old.artist, old.title);
END;

CREATE TRIGGER releases_fts_update AFTER UPDATE ON releases BEGIN
    INSERT INTO releases_fts(releases_fts, rowid, artist, title)
    VALUES ('delete', old.rowid, old.artist, old.title);
    INSERT INTO releases_fts(rowid, artist, title)
    VALUES (new.rowid, new.artist, new.title);
END;
