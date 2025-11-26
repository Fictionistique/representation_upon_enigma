# Representation Upon Enigma

A full-stack civic-tech platform for Indian legislation, enabling citizens to engage with bills and providing MPs with constituent sentiment analysis.

## Overview

**Representation Upon Enigma** is a comprehensive web application that bridges the gap between Indian citizens and their representatives. The platform combines:

- **RAG-based Knowledge System**: Semantic search across legislative bills using vector embeddings
- **Civic Forum**: Discussion threads for each bill with stance tracking (Support/Oppose/Critique)
- **AI Moderation**: Automated content moderation using Ollama/Llama 3.2
- **User Authentication**: Secure registration with constituency/pincode-based location tracking
- **MP Dashboard**: PDF reports with visual sentiment analysis for Members of Parliament
- **Rate Limiting**: Anti-spam protection for forum posts

Built entirely in **Rust** for performance, safety, and reliability.

---

## Features

### For Citizens
- 🔍 **Semantic Search**: Find relevant bills using natural language queries
- 💬 **Bill Forums**: Discuss legislation with Support/Oppose/Critique stances
- 🗳️ **Voting System**: Upvote/downvote posts to surface quality content
- 👤 **User Profiles**: Track your posts and engagement history
- 🛡️ **AI Moderation**: Automatic filtering of toxic/spam content
- 📍 **Location-based**: Register with pincode or constituency

### For MPs & Representatives
- 📊 **Constituency Reports**: Generate comprehensive PDF reports
- 📈 **Sentiment Graphs**: Visual breakdown of Support/Oppose/Critique per bill
- 📝 **Detailed Posts**: All constituent feedback organized by bill
- 🎯 **Data-Driven Insights**: Understand constituent priorities

### Technical Highlights
- ⚡ **Pure Rust**: High-performance backend with Axum web framework
- 🧠 **Vector Search**: Qdrant for semantic bill search
- 🔐 **Secure Auth**: Argon2 password hashing, HTTP-only cookies
- 🎨 **Modern UI**: HTMX for dynamic interactions without heavy JavaScript
- 🤖 **AI-Powered**: Ollama integration for content moderation
- 📄 **PDF Generation**: Professional constituency reports with colored graphs

---

## Tech Stack

### Backend
- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL 16 (SQLx ORM)
- **Vector Database**: Qdrant
- **Embeddings**: Candle + sentence-transformers/all-MiniLM-L6-v2
- **Authentication**: Argon2 password hashing
- **Moderation**: Ollama/Llama 3.2

### Frontend
- **Templating**: Askama
- **Interactivity**: HTMX
- **Styling**: Custom CSS (formal black & white design)

### Infrastructure
- **Containerization**: Docker Compose
- **PDF Generation**: printpdf
- **Rate Limiting**: Governor crate

---

## Prerequisites

