--liquibase formatted sql
--changeset trecker:010-rating-decimal

ALTER TABLE user_releases DROP CONSTRAINT user_releases_rating_check;
ALTER TABLE user_releases ALTER COLUMN rating TYPE DECIMAL(3,1) USING rating::DECIMAL(3,1);
ALTER TABLE user_releases ADD CONSTRAINT user_releases_rating_check
    CHECK (rating >= 0.5 AND rating <= 5.0);

--rollback ALTER TABLE user_releases DROP CONSTRAINT user_releases_rating_check;
--rollback ALTER TABLE user_releases ALTER COLUMN rating TYPE SMALLINT USING rating::SMALLINT;
--rollback ALTER TABLE user_releases ADD CONSTRAINT user_releases_rating_check CHECK (rating BETWEEN 1 AND 5);
