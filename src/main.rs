mod scraper;
mod extractor;
mod chunker;
mod embedder;
mod vector_store;
mod models;
mod web;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "civic-legislation")]
#[command(about = "Civic Legislation Knowledge Base - Ingestion Module", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch recent bills from PRS and ingest them
    Ingest {
        /// Number of bills to fetch
        #[arg(short, long, default_value_t = 5)]
        count: usize,
    },
    /// Query the knowledge base
    Query {
        /// The question to ask
        query: String,
        /// Number of results to return
        #[arg(short, long, default_value_t = 3)]
        limit: usize,
    },
    /// Initialize the vector database
    Init,
    /// Start the web server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            tracing::info!("Initializing vector database...");
            vector_store::initialize_collection().await?;
            tracing::info!("✓ Vector database initialized successfully");
        }
        Commands::Ingest { count } => {
            tracing::info!("Starting ingestion of {} bills...", count);
            
            // Step 1: Scrape bills
            tracing::info!("Fetching bills from PRS...");
            let bills = scraper::fetch_recent_bills(count).await?;
            tracing::info!("✓ Found {} bills", bills.len());
            
            // Step 2: Process each bill
            for bill in bills {
                tracing::info!("Processing: {}", bill.title);
                
                // Extract text from PDF
                tracing::info!("  → Extracting text from PDF...");
                let text = extractor::extract_text_from_pdf(&bill.pdf_url).await?;
                
                // Chunk the text
                tracing::info!("  → Chunking text semantically...");
                let chunks = chunker::chunk_text(&text, &bill.bill_number);
                tracing::info!("  → Created {} chunks", chunks.len());
                
                // Generate embeddings
                tracing::info!("  → Generating embeddings...");
                let embedded_chunks = embedder::embed_chunks(&chunks).await?;
                
                // Store in vector database
                tracing::info!("  → Storing in vector database...");
                vector_store::store_chunks(&bill, &embedded_chunks).await?;
                
                tracing::info!("✓ Completed: {}", bill.title);
            }
            
            tracing::info!("✓ Ingestion completed successfully");
        }
        Commands::Query { query, limit } => {
            tracing::info!("Searching for: \"{}\"", query);
            
            // Generate query embedding
            let query_vector = embedder::embed_query(&query).await?;
            
            // Search vector database
            let results = vector_store::search(&query_vector, limit).await?;
            
            // Display results
            println!("\n{}", "=".repeat(80));
            println!("Search Results for: \"{}\"", query);
            println!("{}", "=".repeat(80));
            
            if results.is_empty() {
                println!("\nNo results found. Try ingesting some bills first with:");
                println!("  cargo run -- ingest");
            } else {
                for (idx, result) in results.iter().enumerate() {
                    println!("\n[Result {}] Score: {:.4}", idx + 1, result.score);
                    println!("Bill: {}", result.bill_title);
                    println!("Section: {}", result.chunk_identifier);
                    println!("\nContent:\n{}", result.content);
                    println!("{}", "-".repeat(80));
                }
            }
        }
        Commands::Serve { port } => {
            tracing::info!("Starting web server on port {}...", port);
            
            let app = web::create_router();
            
            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            
            tracing::info!("🌐 Server running at http://localhost:{}", port);
            tracing::info!("📚 Access the civic knowledge base at http://localhost:{}/", port);
            
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}

