-- Migration: Persist Heimdall auth audit events in an append-only table.

CREATE TABLE IF NOT EXISTS nyx.auth_audit_events (
    id BIGSERIAL PRIMARY KEY,
    event TEXT NOT NULL,
    identity_id TEXT,
    client_ip TEXT NOT NULL,
    path TEXT NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_audit_events_created_at
    ON nyx.auth_audit_events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_audit_events_identity_id
    ON nyx.auth_audit_events(identity_id);

COMMENT ON TABLE nyx.auth_audit_events IS 'Append-only auth audit trail emitted by Heimdall';