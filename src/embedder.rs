use anyhow::Result;
use crate::models::{EmbeddedChunk, TextChunk};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

const MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";
const EMBEDDING_DIM: usize = 384;

struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

lazy_static::lazy_static! {
    static ref EMBEDDING_MODEL: Arc<Mutex<Option<EmbeddingModel>>> = 
        Arc::new(Mutex::new(None));
}

/// Initialize the embedding model (call this once at startup)
async fn get_or_init_model() -> Result<Arc<Mutex<Option<EmbeddingModel>>>> {
    let mut model_guard = EMBEDDING_MODEL.lock().await;
    
    if model_guard.is_none() {
        tracing::info!("Initializing Candle embedding model (downloading {} on first run)...", MODEL_ID);
        
        // Download model from HuggingFace Hub
        let model_data = tokio::task::spawn_blocking(|| -> Result<EmbeddingModel> {
            let api = Api::new()?;
            let repo = api.repo(Repo::with_revision(
                MODEL_ID.to_string(),
                RepoType::Model,
                "main".to_string(),
            ));
            
            tracing::info!("Downloading model files from HuggingFace...");
            let config_path = repo.get("config.json")?;
            let tokenizer_path = repo.get("tokenizer.json")?;
            let weights_path = repo.get("model.safetensors")?;
            
            tracing::info!("Loading model configuration...");
            let config = std::fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&config)?;
            
            tracing::info!("Loading tokenizer...");
            let tokenizer = Tokenizer::from_file(tokenizer_path)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
            
            tracing::info!("Initializing device (CPU)...");
            let device = Device::Cpu;
            
            tracing::info!("Loading model weights...");
            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)?
            };
            
            let model = BertModel::load(vb, &config)?;
            
            Ok(EmbeddingModel {
                model,
                tokenizer,
                device,
            })
        })
        .await??;
        
        *model_guard = Some(model_data);
        tracing::info!("✓ Embedding model initialized successfully");
    }
    
    drop(model_guard);
    Ok(EMBEDDING_MODEL.clone())
}

/// Embeds multiple text chunks using Candle BERT model
pub async fn embed_chunks(chunks: &[TextChunk]) -> Result<Vec<EmbeddedChunk>> {
    let model_arc = get_or_init_model().await?;
    
    // Prepare texts for embedding
    let texts: Vec<String> = chunks
        .iter()
        .map(|chunk| {
            // Combine identifier and content for better semantic representation
            format!("{}\n{}", chunk.chunk_identifier, chunk.content)
        })
        .collect();
    
    tracing::debug!("Generating embeddings for {} chunks...", texts.len());
    
    // Generate embeddings (blocking operation, run in separate thread)
    let embeddings = {
        let model_arc_clone = model_arc.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<Vec<f32>>> {
            let model_guard = model_arc_clone.blocking_lock();
            let model_data = model_guard.as_ref().unwrap();
            
            let mut all_embeddings = Vec::new();
            
            // Process texts in batches to manage memory
            const BATCH_SIZE: usize = 8;
            for batch in texts.chunks(BATCH_SIZE) {
                let batch_embeddings = encode_batch(batch, model_data)?;
                all_embeddings.extend(batch_embeddings);
            }
            
            Ok(all_embeddings)
        })
        .await??
    };
    
    // Combine chunks with their embeddings
    let embedded_chunks = chunks
        .iter()
        .zip(embeddings.iter())
        .map(|(chunk, embedding)| EmbeddedChunk {
            chunk: chunk.clone(),
            embedding: embedding.clone(),
        })
        .collect();
    
    Ok(embedded_chunks)
}

/// Embeds a single query string
pub async fn embed_query(query: &str) -> Result<Vec<f32>> {
    let model_arc = get_or_init_model().await?;
    
    tracing::debug!("Generating query embedding...");
    
    let query_owned = query.to_string();
    let embedding = {
        let model_arc_clone = model_arc.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<f32>> {
            let model_guard = model_arc_clone.blocking_lock();
            let model_data = model_guard.as_ref().unwrap();
            
            let embeddings = encode_batch(&[query_owned], model_data)?;
            Ok(embeddings.into_iter().next().unwrap_or_default())
        })
        .await??
    };
    
    Ok(embedding)
}

