use crate::models::{ChunkType, TextChunk};
use uuid::Uuid;

/// Chunks legislative text into semantic units (clauses, sections, etc.)
pub fn chunk_text(text: &str, bill_number: &str) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let bill_id = Uuid::new_v4();
    
    // Split by chapters and major sections
    let sections = split_into_sections(text);
    
    for (idx, section) in sections.iter().enumerate() {
        let (chunk_type, identifier) = identify_chunk_type(section);
        
        // Only create chunks for non-empty content
        if section.trim().len() > 50 {
            chunks.push(TextChunk {
                bill_id,
                bill_number: bill_number.to_string(),
                chunk_index: idx,
                chunk_type,
                chunk_identifier: identifier,
                content: section.trim().to_string(),
            });
        }
    }
    
    // If no structured chunks found, fall back to simple paragraph chunking
    if chunks.is_empty() {
        chunks = fallback_chunking(text, bill_id, bill_number);
    }
    
    chunks
}

fn split_into_sections(text: &str) -> Vec<String> {
    let mut sections = Vec::new();
    
    // Patterns that indicate section boundaries in Indian legislation
    let section_pattern = regex::Regex::new(
        r"(?m)^(\d+\.|CHAPTER [IVXLCDM]+|PREAMBLE|SCHEDULE|Short title)"
    ).unwrap();
    
    let matches: Vec<_> = section_pattern.find_iter(text).collect();
    
    // Add a sentinel at the end
    if !matches.is_empty() {
        for window in matches.windows(2) {
            let start = window[0].start();
            let end = window[1].start();
            sections.push(text[start..end].to_string());
        }
        
        // Add the last section
        if let Some(last_match) = matches.last() {
            sections.push(text[last_match.start()..].to_string());
        }
    }
    
    // If no matches found, split by double newlines
    if sections.is_empty() {
        sections = text
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();
    }
    
    sections
}

fn identify_chunk_type(section: &str) -> (ChunkType, String) {
    let section_lower = section.to_lowercase();
    let first_line = section.lines().next().unwrap_or("").trim();
    
    // Identify by content patterns
    if section_lower.contains("be it enacted") || section_lower.contains("preamble") {
        return (ChunkType::Preamble, "Preamble".to_string());
    }
    
    if let Some(chapter_match) = regex::Regex::new(r"CHAPTER ([IVXLCDM]+)")
        .unwrap()
        .captures(first_line)
    {
        let chapter = chapter_match.get(1).unwrap().as_str();
        return (ChunkType::Section, format!("Chapter {}", chapter));
    }
    
    if let Some(clause_match) = regex::Regex::new(r"^(\d+)\.")
        .unwrap()
        .captures(first_line)
    {
        let number = clause_match.get(1).unwrap().as_str();
        return (ChunkType::Clause, format!("Clause {}", number));
    }
    
    if section_lower.contains("schedule") {
        return (ChunkType::Schedule, "Schedule".to_string());
    }
    
    // Try to extract a descriptive identifier from the first line
    let identifier = if first_line.len() > 5 && first_line.len() < 100 {
        first_line.to_string()
    } else {
        // Use first few words
        first_line
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ")
    };
    
    (ChunkType::Other, identifier)
}

fn fallback_chunking(text: &str, bill_id: Uuid, bill_number: &str) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .filter(|p| p.trim().len() > 100)
        .collect();
    
    // Combine small paragraphs into larger chunks (aim for 200-500 words)
    let mut current_chunk = String::new();
    let mut chunk_index = 0;
    
    for para in paragraphs {
        if current_chunk.split_whitespace().count() + para.split_whitespace().count() > 500 {
            // Save current chunk
            if !current_chunk.is_empty() {
                let identifier = extract_identifier(&current_chunk, chunk_index);
                chunks.push(TextChunk {
                    bill_id,
                    bill_number: bill_number.to_string(),
                    chunk_index,
                    chunk_type: ChunkType::Other,
                    chunk_identifier: identifier,
                    content: current_chunk.trim().to_string(),
                });
                chunk_index += 1;
            }
            current_chunk = para.to_string();
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        }
    }
    
    // Add the last chunk
    if !current_chunk.is_empty() {
        let identifier = extract_identifier(&current_chunk, chunk_index);
        chunks.push(TextChunk {
            bill_id,
            bill_number: bill_number.to_string(),
            chunk_index,
            chunk_type: ChunkType::Other,
            chunk_identifier: identifier,
            content: current_chunk.trim().to_string(),
        });
    }
    
    chunks
}

fn extract_identifier(chunk: &str, index: usize) -> String {
    // Try to get a meaningful identifier from the chunk
    let first_line = chunk.lines().next().unwrap_or("");
    
    if first_line.len() > 10 && first_line.len() < 100 {
        first_line.trim().to_string()
    } else {
        format!("Section {}", index + 1)
    }
}

