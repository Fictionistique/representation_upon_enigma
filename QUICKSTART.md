# Quick Start Guide

Get up and running in 5 minutes!

## Prerequisites

1. ✅ **Rust** - Install from https://rustup.rs/
2. ✅ **Docker Desktop** - Install from https://docs.docker.com/get-docker/
3. ✅ **200MB free disk space** - For ML model + data

## Option A: Automated Setup (Recommended)

### Run the setup script:

```powershell
.\setup.ps1
```

This will:
- Start Docker services
- Build the project  
- Initialize the vector database
- Verify everything works

### Then ingest and query:

```powershell
# Ingest 3 bills
.\run.ps1 ingest

# Query the knowledge base
.\run.ps1 query "What are the data protection rights?"
```

## Option B: Manual Setup

### 1. Start Services

```bash
docker-compose up -d
```

### 2. Initialize Database

```bash
cargo run -- init
```

### 3. Ingest Bills

```bash
cargo run -- ingest --count 3
```

**Note**: First run downloads the BERT model (~90MB) from HuggingFace - takes 1-2 minutes.

### 4. Query

```bash
cargo run -- query "What are data protection rights?"
```

## Example Queries

```bash
cargo run -- query "How are telecommunications regulated?"
cargo run -- query "What penalties exist for data breaches?"
cargo run -- query "Who can access personal data?"
cargo run -- query "What are the obligations of data fiduciaries?"
```

## Useful Commands

### View Qdrant Dashboard
Open http://localhost:6333/dashboard

### Stop Services
```powershell
.\stop.ps1
```
or
```bash
docker-compose down
```

### View Logs
```bash
docker-compose logs -f qdrant
docker-compose logs -f postgres
```

### Rebuild After Changes
```bash
cargo build --release
```

## Troubleshooting

### "Cannot connect to Docker"
→ Start Docker Desktop and wait for it to fully initialize

### "Failed to connect to Qdrant"
→ Run `docker-compose ps` to check services are running
→ Run `docker-compose up -d` to restart

### "No results found"
→ Ingest some bills first: `cargo run -- ingest --count 3`

### Model download failed?
→ Check internet connection. Model caches to `~/.cache/huggingface/` for future runs.

## What's Happening Under the Hood?

1. **Ingestion**:
   - Fetches bills from PRS India (or uses demo data)
   - Extracts text from PDFs
   - Chunks by clauses/sections (semantic boundaries)
   - Generates 384-dim neural embeddings using Candle + BERT
   - Stores in Qdrant vector DB

2. **Query**:
   - Converts your question to BERT embedding
   - Searches similar vectors (cosine similarity)
   - Returns top matching bill sections

**Note**: Uses Candle (pure Rust ML) with sentence-transformers for true semantic search!

## Next Steps

- Read the full [README.md](README.md) for details
- Explore [PLAN.md](PLAN.md) for the complete vision
- Check `src/` for code structure
- Visit http://localhost:6333/dashboard to explore vectors

## Pro Tips

- Semantic search understands meaning, not just keywords
- Try natural language questions
- Ingest more bills for broader coverage
- Check `RUST_LOG=debug` for detailed logs
- First embedding generation is slower (model initialization)

---

🎉 **Ready to build the future of civic engagement!**

