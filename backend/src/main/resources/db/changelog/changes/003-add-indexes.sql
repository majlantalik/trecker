--liquibase formatted sql
--changeset trecker:003-add-indexes
CREATE INDEX idx_releases_status ON releases(status);
CREATE INDEX idx_releases_artist ON releases(artist);
CREATE INDEX idx_releases_rating ON releases(rating);
CREATE INDEX idx_releases_date_listened ON releases(date_listened);
CREATE INDEX idx_releases_country ON releases(country);
CREATE INDEX idx_releases_release_year ON releases(release_year);
CREATE INDEX idx_release_genres_genre_id ON release_genres(genre_id);
--rollback DROP INDEX idx_release_genres_genre_id; DROP INDEX idx_releases_release_year; DROP INDEX idx_releases_country; DROP INDEX idx_releases_date_listened; DROP INDEX idx_releases_rating; DROP INDEX idx_releases_artist; DROP INDEX idx_releases_status;
