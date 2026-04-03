-- Migration: Track known client device fingerprints for auth alerts.

CREATE TABLE IF NOT EXISTS nyx.known_devices (
    identity_id UUID NOT NULL,
    device_fingerprint TEXT NOT NULL,
    user_agent TEXT,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, device_fingerprint)
);

CREATE INDEX IF NOT EXISTS idx_known_devices_identity_id
    ON nyx.known_devices(identity_id);