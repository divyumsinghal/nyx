CREATE TABLE IF NOT EXISTS "Uzume".profiles (
    id UUID PRIMARY KEY,
    nyx_identity_id UUID NOT NULL,
    app TEXT NOT NULL DEFAULT 'uzume',
    alias TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    bio TEXT NOT NULL DEFAULT '',
    is_private BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uzume_profiles_app_ck CHECK (app = 'uzume'),
    CONSTRAINT uzume_profiles_alias_uk UNIQUE (alias),
    CONSTRAINT uzume_profiles_identity_app_uk UNIQUE (nyx_identity_id, app),
    CONSTRAINT uzume_profiles_alias_fk FOREIGN KEY (nyx_identity_id, app, alias)
        REFERENCES nyx.app_aliases (nyx_identity_id, app, alias)
        ON DELETE CASCADE
);
