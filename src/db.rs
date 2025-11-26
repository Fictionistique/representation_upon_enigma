use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Bill, DbBill, Post, PostWithUser, UserProfile};

/// Create database connection pool
pub async fn create_pool() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://civic_user:civic_pass@localhost/civic_legislation".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

/// Get all bills with pagination
pub async fn get_bills_paginated(pool: &PgPool, page: i64, per_page: i64) -> Result<(Vec<DbBill>, i64)> {
    let offset = (page - 1) * per_page;
    
    let bills = sqlx::query_as::<_, DbBill>(
        r#"
        SELECT * FROM bills 
        ORDER BY created_at DESC 
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await
    .context("Failed to fetch bills")?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bills")
        .fetch_one(pool)
        .await
        .context("Failed to count bills")?;

    Ok((bills, total.0))
}

/// Get bill by ID
pub async fn get_bill_by_id(pool: &PgPool, bill_id: Uuid) -> Result<Option<DbBill>> {
    let bill = sqlx::query_as::<_, DbBill>("SELECT * FROM bills WHERE id = $1")
        .bind(bill_id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch bill")?;

    Ok(bill)
}

/// Get bill by bill number
pub async fn get_bill_by_number(pool: &PgPool, bill_number: &str) -> Result<Option<DbBill>> {
    let bill = sqlx::query_as::<_, DbBill>("SELECT * FROM bills WHERE bill_number = $1")
        .bind(bill_number)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch bill")?;

    Ok(bill)
}

/// Insert a bill into the database
pub async fn insert_bill(pool: &PgPool, bill: &Bill) -> Result<DbBill> {
    let now = Utc::now();
    
    let db_bill = sqlx::query_as::<_, DbBill>(
        r#"
        INSERT INTO bills (id, title, bill_number, year, session, status, pdf_url, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (bill_number) DO UPDATE SET
            title = EXCLUDED.title,
            year = EXCLUDED.year,
            session = EXCLUDED.session,
            status = EXCLUDED.status,
            pdf_url = EXCLUDED.pdf_url,
            updated_at = EXCLUDED.updated_at
        RETURNING *
        "#,
    )
    .bind(bill.id)
    .bind(&bill.title)
    .bind(&bill.bill_number)
    .bind(bill.year)
    .bind(&bill.session)
    .bind(&bill.status)
    .bind(&bill.pdf_url)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to insert bill")?;

    Ok(db_bill)
}

/// Get posts for a bill with user info
pub async fn get_posts_for_bill(pool: &PgPool, bill_id: Uuid) -> Result<Vec<PostWithUser>> {
    let rows = sqlx::query_as::<_, (Uuid, String, Option<String>, String, String, i32, i32, chrono::DateTime<Utc>)>(
        r#"
        SELECT 
            p.id, 
            u.username, 
            c.name as constituency_name,
            p.stance, 
            p.content, 
            p.upvotes, 
            p.downvotes, 
            p.created_at
        FROM posts p
        INNER JOIN users u ON p.user_id = u.id
        LEFT JOIN constituencies c ON u.constituency_id = c.id
        WHERE p.bill_id = $1 AND p.moderation_status = 'approved'
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(bill_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch posts")?;

    let posts = rows
        .into_iter()
        .map(|(id, username, constituency_name, stance, content, upvotes, downvotes, created_at)| {
            PostWithUser {
                id,
                username,
                constituency_name,
                stance,
                content,
                upvotes,
                downvotes,
                created_at,
                formatted_date: created_at.format("%B %d, %Y").to_string(),
            }
        })
        .collect();

    Ok(posts)
}

/// Create a new post
pub async fn create_post(
    pool: &PgPool,
    user_id: Uuid,
    bill_id: Uuid,
    stance: &str,
    content: &str,
    moderation_status: &str,
    moderation_reason: Option<&str>,
) -> Result<Post> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let post = sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (id, user_id, bill_id, stance, content, moderation_status, moderation_reason, upvotes, downvotes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0, $8, $9)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(bill_id)
    .bind(stance)
    .bind(content)
    .bind(moderation_status)
    .bind(moderation_reason)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to create post")?;

    Ok(post)
}

/// Get posts by user ID
pub async fn get_posts_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<(Post, String, String)>> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid, Uuid, String, String, String, Option<String>, i32, i32, chrono::DateTime<Utc>, chrono::DateTime<Utc>, String, String)>(
        r#"
        SELECT 
            p.id, p.user_id, p.bill_id, p.stance, p.content, p.moderation_status, p.moderation_reason,
            p.upvotes, p.downvotes, p.created_at, p.updated_at,
            b.title as bill_title, b.bill_number
        FROM posts p
        INNER JOIN bills b ON p.bill_id = b.id
        WHERE p.user_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch user posts")?;

    let posts = rows
        .into_iter()
        .map(|(id, user_id, bill_id, stance, content, moderation_status, moderation_reason, upvotes, downvotes, created_at, updated_at, bill_title, bill_number)| {
            (
                Post {
                    id,
                    user_id,
                    bill_id,
                    stance,
                    content,
                    moderation_status,
                    moderation_reason,
                    upvotes,
                    downvotes,
                    created_at,
                    updated_at,
                },
                bill_title,
                bill_number,
            )
        })
        .collect();

    Ok(posts)
}

/// Upvote a post - returns (upvotes, downvotes, user_vote_type)
pub async fn upvote_post(pool: &PgPool, post_id: Uuid, user_id: Uuid) -> Result<(i32, i32, Option<String>)> {
    // Check if user already voted
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT vote_type FROM post_votes WHERE post_id = $1 AND user_id = $2"
    )
    .bind(post_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let new_vote_type: Option<String> = match existing {
        Some((vote_type,)) if vote_type == "upvote" => {
            // Already upvoted, remove vote
            sqlx::query("DELETE FROM post_votes WHERE post_id = $1 AND user_id = $2")
                .bind(post_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE posts SET upvotes = upvotes - 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            None // Vote removed
        }
        Some((vote_type,)) if vote_type == "downvote" => {
            // Was downvote, switch to upvote
            sqlx::query("UPDATE post_votes SET vote_type = 'upvote' WHERE post_id = $1 AND user_id = $2")
                .bind(post_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE posts SET upvotes = upvotes + 1, downvotes = downvotes - 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            Some("upvote".to_string())
        }
        _ => {
            // No existing vote, add upvote
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO post_votes (id, post_id, user_id, vote_type) VALUES ($1, $2, $3, 'upvote')"
            )
            .bind(id)
            .bind(post_id)
            .bind(user_id)
            .execute(pool)
            .await?;
            sqlx::query("UPDATE posts SET upvotes = upvotes + 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            Some("upvote".to_string())
        }
    };

    // Get updated counts
    let counts: (i32, i32) = sqlx::query_as(
        "SELECT upvotes, downvotes FROM posts WHERE id = $1"
    )
    .bind(post_id)
    .fetch_one(pool)
    .await?;

    Ok((counts.0, counts.1, new_vote_type))
}

/// Downvote a post - returns (upvotes, downvotes, user_vote_type)
pub async fn downvote_post(pool: &PgPool, post_id: Uuid, user_id: Uuid) -> Result<(i32, i32, Option<String>)> {
    // Check if user already voted
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT vote_type FROM post_votes WHERE post_id = $1 AND user_id = $2"
    )
    .bind(post_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let new_vote_type: Option<String> = match existing {
        Some((vote_type,)) if vote_type == "downvote" => {
            // Already downvoted, remove vote
            sqlx::query("DELETE FROM post_votes WHERE post_id = $1 AND user_id = $2")
                .bind(post_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE posts SET downvotes = downvotes - 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            None // Vote removed
        }
        Some((vote_type,)) if vote_type == "upvote" => {
            // Was upvote, switch to downvote
            sqlx::query("UPDATE post_votes SET vote_type = 'downvote' WHERE post_id = $1 AND user_id = $2")
                .bind(post_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE posts SET downvotes = downvotes + 1, upvotes = upvotes - 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            Some("downvote".to_string())
        }
        _ => {
            // No existing vote, add downvote
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO post_votes (id, post_id, user_id, vote_type) VALUES ($1, $2, $3, 'downvote')"
            )
            .bind(id)
            .bind(post_id)
            .bind(user_id)
            .execute(pool)
            .await?;
            sqlx::query("UPDATE posts SET downvotes = downvotes + 1 WHERE id = $1")
                .bind(post_id)
                .execute(pool)
                .await?;
            Some("downvote".to_string())
        }
    };

    // Get updated counts
    let counts: (i32, i32) = sqlx::query_as(
        "SELECT upvotes, downvotes FROM posts WHERE id = $1"
    )
    .bind(post_id)
    .fetch_one(pool)
    .await?;

    Ok((counts.0, counts.1, new_vote_type))
}

/// Get user profile with post count
pub async fn get_user_profile(pool: &PgPool, username: &str) -> Result<Option<UserProfile>> {
    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<i32>, Option<String>, Option<String>, Option<String>, chrono::DateTime<Utc>, i64)>(
        r#"
        SELECT 
            u.id, u.username, u.real_name, u.age, u.gender, u.pincode,
            c.name as constituency_name,
            u.created_at,
            (SELECT COUNT(*) FROM posts WHERE user_id = u.id) as post_count
        FROM users u
        LEFT JOIN constituencies c ON u.constituency_id = c.id
        WHERE u.username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch user profile")?;

    let profile = row.map(|(id, username, real_name, age, gender, pincode, constituency_name, created_at, post_count)| {
        UserProfile {
            id,
            username,
            real_name,
            age,
            gender,
            pincode,
            constituency_name,
            created_at,
            post_count,
        }
    });

    Ok(profile)
}

