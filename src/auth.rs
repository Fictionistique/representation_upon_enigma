use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Constituency, Session, User};

// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(password_hash)
}

// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// Generate a session token
pub fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

// Create a new user
pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password: &str,
    real_name: Option<&str>,
    age: Option<i32>,
    gender: Option<&str>,
    pincode: Option<&str>,
    constituency_id: Option<i32>,
) -> Result<User> {
    let password_hash = hash_password(password)?;
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    // If pincode is provided but constituency_id is not, look up constituency
    let final_constituency_id = if constituency_id.is_some() {
        constituency_id
    } else if let Some(pc) = pincode {
        get_constituency_by_pincode(pool, pc).await?.map(|c| c.id)
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password_hash, real_name, age, gender, pincode, constituency_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(username)
    .bind(&password_hash)
    .bind(real_name)
    .bind(age)
    .bind(gender)
    .bind(pincode)
    .bind(final_constituency_id)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to create user")?;

    Ok(user)
}

// Get user by username
pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch user")?;
    Ok(user)
}

// Get user by ID
#[allow(dead_code)]
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch user")?;
    Ok(user)
}

// Authenticate user
pub async fn authenticate_user(pool: &PgPool, username: &str, password: &str) -> Result<Option<User>> {
    let user = get_user_by_username(pool, username).await?;
    
    match user {
        Some(u) => {
            if verify_password(password, &u.password_hash)? {
                Ok(Some(u))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

// Create a session for a user
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<Session> {
    let session_id = Uuid::new_v4();
    let session_token = generate_session_token();
    let expires_at = Utc::now() + Duration::days(7); // Session valid for 7 days
    let now = Utc::now();

    let session = sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (id, user_id, session_token, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(&session_token)
    .bind(expires_at)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to create session")?;

    Ok(session)
}

// Get user by session token
pub async fn get_user_by_session(pool: &PgPool, session_token: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT u.* FROM users u
        INNER JOIN sessions s ON u.id = s.user_id
        WHERE s.session_token = $1 AND s.expires_at > NOW()
        "#,
    )
    .bind(session_token)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch user by session")?;

    Ok(user)
}

// Delete session (logout)
pub async fn delete_session(pool: &PgPool, session_token: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE session_token = $1")
        .bind(session_token)
        .execute(pool)
        .await
        .context("Failed to delete session")?;
    Ok(())
}

// Update user profile
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    real_name: Option<&str>,
    age: Option<i32>,
    gender: Option<&str>,
    pincode: Option<&str>,
    constituency_id: Option<i32>,
) -> Result<User> {
    let now = Utc::now();

    // If pincode is provided but constituency_id is not, look up constituency
    let final_constituency_id = if constituency_id.is_some() {
        constituency_id
    } else if let Some(pc) = pincode {
        get_constituency_by_pincode(pool, pc).await?.map(|c| c.id)
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET real_name = $2, age = $3, gender = $4, pincode = $5, constituency_id = $6, updated_at = $7
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(real_name)
    .bind(age)
    .bind(gender)
    .bind(pincode)
    .bind(final_constituency_id)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to update user profile")?;

    Ok(user)
}

// Get all constituencies
pub async fn get_all_constituencies(pool: &PgPool) -> Result<Vec<Constituency>> {
    let constituencies = sqlx::query_as::<_, Constituency>(
        "SELECT * FROM constituencies ORDER BY state, name"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch constituencies")?;

    Ok(constituencies)
}

// Get constituency by ID
#[allow(dead_code)]
pub async fn get_constituency_by_id(pool: &PgPool, id: i32) -> Result<Option<Constituency>> {
    let constituency = sqlx::query_as::<_, Constituency>(
        "SELECT * FROM constituencies WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch constituency")?;

    Ok(constituency)
}

// Get constituency by pincode
pub async fn get_constituency_by_pincode(pool: &PgPool, pincode: &str) -> Result<Option<Constituency>> {
    let constituency = sqlx::query_as::<_, Constituency>(
        r#"
        SELECT c.* FROM constituencies c
        INNER JOIN pincode_constituencies pc ON c.id = pc.constituency_id
        WHERE pc.pincode = $1
        "#,
    )
    .bind(pincode)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch constituency by pincode")?;

    Ok(constituency)
}

// Check if username exists
pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .context("Failed to check username existence")?;

    Ok(count.0 > 0)
}

