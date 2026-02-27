--liquibase formatted sql
--changeset trecker:007-shared-catalog-model

-- 1. Add external dedup IDs to releases catalog
ALTER TABLE releases ADD COLUMN spotify_id VARCHAR(255);
ALTER TABLE releases ADD COLUMN musicbrainz_id VARCHAR(36);
ALTER TABLE releases ADD CONSTRAINT uq_releases_spotify_id UNIQUE (spotify_id);
ALTER TABLE releases ADD CONSTRAINT uq_releases_musicbrainz_id UNIQUE (musicbrainz_id);

-- 2. Create release_streaming_links (catalog - multiple streaming services per release)
CREATE TABLE release_streaming_links (
    release_id UUID        NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    service    VARCHAR(50) NOT NULL,
    url        VARCHAR(2000) NOT NULL,
    PRIMARY KEY (release_id, service)
);

-- 3. Migrate existing streaming_link data (infer service from URL)
INSERT INTO release_streaming_links (release_id, service, url)
SELECT id,
    CASE
        WHEN streaming_link LIKE '%spotify.com%' THEN 'spotify'
        WHEN streaming_link LIKE '%tidal.com%'   THEN 'tidal'
        WHEN streaming_link LIKE '%youtube.com%' OR streaming_link LIKE '%youtu.be%' THEN 'youtube'
        ELSE 'other'
    END,
    streaming_link
FROM releases
WHERE streaming_link IS NOT NULL;

-- 4. Create user_releases tracking table
CREATE TABLE user_releases (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    release_id     UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status         VARCHAR(20) NOT NULL DEFAULT 'QUEUED',
    rating         SMALLINT CHECK (rating BETWEEN 1 AND 5),
    did_not_finish BOOLEAN NOT NULL DEFAULT FALSE,
    date_listened  TIMESTAMP WITH TIME ZONE,
    notes          TEXT,
    discovery_link VARCHAR(2000),
    created_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_releases UNIQUE (release_id, user_id)
);
CREATE INDEX idx_user_releases_user_id       ON user_releases(user_id);
CREATE INDEX idx_user_releases_user_status   ON user_releases(user_id, status);
CREATE INDEX idx_user_releases_user_listened ON user_releases(user_id, date_listened);

-- 5. Migrate existing data into user_releases
INSERT INTO user_releases
    (release_id, user_id, status, rating, did_not_finish,
     date_listened, notes, discovery_link, created_at)
SELECT id, user_id, status, rating, did_not_finish,
       date_listened, notes, discovery_link, created_at
FROM releases
WHERE user_id IS NOT NULL;

-- 6. Remove migrated columns from releases (catalog only)
ALTER TABLE releases DROP COLUMN user_id;
ALTER TABLE releases DROP COLUMN status;
ALTER TABLE releases DROP COLUMN rating;
ALTER TABLE releases DROP COLUMN did_not_finish;
ALTER TABLE releases DROP COLUMN date_listened;
ALTER TABLE releases DROP COLUMN notes;
ALTER TABLE releases DROP COLUMN discovery_link;
ALTER TABLE releases DROP COLUMN streaming_link;

--rollback (complex - requires recreating dropped columns and reversing migration)
