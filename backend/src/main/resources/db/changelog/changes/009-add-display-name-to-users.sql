--liquibase formatted sql
--changeset trecker:009-add-display-name-to-users
ALTER TABLE users ADD COLUMN display_name VARCHAR(100);
--rollback ALTER TABLE users DROP COLUMN display_name;
