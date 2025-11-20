# Architecture Documentation

## Module 1: Ingestion & Knowledge (The "Librarian") - IMPLEMENTED

This module is responsible for fetching bills, processing them, and making them searchable through vector embeddings.

### Components

#### 1. Scraper Service (`src/scraper.rs`)
- **Responsibility**: Fetches legislative bills from PRS India
- **Implementation**: 
  - Attempts to scrape from https://prsindia.org/billtrack/recent-bills
  - Falls back to curated demo bills if scraping fails
  - Extracts bill metadata (title, number, year, PDF URL)
- **Async**: Yes (tokio)

#### 2. Text Extractor (`src/extractor.rs`)
- **Responsibility**: Converts PDF documents to plain text
- **Implementation**:
  - Downloads PDFs via HTTP
  - Uses `lopdf` to extract text
  - Cleans and normalizes extracted text
  - Falls back to demo content if PDF is unavailable
- **Async**: Yes (tokio)

#### 3. Semantic Chunker (`src/chunker.rs`)
- **Responsibility**: Splits legislative text into meaningful units
- **Implementation**:
  - Regex-based detection of legislative structure (Clauses, Sections, Chapters)
  - Falls back to paragraph-based chunking if no structure found
  - Each chunk has type, identifier, and content
- **Async**: No (CPU-bound)

#### 4. Embedder (`src/embedder.rs`)
- **Responsibility**: Generates vector embeddings for semantic search
- **Implementation**: Candle + BERT (sentence-transformers/all-MiniLM-L6-v2)
  - Pure Rust ML framework (no Python dependencies)
  - Downloads model from HuggingFace Hub on first run (~90MB)
  - Generates 384-dimensional normalized vectors
  - Mean pooling with attention masks
  - Batch processing for efficiency (8 texts per batch)
  - True semantic understanding (not just keyword matching)
- **Async**: Yes (tokio with blocking tasks for CPU-intensive operations)

#### 5. Vector Store (`src/vector_store.rs`)
- **Responsibility**: Manages Qdrant vector database
- **Implementation**:
  - Creates/manages collections in Qdrant
  - Stores embeddings with rich metadata
  - Performs cosine similarity search
  - Batch insertion for efficiency
- **Async**: Yes (tokio)

### Data Flow

```
┌─────────────┐
│ User Input  │
│ (CLI)       │
└──────┬──────┘
       │
       ├─── ingest ──────────────────────────┐
       │                                     │
       v                                     v
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Scraper    │───▶│  Extractor   │───▶│   Chunker    │
│ (PRS Bills)  │    │  (PDF→Text)  │    │  (Semantic)  │
└──────────────┘    └──────────────┘    └───────┬──────┘
                                                 │
                                                 v
                                        ┌──────────────┐
                                        │   Embedder   │
                                        │ (Candle+BERT)│
                                        └───────┬──────┘
                                                │
                                                v
                                        ┌──────────────┐
                                        │ Vector Store │
                                        │   (Qdrant)   │
                                        └──────────────┘
```

```
User Query ──▶ Embedder ──▶ Vector Store ──▶ Search Results
           (Candle+BERT)    (Cosine Sim)     (Top-K chunks)
```

### Database Schema

#### PostgreSQL (init.sql)

**bills** table:
- `id`: UUID (PK)
- `title`: TEXT
- `bill_number`: TEXT (UNIQUE)
- `year`: INTEGER
- `session`: TEXT
- `status`: TEXT
- `introduction_date`: DATE
- `pdf_url`: TEXT
- `extracted_text`: TEXT
- `created_at`: TIMESTAMP
- `updated_at`: TIMESTAMP

**bill_chunks** table:
- `id`: UUID (PK)
- `bill_id`: UUID (FK → bills)
- `chunk_index`: INTEGER
- `chunk_type`: TEXT (clause, section, preamble)
- `chunk_identifier`: TEXT
- `content`: TEXT
- `embedding_id`: TEXT (reference to Qdrant)
- `created_at`: TIMESTAMP

#### Qdrant (Vector DB)

**Collection**: `legislation_chunks`
- **Vector dimension**: 384
- **Distance metric**: Cosine
- **Payload fields**:
  - `bill_id`: UUID
  - `bill_title`: String
  - `bill_number`: String
  - `year`: Integer
  - `chunk_index`: Integer
  - `chunk_type`: String
  - `chunk_identifier`: String
  - `content`: String (full text for display)

### API Design

#### CLI Interface

```bash
# Initialize vector database
cargo run -- init

# Ingest bills from PRS
cargo run -- ingest [--count N]

# Query the knowledge base
cargo run -- query "your question" [--limit N]
```

### Configuration

Environment variables (`.env`):
- `DATABASE_URL`: PostgreSQL connection string
- `QDRANT_URL`: Qdrant server URL
- `QDRANT_COLLECTION`: Collection name
- `VECTOR_DIMENSION`: Embedding dimension
- `RUST_LOG`: Logging level

### Error Handling

- **Scraping failures**: Falls back to demo bills
- **PDF download failures**: Uses mock content
- **PDF parsing failures**: Uses demo content
- **Qdrant unavailable**: Clear error message
- All operations use `anyhow::Result` for error propagation

### Performance Considerations

1. **PDF Downloads**: Async HTTP, cached locally
2. **Embedding Generation**: Batch processing
3. **Vector Insertion**: Batch upserts (100 points at a time)
4. **Search**: Optimized with Qdrant's HNSW index

### Testing Strategy

- Unit tests for chunker, embedder logic
- Integration tests (ignored by default) for Qdrant
- Demo data ensures functionality even without network

### Future Enhancements

1. **Embedding Optimizations**:
   - GPU acceleration (CUDA/Metal support in Candle)
   - Quantized models for faster inference
   - Implement hybrid search (keyword + semantic)

2. **Incremental Updates**:
   - Track bill versions
   - Update only changed chunks
   - Maintain chunk history

3. **Advanced Chunking**:
   - ML-based chunk boundary detection
   - Cross-reference detection
   - Amendment tracking

4. **Caching**:
   - Cache embeddings in PostgreSQL
   - Cache search results (Redis)
   - Materialized views for aggregations

---

## Modules 2-5: Not Yet Implemented

See PLAN.md for full system design.

### Module 2: Identity Sentinel (Voter Verification)
### Module 3: Civic Forum Engine (Discussions)
### Module 4: Safety & Moderation Guard
### Module 5: Representative Intelligence (Dashboards)

