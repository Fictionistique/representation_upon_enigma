use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    pub id: Uuid,
    pub title: String,
    pub bill_number: String,
    pub year: i32,
    pub session: Option<String>,
    pub status: Option<String>,
    pub introduction_date: Option<String>,
    pub pdf_url: String,
}

impl Bill {
    pub fn new(
        title: String,
        bill_number: String,
        year: i32,
        pdf_url: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            bill_number,
            year,
            session: None,
            status: None,
            introduction_date: None,
            pdf_url,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    #[allow(dead_code)]
    pub bill_id: Uuid,
    #[allow(dead_code)]
    pub bill_number: String,
    pub chunk_index: usize,
    pub chunk_type: ChunkType,
    pub chunk_identifier: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Preamble,
    Clause,
    Section,
    Schedule,
    Other,
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkType::Preamble => write!(f, "Preamble"),
            ChunkType::Clause => write!(f, "Clause"),
            ChunkType::Section => write!(f, "Section"),
            ChunkType::Schedule => write!(f, "Schedule"),
            ChunkType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddedChunk {
    pub chunk: TextChunk,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub bill_title: String,
    #[allow(dead_code)]
    pub bill_number: String,
    pub chunk_identifier: String,
    pub content: String,
    pub score: f32,
}

// User model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub real_name: Option<String>,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub pincode: Option<String>,
    pub constituency_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Constituency model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Constituency {
    pub id: i32,
    pub name: String,
    pub state: String,
    pub code: String,
}

// Session model
#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Post/Review model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bill_id: Uuid,
    pub stance: String,
    pub content: String,
    pub moderation_status: String,
    pub moderation_reason: Option<String>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Moderation result enum
#[derive(Debug, Clone, PartialEq)]
pub enum ModerationResult {
    Falafel,      // Post is fine, approve
    Popcorn,      // Post needs to be rejected
    AdminReview,  // Needs admin approval (default case)
}

impl ModerationResult {
    pub fn to_status(&self) -> &'static str {
        match self {
            ModerationResult::Falafel => "approved",
            ModerationResult::Popcorn => "rejected",
            ModerationResult::AdminReview => "pending_review",
        }
    }
}

// For displaying posts with user info
#[derive(Debug, Clone, Serialize)]
pub struct PostWithUser {
    pub id: Uuid,
    pub username: String,
    pub constituency_name: Option<String>,
    pub stance: String,
    pub content: String,
    pub upvotes: i32,
    pub downvotes: i32,
    pub created_at: DateTime<Utc>,
    pub formatted_date: String,
}

// Database bill model (with timestamps)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbBill {
    pub id: Uuid,
    pub title: String,
    pub bill_number: String,
    pub year: i32,
    pub session: Option<String>,
    pub status: Option<String>,
    pub introduction_date: Option<chrono::NaiveDate>,
    pub pdf_url: Option<String>,
    pub extracted_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User profile view
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub real_name: Option<String>,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub pincode: Option<String>,
    pub constituency_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub post_count: i64,
}
