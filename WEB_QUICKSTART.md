# Web Frontend Quick Start

## ✅ System Status

Your civic legislation platform is now running with:

- **PostgreSQL Database**: `civic_postgres` container (port 5432) - ✓ Healthy
- **Qdrant Vector DB**: `civic_qdrant` container (ports 6333, 6334) - ✓ Running  
- **Web Server**: `http://localhost:3000` - ✓ Running

## 🎨 Frontend Features

### 1. **Homepage** - `http://localhost:3000`
- **Search Bar**: Type a query about legislation (e.g., "minister arrested")
- **Live Suggestions**: Top 3 relevant results appear as you type
- **Recent Bills Sidebar**: Click any bill to open its forum
- **Formal Design**: Black & white, professional aesthetic

### 2. **Search Functionality**
The search uses your vector embeddings (BERT model) to find semantically similar content:
```
User types → Query embedded → Qdrant search → Top 3 results displayed
```

### 3. **Bill Forums**
When you click a bill or search result:
- Forum section becomes visible
- Shows dummy reviews with Support/Oppose/Critique stances
- Upvote/downvote buttons (functional UI, backend TODO)
- Form to add your own review

## 🚀 Usage

### Start Everything
```powershell
# 1. Start Docker services
docker-compose up -d

# 2. Start web server (if not already running)
cargo run -- serve --port 3000

# 3. Open browser
start http://localhost:3000
```

### Ingest Real Data
Before searching, you need to ingest bills:
```powershell
# Fetch and ingest 3 bills from PRS India
cargo run --release -- ingest --count 3
```

### Test Search
Once bills are ingested, try these queries:
- "What happens if a minister is arrested?"
- "Chief Minister removal procedures"
- "Constitutional amendments for Delhi"

## 🔧 What Was Fixed

### Issue: PostgreSQL Healthcheck Failing
**Problem**: The healthcheck was looking for database `civic_user` instead of `civic_legislation`

**Solution**: Updated `docker-compose.yml`:
```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U civic_user -d civic_legislation"]
```

## 📁 File Structure

```
representation_upon_enigma/
├── templates/               # Askama HTML templates
│   ├── base.html           # Base layout with header
│   ├── index.html          # Main page (search + sidebar)
│   ├── search_suggestions.html  # Autocomplete results
│   └── forum.html          # Bill discussion page
├── static/
│   └── css/
│       └── main.css        # Formal B&W styling
├── src/
│   ├── main.rs             # CLI + serve command
│   ├── web.rs              # Axum routes & handlers
│   ├── embedder.rs         # BERT embeddings
│   ├── vector_store.rs     # Qdrant integration
│   └── ...
└── docker-compose.yml      # PostgreSQL + Qdrant
```

## 🎯 Next Steps

### Phase 1: Connect Real Data (In Progress)
- [x] Vector search backend integrated
- [x] Web UI with HTMX
- [ ] Connect PostgreSQL for bill metadata
- [ ] Store reviews in database

### Phase 2: User Authentication
- [ ] Implement blind hashing (voter ID verification)
- [ ] Constituency-based access control
- [ ] Session management

### Phase 3: AI Features
- [ ] Automatic stance classification
- [ ] Toxicity filter
- [ ] Summarization for MPs

### Phase 4: Production
- [ ] Rate limiting
- [ ] Caching layer
- [ ] Production deployment

## 🐛 Troubleshooting

### Web server not responding?
```powershell
# Check if process is running
Get-Process | Where-Object {$_.ProcessName -like "*representation*"}

# Restart server
cargo run -- serve --port 3000
```

### Docker containers unhealthy?
```powershell
# Check status
docker ps

# View logs
docker logs civic_postgres
docker logs civic_qdrant

# Restart
docker-compose restart
```

### Search returning no results?
```powershell
# Check Qdrant has data
Invoke-WebRequest -Uri "http://localhost:6333/collections/bill_clauses" -UseBasicParsing | ConvertFrom-Json

# Re-ingest if empty
cargo run --release -- ingest --count 2
```

## 🎨 Design Philosophy

**Formal & Serious**: This is a civic platform dealing with legislation
- Georgia serif font for gravitas
- High contrast black/white
- Minimal distractions
- Clear information hierarchy

**Accessible**: No heavy JavaScript, works with screen readers
- HTMX for dynamic updates
- Semantic HTML
- Clean focus states

**Trust-Building**: Professional appearance for credibility
- Clean borders and spacing
- Consistent typography
- Clear call-to-actions

---

**Status**: ✅ All systems operational  
**Access**: http://localhost:3000  
**Docs**: See `FRONTEND.md` for API details


