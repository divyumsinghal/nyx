-- Migration: Track repeated auth failures for distributed lockout enforcement.

CREATE TABLE IF NOT EXISTS nyx.auth_failures (
    failure_key TEXT PRIMARY KEY,
    failures INTEGER NOT NULL,
    blocked_until TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_failures_blocked_until
    ON nyx.auth_failures(blocked_until);