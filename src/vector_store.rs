use anyhow::{Context, Result};
use crate::models::{Bill, EmbeddedChunk, SearchResult};
use serde_json::json;

const COLLECTION_NAME: &str = "legislation_chunks";
const VECTOR_SIZE: usize = 384; // all-MiniLM-L6-v2 dimension

fn get_qdrant_url() -> String {
    std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string())
}

/// Initialize the Qdrant collection
pub async fn initialize_collection() -> Result<()> {
    let base_url = get_qdrant_url();
    let client = reqwest::Client::new();
    
    // Check if collection exists
    let collections_url = format!("{}/collections", base_url);
    let response = client
        .get(&collections_url)
        .send()
        .await
        .context("Failed to list collections")?;
    
    let collections: serde_json::Value = response.json().await?;
    let collection_exists = collections["result"]["collections"]
        .as_array()
        .and_then(|arr| {
            arr.iter().any(|c| c["name"] == COLLECTION_NAME).then_some(true)
        })
        .unwrap_or(false);
    
    if collection_exists {
        tracing::info!("Collection '{}' already exists", COLLECTION_NAME);
        
        // Delete existing collection
        tracing::info!("Deleting existing collection...");
        let delete_url = format!("{}/collections/{}", base_url, COLLECTION_NAME);
        client
            .delete(&delete_url)
            .send()
            .await
            .context("Failed to delete collection")?;
    }
    
    // Create the collection
    tracing::info!("Creating collection '{}'...", COLLECTION_NAME);
    let create_url = format!("{}/collections/{}", base_url, COLLECTION_NAME);
    let create_body = json!({
        "vectors": {
            "size": VECTOR_SIZE,
            "distance": "Cosine"
        }
    });
    
    let response = client
        .put(&create_url)
        .json(&create_body)
        .send()
        .await
        .context("Failed to create collection")?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to create collection: {}", error_text);
    }
    
    tracing::info!("Collection created successfully");
    Ok(())
}

/// Store embedded chunks in Qdrant
pub async fn store_chunks(bill: &Bill, chunks: &[EmbeddedChunk]) -> Result<()> {
    let base_url = get_qdrant_url();
    let client = reqwest::Client::new();
    
    let mut points = Vec::new();
    
    for chunk in chunks {
        let point_id = uuid::Uuid::new_v4().to_string();
        
        let point = json!({
            "id": point_id,
            "vector": chunk.embedding,
            "payload": {
                "bill_id": bill.id.to_string(),
                "bill_title": bill.title,
                "bill_number": bill.bill_number,
                "year": bill.year,
                "chunk_index": chunk.chunk.chunk_index,
                "chunk_type": chunk.chunk.chunk_type.to_string(),
                "chunk_identifier": chunk.chunk.chunk_identifier,
                "content": chunk.chunk.content,
            }
        });
        
        points.push(point);
    }
    
    // Upsert points in batches
    const BATCH_SIZE: usize = 100;
    for batch in points.chunks(BATCH_SIZE) {
        let upsert_url = format!("{}/collections/{}/points", base_url, COLLECTION_NAME);
        let upsert_body = json!({
            "points": batch
        });
        
        let response = client
            .put(&upsert_url)
            .json(&upsert_body)
            .send()
            .await
            .context("Failed to upsert points")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to upsert points: {}", error_text);
        }
    }
    
    tracing::debug!("Stored {} chunks for bill: {}", chunks.len(), bill.title);
    Ok(())
}

/// Search for similar chunks using a query vector
pub async fn search(query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
    let base_url = get_qdrant_url();
    let client = reqwest::Client::new();
    
    let search_url = format!("{}/collections/{}/points/search", base_url, COLLECTION_NAME);
    let search_body = json!({
        "vector": query_vector,
        "limit": limit,
        "with_payload": true
    });
    
    let response = client
        .post(&search_url)
        .json(&search_body)
        .send()
        .await
        .context("Failed to search vectors")?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to search: {}", error_text);
    }
    
    let search_result: serde_json::Value = response.json().await?;
    
    let results: Vec<SearchResult> = search_result["result"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            let payload = &item["payload"];
            Some(SearchResult {
                bill_title: payload["bill_title"].as_str()?.to_string(),
                bill_number: payload["bill_number"].as_str()?.to_string(),
                chunk_identifier: payload["chunk_identifier"].as_str()?.to_string(),
                content: payload["content"].as_str()?.to_string(),
                score: item["score"].as_f64()? as f32,
            })
        })
        .collect();
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Only run when Qdrant is available
    async fn test_connection() {
        let base_url = get_qdrant_url();
        let client = reqwest::Client::new();
        let response = client.get(format!("{}/collections", base_url)).send().await;
        assert!(response.is_ok());
    }
}
