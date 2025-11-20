Project Concept: A civic-tech knowledge base and forum for Indian legislation.

Core Functionality:

    The Data: Hosts legislative bills (sourced from PRS) that are tracked dynamically across multiple parliamentary sessions and reading stages (not static documents).

    The Engine: Uses RAG (Retrieval-Augmented Generation) to let users semantically search and ask questions about complex legal text/clauses.

    The Users: Citizens (verified by constituency) who discuss bills and declare their stance (Support/Oppose).

    The Stakeholders: Elected Representatives (MPs) who access aggregated dashboards to view verified constituent sentiment and data-driven summaries rather than raw comments.

Goal: To demystify legal jargon for voters and provide representatives with a noise-free signal of their constituency's beliefs.

I. The Tech Stack (Tools & Crates)

Core Infrastructure

    Language: Rust (Stable)

    Web Framework: Axum (Ergonomic, modular, runs on Tokio).

    Frontend: Askama (Type-safe HTML templates) + HTMX (For dynamic interactions without heavy JS).

    Async Runtime: Tokio.

Data Persistence

    Relational DB: PostgreSQL (Users, Comments, Bill Metadata, Constituencies).

    ORM/SQL: SQLx (Async, raw SQL with compile-time checking).

    Vector DB: Qdrant (Rust-native, high-performance vector search).

Domain-Specific Crates

    PDF Parsing: pdf (by pdf-rs) or lopdf (Low-level object manipulation) for extracting text from legislative documents.

    Geography: geo and geo-types (For point-in-polygon checks if mapping Pincodes to Constituency shapes).

    Cryptography: hmac, sha2, argon2 (For the Blind Hashing of Voter IDs).

AI & NLP

    Inference Engine: candle-core / candle-transformers (HuggingFace’s minimalist ML framework written in pure Rust). It is lighter than rust-bert.

    LLM Orchestration: langchain-rust (To manage the RAG pipeline).

II. System Modules & Components

The system divides into five distinct technical modules.

1. Ingestion & Knowledge Module (The "Librarian")

    Responsibility: Fetching bills, cleaning data, and making it searchable.

    Components:

        Scraper Service: A scheduled tokio::task using reqwest and scraper to monitor PRS/Sansad TV for new PDF links. (Only bills, not conversations and other irrelevant PDFs)

        Text Extractor: Converts multi-column PDF text into linear Markdown-compatible text.

        Chunker: Splits text by "Clauses" or "Sections" (Semantic chunking) rather than arbitrary character counts.

        Embedder: Passes chunks to a local embedding model (e.g., all-MiniLM-L6-v2 via Candle) to generate vectors.

        Vector Writer: Pushes vectors + metadata (Bill ID, Clause No) to Qdrant.

2. Identity Sentinel (The "Gatekeeper")

    Responsibility: Verifying voters without storing identity.

    Components:

        Blind Hasher: The generic fn(epical_id, server_pepper) -> hash function.

        KYC Connector: Integration with a Voter ID verification API (e.g., via a commercial wrapper or manual verification queue).

        Session Manager: Issues HttpOnly cookies/JWTs containing user_hash and constituency_id. No PII in the token.

3. Civic Forum Engine (The "Town Hall")

    Responsibility: Handling user discussions and storing structured opinions.

    Components:

        Thread Manager: Associates comments with specific bill_id or clause_id.

        Constituency Filter: A middleware that attaches the user's constituency_id to every write operation.

        Stance Classifier (AI): An async service that runs incoming comments through a Zero-Shot Classifier (Candidate labels: "Support", "Oppose", "Constructive Critique") before DB insertion.

4. Safety & Moderation Guard (The "Bouncer")

    Responsibility: Preventing toxicity and bot spam.

    Components:

        Toxicity Filter: A lightweight BERT model (via Candle) checking for hate speech/threats.

        Rate Limiter: governor crate to limit requests based on IP and User Hash (e.g., max 5 comments/hour).

        Switch-Case Guard: The logic preventing prompt injection (as discussed previously).

5. Representative Intelligence (The "Dashboard")

    Responsibility: Aggregating data for MPs.

    Components:

        Aggregator: SQL queries calculating % Support vs % Oppose per constituency per bill.

        Newsletter Cron: A templating engine that generates HTML emails summarizing the week's top arguments.