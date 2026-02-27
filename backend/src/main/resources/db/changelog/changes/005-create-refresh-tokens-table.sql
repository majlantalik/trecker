--liquibase formatted sql
--changeset trecker:005-create-refresh-tokens-table
CREATE TABLE refresh_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    revoked    BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT uq_refresh_tokens_token_hash UNIQUE (token_hash)
);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
--rollback DROP INDEX idx_refresh_tokens_token_hash; DROP INDEX idx_refresh_tokens_user_id; DROP TABLE refresh_tokens;
