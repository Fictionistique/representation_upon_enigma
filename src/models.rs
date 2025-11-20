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

