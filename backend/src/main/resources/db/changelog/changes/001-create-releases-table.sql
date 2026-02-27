--liquibase formatted sql
--changeset trecker:001-create-releases-table
CREATE TABLE releases (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist         VARCHAR(500) NOT NULL,
    title          VARCHAR(500) NOT NULL,
    release_type   VARCHAR(50),
    release_year   INTEGER,
    album_art_url  VARCHAR(2000),
    status         VARCHAR(20) NOT NULL DEFAULT 'QUEUED',
    discovery_link VARCHAR(2000),
    streaming_link VARCHAR(2000),
    country        VARCHAR(100),
    rating         SMALLINT CHECK (rating BETWEEN 1 AND 5),
    did_not_finish BOOLEAN NOT NULL DEFAULT FALSE,
    date_listened  TIMESTAMP WITH TIME ZONE,
    notes          TEXT,
    created_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
--rollback DROP TABLE releases;
