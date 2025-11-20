use anyhow::{Context, Result};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    CreateCollection, Distance, SearchPoints, VectorParams, VectorsConfig, WithPayloadSelector,
    PointId, PointStruct,
};
use qdrant_client::Payload;
use crate::models::{Bill, EmbeddedChunk, SearchResult};

const COLLECTION_NAME: &str = "legislation_chunks";
const VECTOR_SIZE: u64 = 384; // all-MiniLM-L6-v2 dimension

/// Initialize the Qdrant collection
pub async fn initialize_collection() -> Result<()> {
    let client = get_client()?;
    
    // Check if collection exists
    let collections = client.list_collections().await?;
    let collection_exists = collections
        .collections
        .iter()
        .any(|c| c.name == COLLECTION_NAME);
    
    if collection_exists {
        tracing::info!("Collection '{}' already exists", COLLECTION_NAME);
        
        // Optionally recreate it
        tracing::info!("Deleting existing collection...");
        client.delete_collection(COLLECTION_NAME).await?;
    }
    
    // Create the collection
    tracing::info!("Creating collection '{}'...", COLLECTION_NAME);
    client
        .create_collection(&CreateCollection {
            collection_name: COLLECTION_NAME.to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: VECTOR_SIZE,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })
        .await?;
    
    tracing::info!("Collection created successfully");
    Ok(())
}

/// Store embedded chunks in Qdrant
pub async fn store_chunks(bill: &Bill, chunks: &[EmbeddedChunk]) -> Result<()> {
    let client = get_client()?;
    
    let mut points = Vec::new();
    
    for chunk in chunks {
        let point_id = PointId::from(uuid::Uuid::new_v4().to_string());
        
        let payload: Payload = serde_json::json!({
            "bill_id": bill.id.to_string(),
            "bill_title": bill.title,
            "bill_number": bill.bill_number,
            "year": bill.year,
            "chunk_index": chunk.chunk.chunk_index,
            "chunk_type": chunk.chunk.chunk_type.to_string(),
            "chunk_identifier": chunk.chunk.chunk_identifier,
            "content": chunk.chunk.content,
        })
        .try_into()
        .unwrap();
        
        let point = PointStruct::new(point_id, chunk.embedding.clone(), payload);
        points.push(point);
    }
    
    // Upsert points in batches
    const BATCH_SIZE: usize = 100;
    for batch in points.chunks(BATCH_SIZE) {
        client
            .upsert_points_blocking(COLLECTION_NAME, None, batch.to_vec(), None)
            .await?;
    }
    
    tracing::debug!("Stored {} chunks for bill: {}", chunks.len(), bill.title);
    Ok(())
}

/// Search for similar chunks using a query vector
pub async fn search(query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
    let client = get_client()?;
    
    let search_result = client
        .search_points(&SearchPoints {
            collection_name: COLLECTION_NAME.to_string(),
            vector: query_vector.to_vec(),
            limit: limit as u64,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                ),
            }),
            ..Default::default()
        })
        .await
        .context("Failed to search vectors")?;
    
    let results = search_result
        .result
        .into_iter()
        .filter_map(|scored_point| {
            let payload = scored_point.payload;
            
            Some(SearchResult {
                bill_title: payload.get("bill_title")?.as_str()?.to_string(),
                bill_number: payload.get("bill_number")?.as_str()?.to_string(),
                chunk_identifier: payload.get("chunk_identifier")?.as_str()?.to_string(),
                content: payload.get("content")?.as_str()?.to_string(),
                score: scored_point.score,
            })
        })
        .collect();
    
    Ok(results)
}

fn get_client() -> Result<QdrantClient> {
    let url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    
    QdrantClient::from_url(&url)
        .build()
        .context("Failed to connect to Qdrant. Make sure it's running via Docker.")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Only run when Qdrant is available
    async fn test_connection() {
        let client = get_client().unwrap();
        let health = client.health_check().await;
        assert!(health.is_ok());
    }
}

