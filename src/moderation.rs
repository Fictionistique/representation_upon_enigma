use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::models::ModerationResult;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Check content for toxicity, hate speech, threats, or spam using Ollama
/// Returns ModerationResult based on the analysis
pub async fn check_content(content: &str) -> Result<ModerationResult> {
    let prompt = format!(
        r#"You are a content moderator. Analyze the following user post for a civic legislation forum and determine if it contains:
1. Hate speech or discriminatory language
2. Threats
3. Toxic language
4. Advertisements and similar spam

Post to analyze:
"{}"

Respond with ONLY one of these three words:
- "Falafel" if the content is appropriate for the forum
- "Popcorn" if the content clearly violates the guidelines (hate speech, threats, spam)
- "REVIEW" if you're unsure or the content is borderline

Response:"#,
        content
    );

    // Try to connect to Ollama
    let client = reqwest::Client::new();
    let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    
    let request = OllamaRequest {
        model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
        prompt,
        stream: false,
    };

    info!("Sending moderation request to Ollama at {}", ollama_url);

    match client
        .post(format!("{}/api/generate", ollama_url))
        .json(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<OllamaResponse>().await {
                    Ok(ollama_response) => {
                        let response_text = ollama_response.response.trim().to_uppercase();
                        info!("Ollama moderation response: {}", response_text);
                        
                        // Switch case logic as requested
                        let result = match response_text.as_str() {
                            s if s.contains("Falafel") => {
                                info!("Falafel - Post is approved");
                                ModerationResult::Falafel
                            }
                            s if s.contains("Popcorn") => {
                                info!("Popcorn - Post is rejected");
                                ModerationResult::Popcorn
                            }
                            _ => {
                                info!("Default - Post marked for admin review");
                                ModerationResult::AdminReview
                            }
                        };
                        
                        return Ok(result);
                    }
                    Err(e) => {
                        warn!("Failed to parse Ollama response: {}", e);
                    }
                }
            } else {
                warn!("Ollama returned error status: {}", response.status());
            }
        }
        Err(e) => {
            warn!("Failed to connect to Ollama: {}", e);
        }
    }

    // Fallback: If Ollama is not available, use simple keyword-based filtering
    info!("Using fallback keyword-based moderation");
    Ok(fallback_moderation(content))
}

/// Simple keyword-based fallback moderation when LLM is unavailable
fn fallback_moderation(content: &str) -> ModerationResult {
    let content_lower = content.to_lowercase();
    
    // List of obviously toxic patterns
    let toxic_patterns = [
        "kill", "murder", "hate", "terrorist", "bomb", "die",
        "stupid", "idiot", "moron", "racist", "sexist",
    ];
    
    // List of spam patterns
    let spam_patterns = [
        "buy now", "click here", "free money", "lottery",
        "crypto", "bitcoin", "investment opportunity",
    ];
    
    // Check for toxic content
    for pattern in toxic_patterns {
        if content_lower.contains(pattern) {
            info!("Fallback moderation: Found toxic pattern '{}', marking for review", pattern);
            return ModerationResult::AdminReview;
        }
    }
    
    // Check for spam content
    for pattern in spam_patterns {
        if content_lower.contains(pattern) {
            info!("Fallback moderation: Found spam pattern '{}', rejecting", pattern);
            return ModerationResult::Popcorn;
        }
    }
    
    // If no concerning patterns found, approve
    info!("Fallback moderation: No issues found, approving");
    ModerationResult::Falafel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_moderation_safe() {
        let result = fallback_moderation("This is a thoughtful comment about the legislation.");
        assert_eq!(result, ModerationResult::Falafel);
    }

    #[test]
    fn test_fallback_moderation_spam() {
        let result = fallback_moderation("Click here for free money!");
        assert_eq!(result, ModerationResult::Popcorn);
    }

    #[test]
    fn test_fallback_moderation_toxic() {
        let result = fallback_moderation("This is a hateful message");
        assert_eq!(result, ModerationResult::AdminReview);
    }
}

