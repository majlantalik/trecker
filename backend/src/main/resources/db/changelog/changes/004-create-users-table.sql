--liquibase formatted sql
--changeset trecker:004-create-users-table
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_users_email UNIQUE (email)
);
CREATE INDEX idx_users_email ON users(email);
--rollback DROP INDEX idx_users_email; DROP TABLE users;
