-- Migration: Track revoked JWTs so compromised sessions can be rejected.

CREATE TABLE IF NOT EXISTS nyx.revoked_jwts (
    jti TEXT PRIMARY KEY,
    subject UUID NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_revoked_jwts_expires_at
    ON nyx.revoked_jwts(expires_at);

COMMENT ON TABLE nyx.revoked_jwts IS 'Revoked JWT IDs rejected by Heimdall auth middleware';