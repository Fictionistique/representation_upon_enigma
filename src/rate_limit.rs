use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

const MAX_POSTS_PER_HOUR: i64 = 5;

/// Check if user can create a new post (rate limiting)
pub async fn can_create_post(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    let one_hour_ago = Utc::now() - Duration::hours(1);
    
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM rate_limits
        WHERE user_id = $1 AND action_type = 'post_create' AND timestamp > $2
        "#,
    )
    .bind(user_id)
    .bind(one_hour_ago)
    .fetch_one(pool)
    .await
    .context("Failed to check rate limit")?;

    Ok(count.0 < MAX_POSTS_PER_HOUR)
}

/// Get remaining posts allowed for user this hour
pub async fn get_remaining_posts(pool: &PgPool, user_id: Uuid) -> Result<i64> {
    let one_hour_ago = Utc::now() - Duration::hours(1);
    
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM rate_limits
        WHERE user_id = $1 AND action_type = 'post_create' AND timestamp > $2
        "#,
    )
    .bind(user_id)
    .bind(one_hour_ago)
    .fetch_one(pool)
    .await
    .context("Failed to get rate limit count")?;

    Ok(MAX_POSTS_PER_HOUR - count.0)
}

/// Record a post action for rate limiting
pub async fn record_post_action(pool: &PgPool, user_id: Uuid) -> Result<()> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query(
        r#"
        INSERT INTO rate_limits (id, user_id, action_type, timestamp)
        VALUES ($1, $2, 'post_create', $3)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(now)
    .execute(pool)
    .await
    .context("Failed to record rate limit action")?;

    Ok(())
}

/// Clean up old rate limit records (older than 1 day)
#[allow(dead_code)]
pub async fn cleanup_old_records(pool: &PgPool) -> Result<u64> {
    let one_day_ago = Utc::now() - Duration::days(1);
    
    let result = sqlx::query("DELETE FROM rate_limits WHERE timestamp < $1")
        .bind(one_day_ago)
        .execute(pool)
        .await
        .context("Failed to cleanup old rate limit records")?;

    Ok(result.rows_affected())
}

/// Get time until next post is allowed (returns None if user can post now)
#[allow(dead_code)]
pub async fn get_time_until_next_post(pool: &PgPool, user_id: Uuid) -> Result<Option<i64>> {
    if can_create_post(pool, user_id).await? {
        return Ok(None);
    }

    // Get the oldest rate limit record within the last hour
    let one_hour_ago = Utc::now() - Duration::hours(1);
    
    let oldest: Option<(chrono::DateTime<Utc>,)> = sqlx::query_as(
        r#"
        SELECT timestamp FROM rate_limits
        WHERE user_id = $1 AND action_type = 'post_create' AND timestamp > $2
        ORDER BY timestamp ASC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(one_hour_ago)
    .fetch_optional(pool)
    .await
    .context("Failed to get oldest rate limit record")?;

    match oldest {
        Some((timestamp,)) => {
            // Time until the oldest record expires (1 hour from its creation)
            let expires_at = timestamp + Duration::hours(1);
            let seconds_remaining = (expires_at - Utc::now()).num_seconds();
            Ok(Some(seconds_remaining.max(0)))
        }
        None => Ok(None),
    }
}

