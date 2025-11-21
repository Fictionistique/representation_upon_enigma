use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::{embedder, models, vector_store};

// Application State
#[derive(Clone)]
pub struct AppState {
    // Add database pool, etc. here later
}

// Templates
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    recent_bills: Vec<RecentBill>,
}

#[derive(Template)]
#[template(path = "search_suggestions.html")]
struct SearchSuggestionsTemplate {
    results: Vec<SearchResult>,
}

#[derive(Template)]
#[template(path = "forum.html")]
struct ForumTemplate {
    bill: BillInfo,
    reviews: Vec<Review>,
}

// Data structures
#[derive(Clone, Serialize)]
struct RecentBill {
    id: i32,
    title: String,
    number: String,
    year: i32,
}

#[derive(Clone, Serialize)]
struct SearchResult {
    bill_id: i32,
    bill_title: String,
    bill_number: String,
    section: String,
    score: String,
}

#[derive(Clone, Serialize)]
struct BillInfo {
    id: i32,
    title: String,
    number: String,
    year: i32,
}

#[derive(Clone, Serialize)]
struct Review {
    id: i32,
    constituency: String,
    stance: String,
    content: String,
    date: String,
    upvotes: i32,
    downvotes: i32,
}

// Query parameters
#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Deserialize)]
struct ReviewForm {
    stance: String,
    content: String,
}

// Handlers
async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        recent_bills: get_recent_bills().await,
    };
    
    HtmlTemplate(template)
}

async fn search_handler(
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.query.trim();
    
    if query.is_empty() {
        return HtmlTemplate(SearchSuggestionsTemplate {
            results: vec![],
        });
    }

    // Perform vector search
    match perform_search(query).await {
        Ok(results) => HtmlTemplate(SearchSuggestionsTemplate { results }),
        Err(_) => HtmlTemplate(SearchSuggestionsTemplate {
            results: vec![],
        }),
    }
}

async fn bill_forum_handler(
    Path(bill_id): Path<i32>,
) -> impl IntoResponse {
    let bill = get_bill_info(bill_id).await;
    let reviews = get_reviews(bill_id).await;
    
    HtmlTemplate(ForumTemplate { bill, reviews })
}

async fn submit_review_handler(
    Path(bill_id): Path<i32>,
    Form(form): Form<ReviewForm>,
) -> impl IntoResponse {
    // TODO: Store review in database
    tracing::info!("Review submitted for bill {}: {:?} - {}", bill_id, form.stance, form.content);
    
    StatusCode::OK
}

async fn upvote_handler(Path(review_id): Path<i32>) -> impl IntoResponse {
    tracing::info!("Upvote for review {}", review_id);
    StatusCode::OK
}

async fn downvote_handler(Path(review_id): Path<i32>) -> impl IntoResponse {
    tracing::info!("Downvote for review {}", review_id);
    StatusCode::OK
}

// Helper functions
async fn perform_search(query: &str) -> anyhow::Result<Vec<SearchResult>> {
    // Generate embedding for query
    let query_embedding = embedder::embed_query(query).await?;
    
    // Search in Qdrant
    let search_results = vector_store::search(&query_embedding, 3).await?;
    
    // Convert to SearchResult format
    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|r| SearchResult {
            bill_id: 1, // TODO: Extract from metadata
            bill_title: r.bill_title,
            bill_number: r.bill_number,
            section: r.chunk_identifier,
            score: format!("{:.2}", r.score),
        })
        .collect();
    
    Ok(results)
}

async fn get_recent_bills() -> Vec<RecentBill> {
    // TODO: Fetch from database
    vec![
        RecentBill {
            id: 1,
            title: "The Constitution (One Hundred and Thirtieth Amendment) Bill, 2025".to_string(),
            number: "111/2025".to_string(),
            year: 2025,
        },
        RecentBill {
            id: 2,
            title: "The Jammu and Kashmir Reorganisation (Amendment) Bill, 2025".to_string(),
            number: "112/2025".to_string(),
            year: 2025,
        },
        RecentBill {
            id: 3,
            title: "The Government of Union Territories (Amendment) Bill, 2025".to_string(),
            number: "113/2025".to_string(),
            year: 2025,
        },
    ]
}

async fn get_bill_info(bill_id: i32) -> BillInfo {
    // TODO: Fetch from database
    let bills = vec![
        BillInfo {
            id: 1,
            title: "The Constitution (One Hundred and Thirtieth Amendment) Bill, 2025".to_string(),
            number: "111/2025".to_string(),
            year: 2025,
        },
        BillInfo {
            id: 2,
            title: "The Jammu and Kashmir Reorganisation (Amendment) Bill, 2025".to_string(),
            number: "112/2025".to_string(),
            year: 2025,
        },
        BillInfo {
            id: 3,
            title: "The Government of Union Territories (Amendment) Bill, 2025".to_string(),
            number: "113/2025".to_string(),
            year: 2025,
        },
    ];
    
    let default_bill = BillInfo {
        id: 1,
        title: "The Constitution (One Hundred and Thirtieth Amendment) Bill, 2025".to_string(),
        number: "111/2025".to_string(),
        year: 2025,
    };
    
    bills.into_iter()
        .find(|b| b.id == bill_id)
        .unwrap_or(default_bill)
}

async fn get_reviews(_bill_id: i32) -> Vec<Review> {
    // TODO: Fetch from database
    vec![
        Review {
            id: 1,
            constituency: "Mumbai South".to_string(),
            stance: "Support".to_string(),
            content: "This amendment addresses a critical gap in our constitutional framework. The provision for automatic removal of ministers facing serious criminal charges upholds the principles of constitutional morality and ensures accountability at the highest levels of governance.".to_string(),
            date: "November 18, 2025".to_string(),
            upvotes: 42,
            downvotes: 7,
        },
        Review {
            id: 2,
            constituency: "Delhi Central".to_string(),
            stance: "Oppose".to_string(),
            content: "While the intent is commendable, this bill raises concerns about the presumption of innocence. Arrest does not equate to conviction, and this provision could be misused for political vendetta. We need safeguards against false accusations.".to_string(),
            date: "November 17, 2025".to_string(),
            upvotes: 38,
            downvotes: 12,
        },
        Review {
            id: 3,
            constituency: "Bangalore North".to_string(),
            stance: "Critique".to_string(),
            content: "The 31-day period needs clarification. What happens during appeals? The bill should distinguish between bailable and non-bailable offenses. Additionally, there should be provisions for expedited trials to prevent indefinite limbo situations.".to_string(),
            date: "November 16, 2025".to_string(),
            upvotes: 56,
            downvotes: 3,
        },
    ]
}

// Template wrapper to handle errors
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

// Router setup
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/search", get(search_handler))
        .route("/api/bill/:id/forum", get(bill_forum_handler))
        .route("/api/bill/:id/review", post(submit_review_handler))
        .route("/api/review/:id/upvote", post(upvote_handler))
        .route("/api/review/:id/downvote", post(downvote_handler))
        .nest_service("/static", ServeDir::new("static"))
}