fn encode_batch(texts: &[String], model_data: &EmbeddingModel) -> Result<Vec<Vec<f32>>> {
    let tokens_list: Vec<_> = texts
        .iter()
        .map(|text| {
            model_data
                .tokenizer
                .encode(text.clone(), true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))
        })
        .collect::<Result<Vec<_>>>()?;
    
    let token_ids: Vec<Vec<u32>> = tokens_list
        .iter()
        .map(|tokens| tokens.get_ids().to_vec())
        .collect();
    
    let attention_masks: Vec<Vec<u32>> = tokens_list
        .iter()
        .map(|tokens| tokens.get_attention_mask().to_vec())
        .collect();
    
    // Find max length for padding
    let max_len = token_ids.iter().map(|ids| ids.len()).max().unwrap_or(0);
    
    // Pad sequences
    let token_ids_padded: Vec<Vec<u32>> = token_ids
        .iter()
        .map(|ids| {
            let mut padded = ids.clone();
            padded.resize(max_len, 0);
            padded
        })
        .collect();
    
    let attention_masks_padded: Vec<Vec<u32>> = attention_masks
        .iter()
        .map(|mask| {
            let mut padded = mask.clone();
            padded.resize(max_len, 0);
            padded
        })
        .collect();
    
    // Convert to tensors
    let token_ids_array: Vec<u32> = token_ids_padded.concat();
    let attention_mask_array: Vec<u32> = attention_masks_padded.concat();
    
    let token_ids_tensor = Tensor::from_vec(
        token_ids_array,
        (texts.len(), max_len),
        &model_data.device,
    )?;
    
    let attention_mask_tensor = Tensor::from_vec(
        attention_mask_array,
        (texts.len(), max_len),
        &model_data.device,
    )?;
    
    // Run model (third parameter is token_type_ids, None for sentence embeddings)
    let embeddings = model_data.model.forward(&token_ids_tensor, &attention_mask_tensor, None)?;
    
    // Mean pooling
    let (batch_size, seq_len, hidden_size) = embeddings.dims3()?;
    
    let mut result_embeddings = Vec::new();
    
    for i in 0..batch_size {
        let seq_embeddings = embeddings.i(i)?;
        let mask = attention_mask_tensor.i(i)?;
        
        // Apply mean pooling with attention mask
        let mask_expanded = mask
            .unsqueeze(1)?
            .expand((seq_len, hidden_size))?
            .to_dtype(DTYPE)?;
        
        let masked_embeddings = (seq_embeddings * mask_expanded)?;
        let sum_embeddings = masked_embeddings.sum(0)?;
        let sum_mask = mask.sum_all()?.to_scalar::<f32>()?;
        
        // Convert sum_mask to tensor for division
        let sum_mask_tensor = Tensor::new(&[sum_mask], &model_data.device)?.to_dtype(DTYPE)?;
        let mean_embedding = sum_embeddings.broadcast_div(&sum_mask_tensor)?;
        
        // Normalize
        let embedding_norm = mean_embedding.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
        let normalized = if embedding_norm > 0.0 {
            let norm_tensor = Tensor::new(&[embedding_norm], &model_data.device)?.to_dtype(DTYPE)?;
            mean_embedding.broadcast_div(&norm_tensor)?
        } else {
            mean_embedding
        };
        
        let embedding_vec: Vec<f32> = normalized.to_vec1()?;
        result_embeddings.push(embedding_vec);
    }
    
    Ok(result_embeddings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::models::ChunkType;
    
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored (requires model download)
    async fn test_embedding_dimension() {
        let query = "What are the privacy rights?";
        let embedding = embed_query(query).await.unwrap();
        
        // all-MiniLM-L6-v2 produces 384-dimensional vectors
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored (requires model download)
    async fn test_embed_chunks() {
        let chunks = vec![
            TextChunk {
                bill_id: Uuid::new_v4(),
                bill_number: "TEST/2024".to_string(),
                chunk_index: 0,
                chunk_type: ChunkType::Clause,
                chunk_identifier: "Clause 1".to_string(),
                content: "This is a test clause about data protection.".to_string(),
            },
        ];
        
        let embedded = embed_chunks(&chunks).await.unwrap();
        assert_eq!(embedded.len(), 1);
        assert_eq!(embedded[0].embedding.len(), EMBEDDING_DIM);
        
        // Check that embedding is normalized
        let magnitude: f32 = embedded[0].embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01, "Embedding should be normalized");
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored (requires model download)
    async fn test_semantic_similarity() {
        let chunks = vec![
            TextChunk {
                bill_id: Uuid::new_v4(),
                bill_number: "TEST/2024".to_string(),
                chunk_index: 0,
                chunk_type: ChunkType::Clause,
                chunk_identifier: "Clause 1".to_string(),
                content: "Data protection and privacy rights for citizens".to_string(),
            },
            TextChunk {
                bill_id: Uuid::new_v4(),
                bill_number: "TEST/2024".to_string(),
                chunk_index: 1,
                chunk_type: ChunkType::Clause,
                chunk_identifier: "Clause 2".to_string(),
                content: "Telecommunications infrastructure and network regulations".to_string(),
            },
        ];
        
        let embedded = embed_chunks(&chunks).await.unwrap();
        let query_embed = embed_query("privacy rights").await.unwrap();
        
        // Calculate cosine similarity
        let sim1: f32 = query_embed.iter().zip(&embedded[0].embedding).map(|(a, b)| a * b).sum();
        let sim2: f32 = query_embed.iter().zip(&embedded[1].embedding).map(|(a, b)| a * b).sum();
        
        // Query about privacy should be more similar to first chunk
        assert!(sim1 > sim2, "Privacy query should match privacy content better (sim1={}, sim2={})", sim1, sim2);
    }
}
