# Representation Upon Enigma

A civic-tech knowledge base and forum for Indian legislation, built with Rust.

## Overview

This project aims to demystify legal jargon for voters and provide representatives with data-driven insights from their constituents. The first module (Ingestion & Knowledge) is a RAG-based system that:

- Fetches legislative bills from PRS India
- Extracts and semantically chunks bill text
- Generates embeddings using sentence transformers
- Enables semantic search over legislation

## Tech Stack

- **Language**: Rust
- **Web Framework**: Axum (for future web interface)
- **Vector Database**: Qdrant
- **Relational Database**: PostgreSQL
- **Embeddings**: Candle (pure Rust ML) with sentence-transformers/all-MiniLM-L6-v2
- **PDF Parsing**: lopdf

## Prerequisites

- Rust (stable) - [Install](https://rustup.rs/)
- Docker & Docker Compose - [Install](https://docs.docker.com/get-docker/)
- ~200MB disk space for ML model + data

## Quick Start

### 1. Start Services

Start Qdrant and PostgreSQL using Docker:

```bash
docker-compose up -d
```

Verify services are running:

```bash
docker-compose ps
```

### 2. Initialize Vector Database

```bash
cargo run -- init
```

This creates the Qdrant collection for storing bill embeddings.

### 3. Ingest Bills

Fetch and process bills from PRS India:

```bash
cargo run -- ingest --count 3
```

This will:
- Download recent bills (or use demo data)
- Extract text from PDFs
- Chunk text semantically by clauses/sections
- Generate neural embeddings using Candle + BERT
- Store in Qdrant

**Note**: First run downloads the sentence-transformers model (~90MB) from HuggingFace - this is one-time only!

### 4. Query the Knowledge Base

Ask questions in natural language:

```bash
cargo run -- query "What are the data protection rights for citizens?"
```

```bash
cargo run -- query "How are telecommunications regulated?"
```

```bash
cargo run -- query "What penalties exist for data breaches?"
```

## CLI Commands

### Initialize

```bash
cargo run -- init
```

Creates/resets the vector database collection.

### Ingest

```bash
cargo run -- ingest [--count <number>]
```

Options:
- `--count`: Number of bills to fetch (default: 5)

### Query

```bash
cargo run -- query "<your question>" [--limit <number>]
```

Options:
- `--limit`: Number of results to return (default: 3)

## Project Structure

```
.
├── src/
│   ├── main.rs           # CLI interface
│   ├── models.rs         # Data structures
│   ├── scraper.rs        # Bill fetching from PRS
│   ├── extractor.rs      # PDF text extraction
│   ├── chunker.rs        # Semantic text chunking
│   ├── embedder.rs       # Sentence embeddings
│   └── vector_store.rs   # Qdrant integration
├── docker-compose.yml    # Service definitions
├── init.sql              # Database schema
└── Cargo.toml            # Dependencies
```

## How It Works

### Ingestion Pipeline

1. **Scraper**: Fetches bills from PRS India website (with fallback to demo data)
2. **Extractor**: Converts PDF to clean text
3. **Chunker**: Splits text by clauses, sections, and chapters (semantic boundaries)
4. **Embedder**: Generates 384-dimensional vectors using Candle + BERT (sentence-transformers/all-MiniLM-L6-v2)
5. **Vector Store**: Stores chunks with metadata in Qdrant

### Query Pipeline

1. **Embed Query**: Convert user question to vector using BERT
2. **Vector Search**: Find top-k similar chunks using cosine similarity
3. **Return Results**: Display relevant bill sections with scores

### Embedding Approach

Uses **Candle** (HuggingFace's pure Rust ML framework) with **sentence-transformers/all-MiniLM-L6-v2**:
- ✅ Pure Rust implementation (no Python dependencies)
- ✅ CPU-optimized inference
- ✅ Proper semantic understanding (not just keyword matching)
- ✅ 384-dimensional normalized embeddings
- ✅ Model auto-downloads from HuggingFace Hub on first run

## Configuration

Environment variables (`.env` file):

```env
DATABASE_URL=postgresql://civic_user:civic_pass@localhost:5432/civic_legislation
QDRANT_URL=http://localhost:6333
QDRANT_COLLECTION=legislation_chunks
VECTOR_DIMENSION=384
RUST_LOG=info
```

## Development

### Run Tests

```bash
cargo test
```

### Check Code

```bash
cargo clippy
cargo fmt
```

### View Logs

Set log level:

```bash
RUST_LOG=debug cargo run -- query "your question"
```

### Access Qdrant Dashboard

Open http://localhost:6333/dashboard in your browser to explore the vector database.

## Troubleshooting

### "Failed to connect to Qdrant"

Ensure Docker services are running:

```bash
docker-compose up -d
docker-compose logs qdrant
```

### Model download failed

First run downloads ~90MB from HuggingFace. Ensure stable internet. Models are cached in `~/.cache/huggingface/`.

### Slow embedding generation

BERT inference on CPU takes a few seconds per batch. This is normal. Consider upgrading to a GPU-enabled device for faster processing.

### "No results found"

Ingest some bills first:

```bash
cargo run -- ingest --count 5
```

## Next Steps

This is Module 1 of 5. Future modules will add:

- **Module 2**: Identity verification (voter ID hashing)
- **Module 3**: Forum and discussion threads
- **Module 4**: Moderation and safety
- **Module 5**: Representative dashboards

## License

MIT License - see LICENSE file

## Contributing

Contributions welcome! Please open an issue first to discuss changes.

---

Built with ❤️ for civic engagement in India

