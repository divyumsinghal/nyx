CREATE TABLE IF NOT EXISTS nyx.app_aliases (
    nyx_identity_id UUID NOT NULL,
    app TEXT NOT NULL,
    alias TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT app_aliases_pk PRIMARY KEY (nyx_identity_id, app),
    CONSTRAINT app_aliases_app_ck CHECK (app IN ('uzume', 'anteros', 'themis')),
    CONSTRAINT app_aliases_app_alias_uk UNIQUE (app, alias),
    CONSTRAINT app_aliases_identity_app_alias_uk UNIQUE (nyx_identity_id, app, alias)
);
