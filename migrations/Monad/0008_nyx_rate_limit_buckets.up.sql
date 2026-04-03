-- Migration: Persist Heimdall rate-limit buckets for distributed enforcement.

CREATE TABLE IF NOT EXISTS nyx.rate_limit_buckets (
    bucket_key TEXT PRIMARY KEY,
    tokens INTEGER NOT NULL,
    reset_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_buckets_reset_at
    ON nyx.rate_limit_buckets(reset_at);