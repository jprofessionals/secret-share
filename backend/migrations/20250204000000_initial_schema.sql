-- Initial schema for secrets table
CREATE TABLE secrets (
    id UUID PRIMARY KEY,
    encrypted_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    max_views INTEGER,
    views INTEGER NOT NULL DEFAULT 0,
    extendable BOOLEAN NOT NULL DEFAULT TRUE,
    failed_attempts INTEGER NOT NULL DEFAULT 0
);

-- Index for cleanup queries
CREATE INDEX idx_secrets_expires_at ON secrets(expires_at);
