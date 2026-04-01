//! Raw SQL queries for engagement windows used by the trending algorithm.

use sqlx::PgPool;
use uuid::Uuid;

use nun::NyxError;

/// A post's engagement metrics within a rolling time window.
#[derive(Debug, Clone)]
pub struct PostEngagement {
    /// UUID of the post.
    pub post_id: Uuid,
    /// Denormalized like count.
    pub like_count: i64,
    /// Denormalized comment count.
    pub comment_count: i64,
    /// Denormalized save count.
    pub save_count: i64,
    /// Hashtags attached to this post.
    pub hashtags: Vec<String>,
}

/// Fetch posts with engagement activity within the last `hours` hours.
///
/// Returns posts ordered by a combined engagement score:
/// `likes * 3 + comments * 5 + saves * 2`.
pub async fn get_post_engagement_window(
    pool: &PgPool,
    hours: i32,
) -> Result<Vec<PostEngagement>, NyxError> {
    let rows: Vec<(Uuid, i64, i64, i64, Vec<String>)> = sqlx::query_as(
        r#"
        SELECT
            p.id,
            p.like_count,
            p.comment_count,
            p.save_count,
            p.hashtags
        FROM "Uzume".posts p
        WHERE p.created_at > NOW() - ($1 * INTERVAL '1 hour')
          AND (p.like_count + p.comment_count + p.save_count) > 0
        ORDER BY (p.like_count * 3 + p.comment_count * 5 + p.save_count * 2) DESC
        LIMIT 500
        "#,
    )
    .bind(hours)
    .fetch_all(pool)
    .await
    .map_err(NyxError::from)?;

    Ok(rows
        .into_iter()
        .map(
            |(post_id, like_count, comment_count, save_count, hashtags)| PostEngagement {
                post_id,
                like_count,
                comment_count,
                save_count,
                hashtags,
            },
        )
        .collect())
}

/// Fetch hashtag usage counts from posts created within the last `hours` hours.
///
/// Returns `(hashtag, usage_count)` pairs sorted by count descending.
pub async fn get_hashtag_counts_window(
    pool: &PgPool,
    hours: i32,
) -> Result<Vec<(String, i64)>, NyxError> {
    sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT
            tag AS hashtag,
            COUNT(*) AS usage_count
        FROM "Uzume".posts p,
             UNNEST(p.hashtags) AS tag
        WHERE p.created_at > NOW() - ($1 * INTERVAL '1 hour')
          AND tag <> ''
        GROUP BY tag
        ORDER BY usage_count DESC
        LIMIT 100
        "#,
    )
    .bind(hours)
    .fetch_all(pool)
    .await
    .map_err(NyxError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_engagement_fields_accessible() {
        let e = PostEngagement {
            post_id: Uuid::nil(),
            like_count: 10,
            comment_count: 5,
            save_count: 2,
            hashtags: vec!["rust".to_string()],
        };
        assert_eq!(e.like_count, 10);
        assert_eq!(e.hashtags.len(), 1);
    }
}
