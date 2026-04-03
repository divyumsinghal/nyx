-- Migration: Create nyx.identities table for platform-wide identity management
-- This table stores the mapping between Kratos identity UUIDs and Nyx IDs

CREATE TABLE IF NOT EXISTS nyx.identities (
    id UUID PRIMARY KEY,  -- Kratos identity ID (external reference)
    nyx_id TEXT UNIQUE,   -- User-chosen public handle (e.g., "alice_nyx")
    email TEXT,           -- Cached email for quick lookup
    display_name TEXT,    -- User's display name
    is_email_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for email lookups (login with email)
CREATE INDEX IF NOT EXISTS idx_identities_email ON nyx.identities(email) WHERE is_active = TRUE;

-- Index for Nyx ID lookups (login with Nyx ID)
CREATE INDEX IF NOT EXISTS idx_identities_nyx_id ON nyx.identities(nyx_id) WHERE is_active = TRUE;

-- Partial index: only enforce uniqueness on non-null nyx_id
-- (users during registration may not have chosen their ID yet)
CREATE UNIQUE INDEX IF NOT EXISTS idx_identities_nyx_id_unique 
    ON nyx.identities(nyx_id) 
    WHERE nyx_id IS NOT NULL;

-- Comment on table
COMMENT ON TABLE nyx.identities IS 'Platform identity registry linking Kratos UUIDs to Nyx IDs';
COMMENT ON COLUMN nyx.identities.id IS 'Kratos identity UUID (external reference to kratos.identities)';
COMMENT ON COLUMN nyx.identities.nyx_id IS 'User-chosen public handle, unique across platform';