- **Rust** (stable) - [Install](https://rustup.rs/)
- **Docker & Docker Compose** - [Install](https://docs.docker.com/get-docker/)
- **Ollama** (optional, for AI moderation) - [Install](https://ollama.ai/)
- ~500MB disk space for ML models + data

---

## Quick Start

### 1. Clone & Setup

```bash
git clone <repository-url>
cd representation_upon_enigma
```

### 2. Start Services

Start PostgreSQL and Qdrant using Docker:

```bash
docker-compose up -d
```

Verify services are running:

```bash
docker-compose ps
```

### 3. Set Environment Variables

Create a `.env` file or export:

```bash
export DATABASE_URL="postgres://civic_user:civic_pass@localhost/civic_legislation"
```

### 4. Initialize Vector Database

```bash
cargo run -- init
```

This creates the Qdrant collection for bill embeddings.

### 5. Ingest Bills

Fetch and process bills from PRS India:

```bash
cargo run -- ingest --count 5
```

**Note**: First run downloads the sentence-transformers model (~90MB) from HuggingFace.

### 6. Start Web Server

```bash
cargo run -- serve --port 3000
```

Access the application at **http://localhost:3000**

---

## CLI Commands

### Initialize Vector Database

```bash
cargo run -- init
```

Creates/resets the Qdrant collection.

### Ingest Bills

```bash
cargo run -- ingest [--count <number>]
```

Options:
- `--count`: Number of bills to fetch (default: 5)

### Query Knowledge Base

```bash
cargo run -- query "<your question>" [--limit <number>]
```

Options:
- `--limit`: Number of results (default: 3)

Example:
```bash
cargo run -- query "What are data protection rights?"
```

### Start Web Server

```bash
cargo run -- serve [--port <port>]
```

Options:
- `--port`: Port to listen on (default: 3000)

---

## Project Structure

```
.
├── src/
│   ├── main.rs           # CLI interface & entry point
│   ├── models.rs         # Data structures
│   ├── web.rs            # Web routes & handlers
│   ├── auth.rs           # User authentication & sessions
│   ├── db.rs             # Database operations
│   ├── scraper.rs        # Bill fetching from PRS India
│   ├── extractor.rs      # PDF text extraction
│   ├── chunker.rs        # Semantic text chunking
│   ├── embedder.rs       # Sentence embeddings (Candle + BERT)
│   ├── vector_store.rs   # Qdrant integration
│   ├── moderation.rs     # AI content moderation
│   ├── rate_limit.rs     # Rate limiting logic
│   └── pdf_generator.rs  # MP constituency reports
├── templates/            # Askama HTML templates
│   ├── base.html
│   ├── index.html
│   ├── forum.html
│   ├── forum_page.html
│   ├── login.html
│   ├── register.html
│   ├── profile.html
│   └── ...
├── static/
│   └── css/
│       └── main.css      # Formal black & white styling
├── docker-compose.yml    # PostgreSQL + Qdrant services
├── init.sql              # Database schema
├── Cargo.toml            # Rust dependencies
└── README.md
```

---

## How It Works

### Ingestion Pipeline

1. **Scraper** (`scraper.rs`): Fetches bills from PRS India website
2. **Extractor** (`extractor.rs`): Converts PDF to clean text
3. **Chunker** (`chunker.rs`): Splits text by clauses/sections (semantic boundaries)
4. **Embedder** (`embedder.rs`): Generates 384-dim vectors using Candle + BERT
5. **Vector Store** (`vector_store.rs`): Stores chunks with metadata in Qdrant

### Search Pipeline

1. **Embed Query**: Convert user question to vector
2. **Vector Search**: Find top-k similar chunks (cosine similarity)
3. **Return Results**: Display relevant bill sections with scores

### Forum System

1. **User Posts**: Citizens submit Support/Oppose/Critique stances
2. **AI Moderation**: Ollama checks for toxicity/spam
   - **Falafel** → Approved
   - **Popcorn** → Rejected
   - **Default** → Admin review
3. **Voting**: Users upvote/downvote posts
4. **Rate Limiting**: Prevents spam (configurable posts/hour)

### MP Dashboard

1. **Constituency Selection**: MP chooses their constituency
2. **Data Aggregation**: System queries all posts from that constituency
3. **PDF Generation**: Creates report with:
   - Colored bar graphs (Support=green, Oppose=red, Critique=yellow)
   - Bill details and metadata
   - All constituent posts with voting data
4. **Download**: PDF automatically downloads

---

## Database Schema

### Core Tables
- **`bills`**: Legislative bills with metadata
- **`bill_chunks`**: Semantically chunked bill text
- **`users`**: User accounts with Argon2 password hashing
- **`constituencies`**: Indian parliamentary constituencies (25 major cities)
- **`pincode_constituencies`**: Pincode to constituency mapping
- **`sessions`**: User session tokens (7-day expiry)
- **`posts`**: Forum posts with stance and moderation status
- **`post_votes`**: User votes (prevents duplicate voting)
- **`rate_limits`**: Rate limiting tracking

---

## Configuration

### Environment Variables

```env
DATABASE_URL=postgres://civic_user:civic_pass@localhost/civic_legislation
QDRANT_URL=http://localhost:6333
QDRANT_COLLECTION=legislation_chunks
VECTOR_DIMENSION=384
RUST_LOG=info
```

### Docker Services

- **PostgreSQL**: Port 5432
- **Qdrant**: Port 6333 (HTTP), 6334 (gRPC)
- **Qdrant Dashboard**: http://localhost:6333/dashboard

### Ollama (Optional)

For AI moderation, install Ollama and pull Llama 3.2:

```bash
ollama pull llama3.2
```

Fallback keyword-based moderation is used if Ollama is unavailable.

---

## Development

### Run Tests

```bash
cargo test
```

### Check Code Quality

```bash
cargo clippy
cargo fmt
```

### View Logs

Set log level for debugging:

```bash
RUST_LOG=debug cargo run -- serve
```

### Access Qdrant Dashboard

Open http://localhost:6333/dashboard to explore the vector database.

---

## API Endpoints

### Public Routes
- `GET /` - Homepage with recent bills
- `GET /login` - Login page
- `GET /register` - Registration page
- `GET /f/:bill_id` - Forum page for specific bill
- `GET /u/:username` - User profile page

### API Routes
- `GET /api/search?query=...` - Semantic search
- `GET /api/bills?page=N` - Paginated bills list
- `GET /api/bill/:id/forum` - Forum content (HTMX partial)
- `POST /api/bill/:id/review` - Submit new post
- `POST /api/review/:id/upvote` - Upvote post
- `POST /api/review/:id/downvote` - Downvote post
- `GET /api/constituencies` - List all constituencies (JSON)
- `GET /api/mp/report?constituency_id=N` - Generate MP PDF report

### Authentication Routes
- `POST /login` - User login
- `POST /register` - User registration
- `GET /logout` - User logout
- `POST /u/:username` - Update profile

---

## Troubleshooting

### "Failed to connect to database"

Ensure Docker services are running:

```bash
docker-compose up -d
docker-compose logs postgres
```

### "Failed to connect to Qdrant"

Check Qdrant status:

```bash
docker-compose logs qdrant
curl http://localhost:6333/health
```

### Model Download Failed

First run downloads ~90MB from HuggingFace. Ensure stable internet. Models are cached in `~/.cache/huggingface/`.

### Slow Embedding Generation

BERT inference on CPU takes a few seconds per batch. This is normal for development.

### "No results found" in Search

Ingest bills first:

```bash
cargo run -- ingest --count 5
```

### Constituencies Not Showing in Registration

The database is initialized with 25 major Indian constituencies. If they're not appearing, manually apply the schema:

```bash
docker exec -i civic_postgres psql -U civic_user -d civic_legislation < init.sql
```

---

## Security Features

- ✅ **Argon2 Password Hashing**: Industry-standard password security
- ✅ **HTTP-only Cookies**: Prevents XSS attacks on session tokens
- ✅ **Session Expiry**: 7-day automatic logout
- ✅ **Rate Limiting**: Prevents forum spam and abuse
- ✅ **AI Moderation**: Filters toxic/harmful content
- ✅ **SQL Injection Prevention**: Parameterized queries via SQLx
- ✅ **Input Validation**: Username uniqueness, required fields

---

## Future Enhancements

### Planned Features
- 📧 Email verification for registration
- 🔑 Password reset functionality
- 👨‍💼 Admin panel for moderation review
- 🔔 Real-time notifications
- 📱 Mobile-responsive design improvements
- 🌐 Multi-language support (Hindi, regional languages)
- 🔐 Two-factor authentication
- 📊 Advanced analytics for MPs
- 🤝 Integration with official government APIs

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust conventions (`cargo fmt`, `cargo clippy`)
- Write tests for new features
- Update documentation as needed

---

## License

MIT License - See LICENSE file for details

---

## Acknowledgments

- **PRS Legislative Research**: Bill data source
- **HuggingFace**: Sentence transformer models
- **Qdrant**: Vector database
- **Ollama**: AI moderation capabilities

---

## Contact & Support

For questions, issues, or feature requests, please open an issue on GitHub.

---

**Built with ❤️ for civic engagement in India**

*Empowering citizens to understand legislation and enabling representatives to hear their constituents.*
