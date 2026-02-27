--liquibase formatted sql
--changeset trecker:006-add-user-id-to-releases
ALTER TABLE releases ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
CREATE INDEX idx_releases_user_id ON releases(user_id);
--rollback DROP INDEX idx_releases_user_id; ALTER TABLE releases DROP COLUMN user_id;
