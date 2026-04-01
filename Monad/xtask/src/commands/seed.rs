//! Seed command — loads development fixture data from JSON files into the database.
//!
//! Insert order respects FK dependencies:
//! 1. `nyx.app_aliases` (FK parent)
//! 2. `"Uzume".profiles` (references `nyx.app_aliases`)
//! 3. `"Uzume".posts` (references `"Uzume".profiles`)
//!
//! All inserts use `ON CONFLICT DO NOTHING` so the command is idempotent.
#![warn(clippy::pedantic)]

/// Deserialised representation of a single entry in `tools/seed-data/users.json`.
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct SeedUser {
    /// Kratos identity UUID (`UUIDv7` string).
    pub nyx_identity_id: String,
    /// Primary email address.
    pub email: String,
    /// E.164 phone number.
    pub phone: String,
    /// App-scoped alias (unique within the `uzume` app).
    pub username: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Short profile biography.
    pub bio: String,
    /// Whether the profile is private by default.
    pub is_private: bool,
    /// Seed string used to generate a deterministic avatar.
    pub avatar_seed: String,
}

/// Deserialised representation of a single entry in `tools/seed-data/uzume_posts.json`.
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct SeedPost {
    /// Deterministic post UUID (`UUIDv7` string).
    pub id: String,
    /// Alias of the author (matches [`SeedUser::username`]).
    pub author_username: String,
    /// Post caption text.
    pub caption: String,
    /// Hashtag list (without leading `#`).
    pub hashtags: Vec<String>,
    /// Optional location label.
    #[serde(default)]
    pub location_name: Option<String>,
    /// Media attachments (type + placeholder filename).
    pub media: Vec<serde_json::Value>,
}

/// Insert all seed data from `seed_dir` into the database at `db_url`.
///
/// Reads `users.json` and `uzume_posts.json` from `seed_dir`. Posts whose
/// `author_username` does not match a seeded profile are skipped with a
/// warning rather than causing an error.
///
/// # Errors
///
/// Returns [`anyhow::Error`] on connection failure, JSON parse error, UUID
/// parse error, or SQL execution error.
pub async fn run(db_url: &str, seed_dir: &std::path::Path) -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(db_url).await?;

    // --- Users ---
    let users_json = std::fs::read_to_string(seed_dir.join("users.json"))?;
    let users: Vec<SeedUser> = serde_json::from_str(&users_json)?;

    // 1. Insert into nyx.app_aliases FIRST (FK parent).
    for user in &users {
        let identity_id = uuid::Uuid::parse_str(&user.nyx_identity_id)?;
        sqlx::query(
            "INSERT INTO nyx.app_aliases (nyx_identity_id, app, alias) \
             VALUES ($1, 'uzume', $2) \
             ON CONFLICT DO NOTHING",
        )
        .bind(identity_id)
        .bind(&user.username)
        .execute(&pool)
        .await?;
    }

    // 2. Insert into "Uzume".profiles.
    for user in &users {
        let identity_id = uuid::Uuid::parse_str(&user.nyx_identity_id)?;
        let profile_id = uuid::Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO "Uzume".profiles
                   (id, nyx_identity_id, app, alias, display_name, bio, is_private)
               VALUES ($1, $2, 'uzume', $3, $4, $5, $6)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(profile_id)
        .bind(identity_id)
        .bind(&user.username)
        .bind(&user.display_name)
        .bind(&user.bio)
        .bind(user.is_private)
        .execute(&pool)
        .await?;
    }

    tracing::info!(count = users.len(), "Seeded users");

    // --- Posts ---
    let posts_json = std::fs::read_to_string(seed_dir.join("uzume_posts.json"))?;
    let posts: Vec<SeedPost> = serde_json::from_str(&posts_json)?;

    let mut post_count: usize = 0;
    for post in &posts {
        let post_id = uuid::Uuid::parse_str(&post.id)?;

        // Resolve author alias → profile id.
        let profile: Option<(uuid::Uuid,)> =
            sqlx::query_as(r#"SELECT id FROM "Uzume".profiles WHERE alias = $1"#)
                .bind(&post.author_username)
                .fetch_optional(&pool)
                .await?;

        let Some((author_profile_id,)) = profile else {
            tracing::warn!(
                post_id = %post.id,
                author = %post.author_username,
                "Skipping post — author not found in profiles"
            );
            continue;
        };

        sqlx::query(
            r#"INSERT INTO "Uzume".posts (id, author_profile_id, caption)
               VALUES ($1, $2, $3)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(post_id)
        .bind(author_profile_id)
        .bind(&post.caption)
        .execute(&pool)
        .await?;

        post_count += 1;
    }

    tracing::info!(count = post_count, "Seeded posts");
    Ok(())
}
