-- =============================================================================
-- 0004_nyx_push_tokens.up.sql
-- Push notification device tokens.
--
-- Design notes:
--   - One row per (app, device). A user may have many devices per app.
--   - device_fingerprint is a stable, non-PII device identifier (e.g. IDFV on
--     iOS, Android ID on Android). Used to deduplicate re-registrations without
--     storing raw device IDs.
--   - platform drives which Gorush channel to use (ios → APNs, android → FCM).
--   - Tokens expire or are replaced; stale tokens are cleaned up by Ushas worker
--     when Gorush returns a 410 Gone or "invalid token" response.
-- =============================================================================

CREATE TABLE IF NOT EXISTS nyx.push_tokens (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    nyx_identity_id     UUID        NOT NULL,
    app                 TEXT        NOT NULL,
    platform            TEXT        NOT NULL,
    token               TEXT        NOT NULL,
    -- A stable, non-PII device identifier used to overwrite stale tokens.
    -- Derive on-device from IDFV (iOS) or Android ID (Android).
    -- NULL for web push (tokens are per-subscription, not per-device).
    device_fingerprint  TEXT,
    -- Ushas flips this to FALSE when Gorush reports an invalid/expired token.
    is_active           BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT push_tokens_pk
        PRIMARY KEY (id),

    CONSTRAINT push_tokens_app_ck
        CHECK (app IN ('uzume', 'anteros', 'themis')),

    CONSTRAINT push_tokens_platform_ck
        CHECK (platform IN ('ios', 'android', 'web')),

    -- A given device (fingerprint) can only have one active token per app.
    -- NULL fingerprints (web push) are excluded from this constraint.
    CONSTRAINT push_tokens_device_unique
        UNIQUE NULLS NOT DISTINCT (app, device_fingerprint),

    -- The token itself must be unique per platform (tokens are globally unique
    -- within APNs/FCM; duplicates indicate a device re-registered).
    CONSTRAINT push_tokens_token_unique
        UNIQUE (platform, token),

    -- Validates that the nyx_identity_id + app combination exists in app_aliases.
    -- This prevents orphaned push tokens for users who have been deleted.
    CONSTRAINT push_tokens_identity_fk
        FOREIGN KEY (nyx_identity_id, app)
        REFERENCES nyx.app_aliases (nyx_identity_id, app)
        ON DELETE CASCADE
);

-- Lookup: "give me all active tokens for user X in app Y"
CREATE INDEX IF NOT EXISTS push_tokens_identity_app_active_idx
    ON nyx.push_tokens (nyx_identity_id, app)
    WHERE is_active = TRUE;

-- Cleanup: "give me all stale tokens older than N days"
CREATE INDEX IF NOT EXISTS push_tokens_updated_at_idx
    ON nyx.push_tokens (updated_at)
    WHERE is_active = FALSE;
