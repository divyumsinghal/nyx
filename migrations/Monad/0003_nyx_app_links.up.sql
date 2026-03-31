CREATE TABLE IF NOT EXISTS nyx.app_links (
    source_nyx_identity_id UUID NOT NULL,
    source_app TEXT NOT NULL,
    target_nyx_identity_id UUID NOT NULL,
    target_app TEXT NOT NULL,
    policy JSONB NOT NULL DEFAULT '{"type":"revoked"}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT app_links_pk PRIMARY KEY (
        source_nyx_identity_id,
        source_app,
        target_nyx_identity_id,
        target_app
    ),
    CONSTRAINT app_links_source_app_ck CHECK (source_app IN ('uzume', 'anteros', 'themis')),
    CONSTRAINT app_links_target_app_ck CHECK (target_app IN ('uzume', 'anteros', 'themis')),
    CONSTRAINT app_links_cross_app_ck CHECK (source_app <> target_app),
    CONSTRAINT app_links_not_self_ck CHECK (source_nyx_identity_id <> target_nyx_identity_id),
    CONSTRAINT app_links_source_fk FOREIGN KEY (source_nyx_identity_id, source_app)
        REFERENCES nyx.app_aliases (nyx_identity_id, app)
        ON DELETE CASCADE,
    CONSTRAINT app_links_target_fk FOREIGN KEY (target_nyx_identity_id, target_app)
        REFERENCES nyx.app_aliases (nyx_identity_id, app)
        ON DELETE CASCADE
);
