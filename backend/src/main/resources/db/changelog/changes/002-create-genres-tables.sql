--liquibase formatted sql
--changeset trecker:002-create-genres-tables
CREATE TABLE genres (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE release_genres (
    release_id UUID    NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    genre_id   INTEGER NOT NULL REFERENCES genres(id)  ON DELETE CASCADE,
    PRIMARY KEY (release_id, genre_id)
);
--rollback DROP TABLE release_genres; DROP TABLE genres;
