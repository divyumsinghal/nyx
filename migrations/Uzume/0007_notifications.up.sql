-- =============================================================================
-- 0007_notifications.up.sql
-- In-app notification store.
--
-- Design notes:
--   GROUPING:
--     Multiple similar events collapse into one notification row.
--     e.g., "Alice, Bob, and 3 others liked your post" is one row with
--     group_count = 5, summary pre-rendered by Ushas worker.
--     group_key identifies the "bucket": e.g. "post_liked:{post_id}".
--     When a new event arrives for the same group_key, Ushas UPSERTs the row:
--       - increments group_count
--       - re-renders summary
--       - resets is_read = FALSE (re-surfaces the notification)
--
--   DELIVERY:
--     Ushas worker creates a row here AND dispatches a push via Gorush.
--     WebSocket push to the active browser/app session is handled by
--     Ushas in-app.rs (reads the NATS event and pushes over WS).
--
--   PRIVACY:
--     actor_profile_id is nullable (system notifications have no actor).
--     actor details (avatar, alias) are fetched at read time, not stored,
--     to avoid PII duplication and to reflect account deletions.
--
--   RETENTION:
--     Notifications older than 90 days are purged by Ushas cleanup worker.
-- =============================================================================

CREATE TABLE IF NOT EXISTS "Uzume".notifications (
    id                      UUID        NOT NULL DEFAULT gen_random_uuid(),
    recipient_profile_id    UUID        NOT NULL,
    -- The user whose action triggered the notification. NULL for system notifs.
    actor_profile_id        UUID,
    notification_type       TEXT        NOT NULL,

    -- The entity the notification is about (a post, reel, comment, profile…)
    entity_type             TEXT,
    entity_id               UUID,

    -- Pre-rendered summary string built by Ushas.
    -- e.g. "alice and 3 others liked your photo"
    summary                 TEXT        NOT NULL,

    -- Grouping: multiple same-type events on the same entity collapse here.
    -- group_key format: "{notification_type}:{entity_id}"
    -- NULL for ungrouped (system notifications, follow requests).
    group_key               TEXT,
    group_count             INTEGER     NOT NULL DEFAULT 1,

    is_read                 BOOLEAN     NOT NULL DEFAULT FALSE,
    read_at                 TIMESTAMPTZ,

    -- True when this notification has been sent as a push notification.
    push_sent               BOOLEAN     NOT NULL DEFAULT FALSE,
    push_sent_at            TIMESTAMPTZ,

    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_notifications_pk
        PRIMARY KEY (id),

    CONSTRAINT uzume_notifications_recipient_fk
        FOREIGN KEY (recipient_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_notifications_actor_fk
        FOREIGN KEY (actor_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE SET NULL,

    CONSTRAINT uzume_notifications_type_ck
        CHECK (notification_type IN (
            'post_liked',
            'post_commented',
            'post_saved',
            'reel_liked',
            'reel_commented',
            'story_viewed',
            'story_reacted',
            'user_followed',
            'follow_request',
            'follow_accepted',
            'mention_post',
            'mention_comment',
            'system'
        )),

    CONSTRAINT uzume_notifications_entity_type_ck
        CHECK (entity_type IS NULL OR entity_type IN ('post', 'reel', 'story', 'comment', 'profile')),

    CONSTRAINT uzume_notifications_group_count_ck
        CHECK (group_count >= 1),

    -- If read_at is set, is_read must be TRUE.
    CONSTRAINT uzume_notifications_read_consistency
        CHECK (read_at IS NULL OR is_read = TRUE),

    -- group_key unique per recipient: only one grouped row per bucket.
    CONSTRAINT uzume_notifications_group_key_unique
        UNIQUE NULLS NOT DISTINCT (recipient_profile_id, group_key)
);

-- Bell icon query: "N unread notifications for profile P"
CREATE INDEX IF NOT EXISTS uzume_notifications_recipient_unread_idx
    ON "Uzume".notifications (recipient_profile_id, created_at DESC)
    WHERE is_read = FALSE;

-- Notification list (read + unread, newest first)
CREATE INDEX IF NOT EXISTS uzume_notifications_recipient_all_idx
    ON "Uzume".notifications (recipient_profile_id, created_at DESC);

-- Ushas upsert lookup: "find the existing group row for this key"
CREATE INDEX IF NOT EXISTS uzume_notifications_group_key_idx
    ON "Uzume".notifications (group_key)
    WHERE group_key IS NOT NULL;

-- Push retry / audit: unsent push notifications
CREATE INDEX IF NOT EXISTS uzume_notifications_push_pending_idx
    ON "Uzume".notifications (created_at)
    WHERE push_sent = FALSE;
