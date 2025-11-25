use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use askama::Template;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::services::ServeDir;
use uuid::Uuid;

use crate::{auth, db, embedder, moderation, models, rate_limit, vector_store};

const SESSION_COOKIE_NAME: &str = "session_token";

// Application State
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

// Templates
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    recent_bills: Vec<RecentBill>,
    current_page: i64,
    total_pages: i64,
    has_prev: bool,
    has_next: bool,
    user: Option<CurrentUser>,
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
    user: Option<CurrentUser>,
    rate_limit_remaining: i64,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: Option<String>,
    constituencies: Vec<ConstituencyOption>,
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {
    profile: ProfileData,
    posts: Vec<UserPost>,
    is_own_profile: bool,
    user: Option<CurrentUser>,
    constituencies: Vec<ConstituencyOption>,
}

#[derive(Template)]
#[template(path = "bills_list.html")]
struct BillsListTemplate {
    bills: Vec<RecentBill>,
    current_page: i64,
    total_pages: i64,
    has_prev: bool,
    has_next: bool,
}

// Data structures
#[derive(Clone, Serialize)]
struct CurrentUser {
    id: String,
    username: String,
}

#[derive(Clone, Serialize)]
struct RecentBill {
    id: String,
    title: String,
    number: String,
    year: i32,
}

#[derive(Clone, Serialize)]
struct SearchResult {
    bill_id: String,
    bill_title: String,
    bill_number: String,
    section: String,
    score: String,
}

#[derive(Clone, Serialize)]
struct BillInfo {
    id: String,
    title: String,
    number: String,
    year: i32,
}

#[derive(Clone, Serialize)]
struct Review {
    id: String,
    username: String,
    constituency: String,
    stance: String,
    content: String,
    date: String,
    upvotes: i32,
    downvotes: i32,
}

#[derive(Clone, Serialize)]
struct ConstituencyOption {
    id: i32,
    name: String,
    state: String,
}

#[derive(Clone, Serialize)]
struct ProfileData {
    username: String,
    real_name: Option<String>,
    age: Option<i32>,
    gender: Option<String>,
    pincode: Option<String>,
    constituency_id: i32,  // 0 if not set
    constituency_name: Option<String>,
    member_since: String,
    post_count: i64,
}

#[derive(Clone, Serialize)]
struct UserPost {
    id: String,
    bill_title: String,
    bill_number: String,
    stance: String,
    content: String,
    moderation_status: String,
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
struct PaginationQuery {
    page: Option<i64>,
}

#[derive(Deserialize)]
struct ReviewForm {
    stance: String,
    content: String,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    real_name: Option<String>,
    age: Option<String>,
    gender: Option<String>,
    location_type: String, // "pincode" or "constituency"
    pincode: Option<String>,
    constituency_id: Option<String>,
}

#[derive(Deserialize)]
struct ProfileUpdateForm {
    real_name: Option<String>,
    age: Option<String>,
    gender: Option<String>,
    location_type: String,
    pincode: Option<String>,
    constituency_id: Option<String>,
}

// Helper to get current user from session
async fn get_current_user(jar: &CookieJar, pool: &PgPool) -> Option<models::User> {
    let session_token = jar.get(SESSION_COOKIE_NAME)?.value().to_string();
    auth::get_user_by_session(pool, &session_token).await.ok()?
}

// Handlers
async fn index(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = 5;

    let user = get_current_user(&jar, &state.db_pool).await;
    let current_user = user.map(|u| CurrentUser {
        id: u.id.to_string(),
        username: u.username,
    });

    let (bills, total) = db::get_bills_paginated(&state.db_pool, page, per_page)
        .await
        .unwrap_or((vec![], 0));

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    let recent_bills: Vec<RecentBill> = bills
        .into_iter()
        .map(|b| RecentBill {
            id: b.id.to_string(),
            title: b.title,
            number: b.bill_number,
            year: b.year,
        })
        .collect();

    let template = IndexTemplate {
        recent_bills,
        current_page: page,
        total_pages,
        has_prev: page > 1,
        has_next: page < total_pages,
        user: current_user,
    };

    HtmlTemplate(template)
}

async fn bills_list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = 5;

    let (bills, total) = db::get_bills_paginated(&state.db_pool, page, per_page)
        .await
        .unwrap_or((vec![], 0));

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    let bills: Vec<RecentBill> = bills
        .into_iter()
        .map(|b| RecentBill {
            id: b.id.to_string(),
            title: b.title,
            number: b.bill_number,
            year: b.year,
        })
        .collect();

    HtmlTemplate(BillsListTemplate {
        bills,
        current_page: page,
        total_pages,
        has_prev: page > 1,
        has_next: page < total_pages,
    })
}

async fn search_handler(Query(params): Query<SearchQuery>) -> impl IntoResponse {
    let query = params.query.trim();

    if query.is_empty() {
        return HtmlTemplate(SearchSuggestionsTemplate { results: vec![] });
    }

    match perform_search(query).await {
        Ok(results) => HtmlTemplate(SearchSuggestionsTemplate { results }),
        Err(_) => HtmlTemplate(SearchSuggestionsTemplate { results: vec![] }),
    }
}

async fn bill_forum_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(bill_id): Path<String>,
) -> impl IntoResponse {
    let bill_uuid = match Uuid::parse_str(&bill_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid bill ID").into_response();
        }
    };

    let user = get_current_user(&jar, &state.db_pool).await;
    let current_user = user.as_ref().map(|u| CurrentUser {
        id: u.id.to_string(),
        username: u.username.clone(),
    });

    let rate_limit_remaining = if let Some(ref u) = user {
        rate_limit::get_remaining_posts(&state.db_pool, u.id)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let bill = match db::get_bill_by_id(&state.db_pool, bill_uuid).await {
        Ok(Some(b)) => BillInfo {
            id: b.id.to_string(),
            title: b.title,
            number: b.bill_number,
            year: b.year,
        },
        _ => {
            return (StatusCode::NOT_FOUND, "Bill not found").into_response();
        }
    };

    let posts = db::get_posts_for_bill(&state.db_pool, bill_uuid)
        .await
        .unwrap_or_default();

    let reviews: Vec<Review> = posts
        .into_iter()
        .map(|p| Review {
            id: p.id.to_string(),
            username: p.username,
            constituency: p.constituency_name.unwrap_or_else(|| "Unknown".to_string()),
            stance: p.stance,
            content: p.content,
            date: p.formatted_date,
            upvotes: p.upvotes,
            downvotes: p.downvotes,
        })
        .collect();

    HtmlTemplate(ForumTemplate {
        bill,
        reviews,
        user: current_user,
        rate_limit_remaining,
    })
    .into_response()
}

async fn submit_review_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(bill_id): Path<String>,
    Form(form): Form<ReviewForm>,
) -> impl IntoResponse {
    let user = match get_current_user(&jar, &state.db_pool).await {
        Some(u) => u,
        None => {
            return (StatusCode::UNAUTHORIZED, "Please log in to submit a review").into_response();
        }
    };

    let bill_uuid = match Uuid::parse_str(&bill_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid bill ID").into_response();
        }
    };

    // Check rate limit
    if !rate_limit::can_create_post(&state.db_pool, user.id)
        .await
        .unwrap_or(false)
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please wait before posting again.",
        )
            .into_response();
    }

    // Moderate content
    let moderation_result = moderation::check_content(&form.content)
        .await
        .unwrap_or(models::ModerationResult::AdminReview);

    let moderation_status = moderation_result.to_status();
    let moderation_reason = match moderation_result {
        models::ModerationResult::Popcorn => Some("Content rejected by moderation"),
        models::ModerationResult::AdminReview => Some("Pending admin review"),
        _ => None,
    };

    // Normalize stance
    let stance = match form.stance.to_lowercase().as_str() {
        "support" => "Support",
        "oppose" => "Oppose",
        "critique" => "Critique",
        _ => "Critique",
    };

    // Create post
    match db::create_post(
        &state.db_pool,
        user.id,
        bill_uuid,
        stance,
        &form.content,
        moderation_status,
        moderation_reason,
    )
    .await
    {
        Ok(_) => {
            // Record rate limit action
            let _ = rate_limit::record_post_action(&state.db_pool, user.id).await;

            let message = match moderation_result {
                models::ModerationResult::Falafel => "Review submitted successfully!",
                models::ModerationResult::Popcorn => {
                    "Your review was rejected due to content policy violation."
                }
                models::ModerationResult::AdminReview => {
                    "Your review is pending admin approval."
                }
            };

            (StatusCode::OK, message).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create post: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to submit review").into_response()
        }
    }
}

async fn upvote_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    let user = match get_current_user(&jar, &state.db_pool).await {
        Some(u) => u,
        None => {
            return StatusCode::UNAUTHORIZED;
        }
    };

    let post_uuid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return StatusCode::BAD_REQUEST;
        }
    };

    match db::upvote_post(&state.db_pool, post_uuid, user.id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn downvote_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    let user = match get_current_user(&jar, &state.db_pool).await {
        Some(u) => u,
        None => {
            return StatusCode::UNAUTHORIZED;
        }
    };

    let post_uuid = match Uuid::parse_str(&review_id) {
        Ok(id) => id,
        Err(_) => {
            return StatusCode::BAD_REQUEST;
        }
    };

    match db::downvote_post(&state.db_pool, post_uuid, user.id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// Auth handlers
async fn login_page() -> impl IntoResponse {
    HtmlTemplate(LoginTemplate { error: None })
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    match auth::authenticate_user(&state.db_pool, &form.username, &form.password).await {
        Ok(Some(user)) => {
            match auth::create_session(&state.db_pool, user.id).await {
                Ok(session) => {
                    let cookie = Cookie::build((SESSION_COOKIE_NAME, session.session_token))
                        .path("/")
                        .http_only(true)
                        .max_age(time::Duration::days(7))
                        .build();

                    (jar.add(cookie), Redirect::to("/")).into_response()
                }
                Err(_) => HtmlTemplate(LoginTemplate {
                    error: Some("Failed to create session".to_string()),
                })
                .into_response(),
            }
        }
        Ok(None) => HtmlTemplate(LoginTemplate {
            error: Some("Invalid username or password".to_string()),
        })
        .into_response(),
        Err(_) => HtmlTemplate(LoginTemplate {
            error: Some("An error occurred".to_string()),
        })
        .into_response(),
    }
}

async fn register_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let constituencies = auth::get_all_constituencies(&state.db_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|c| ConstituencyOption {
            id: c.id,
            name: c.name,
            state: c.state,
        })
        .collect();

    HtmlTemplate(RegisterTemplate {
        error: None,
        constituencies,
    })
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    let constituencies = auth::get_all_constituencies(&state.db_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|c| ConstituencyOption {
            id: c.id,
            name: c.name,
            state: c.state,
        })
        .collect::<Vec<_>>();

    // Validate username
    if form.username.is_empty() {
        return HtmlTemplate(RegisterTemplate {
            error: Some("Username is required".to_string()),
            constituencies,
        })
        .into_response();
    }

    // Check if username exists
    if auth::username_exists(&state.db_pool, &form.username)
        .await
        .unwrap_or(false)
    {
        return HtmlTemplate(RegisterTemplate {
            error: Some("Username already taken".to_string()),
            constituencies,
        })
        .into_response();
    }

    // Parse age if provided
    let age: Option<i32> = form.age.as_ref().and_then(|a| a.parse().ok());

    // Determine pincode and constituency based on location_type
    let (pincode, constituency_id) = if form.location_type == "pincode" {
        (form.pincode.clone(), None)
    } else {
        (None, form.constituency_id.as_ref().and_then(|c| c.parse().ok()))
    };

    match auth::create_user(
        &state.db_pool,
        &form.username,
        &form.password,
        form.real_name.as_deref(),
        age,
        form.gender.as_deref(),
        pincode.as_deref(),
        constituency_id,
    )
    .await
    {
        Ok(user) => {
            match auth::create_session(&state.db_pool, user.id).await {
                Ok(session) => {
                    let cookie = Cookie::build((SESSION_COOKIE_NAME, session.session_token))
                        .path("/")
                        .http_only(true)
                        .max_age(time::Duration::days(7))
                        .build();

                    (jar.add(cookie), Redirect::to("/")).into_response()
                }
                Err(_) => Redirect::to("/login").into_response(),
            }
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            HtmlTemplate(RegisterTemplate {
                error: Some("Failed to create account".to_string()),
                constituencies,
            })
            .into_response()
        }
    }
}

async fn logout_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let _ = auth::delete_session(&state.db_pool, cookie.value()).await;
    }

    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    (jar.remove(cookie), Redirect::to("/"))
}

// Profile handlers
async fn profile_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let current_user = get_current_user(&jar, &state.db_pool).await;
    let is_own_profile = current_user
        .as_ref()
        .map(|u| u.username == username)
        .unwrap_or(false);

    let profile = match db::get_user_profile(&state.db_pool, &username).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading profile").into_response();
        }
    };

    // Get constituency_id for the profile
    let constituency_id = if let Some(ref u) = current_user {
        if u.username == username {
            u.constituency_id.unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    let posts = db::get_posts_by_user(&state.db_pool, profile.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(post, bill_title, bill_number)| UserPost {
            id: post.id.to_string(),
            bill_title,
            bill_number,
            stance: post.stance,
            content: post.content,
            moderation_status: post.moderation_status,
            date: post.created_at.format("%B %d, %Y").to_string(),
            upvotes: post.upvotes,
            downvotes: post.downvotes,
        })
        .collect();

    let constituencies = if is_own_profile {
        auth::get_all_constituencies(&state.db_pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|c| ConstituencyOption {
                id: c.id,
                name: c.name,
                state: c.state,
            })
            .collect()
    } else {
        vec![]
    };

    let profile_data = ProfileData {
        username: profile.username,
        real_name: profile.real_name,
        age: profile.age,
        gender: profile.gender,
        pincode: profile.pincode,
        constituency_id,
        constituency_name: profile.constituency_name,
        member_since: profile.created_at.format("%B %Y").to_string(),
        post_count: profile.post_count,
    };

    HtmlTemplate(ProfileTemplate {
        profile: profile_data,
        posts,
        is_own_profile,
        user: current_user.map(|u| CurrentUser {
            id: u.id.to_string(),
            username: u.username,
        }),
        constituencies,
    })
    .into_response()
}

async fn update_profile_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(username): Path<String>,
    Form(form): Form<ProfileUpdateForm>,
) -> impl IntoResponse {
    let user = match get_current_user(&jar, &state.db_pool).await {
        Some(u) => u,
        None => {
            return Redirect::to("/login").into_response();
        }
    };

    // Only allow editing own profile
    if user.username != username {
        return (StatusCode::FORBIDDEN, "Cannot edit another user's profile").into_response();
    }

    let age: Option<i32> = form.age.as_ref().and_then(|a| a.parse().ok());

    let (pincode, constituency_id) = if form.location_type == "pincode" {
        (form.pincode.clone(), None)
    } else {
        (None, form.constituency_id.as_ref().and_then(|c| c.parse().ok()))
    };

    match auth::update_user_profile(
        &state.db_pool,
        user.id,
        form.real_name.as_deref(),
        age,
        form.gender.as_deref(),
        pincode.as_deref(),
        constituency_id,
    )
    .await
    {
        Ok(_) => Redirect::to(&format!("/u/{}", username)).into_response(),
        Err(e) => {
            tracing::error!("Failed to update profile: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update profile").into_response()
        }
    }
}

// Helper functions
async fn perform_search(query: &str) -> anyhow::Result<Vec<SearchResult>> {
    let query_embedding = embedder::embed_query(query).await?;
    let search_results = vector_store::search(&query_embedding, 3).await?;

    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|r| SearchResult {
            bill_id: "".to_string(), // TODO: Extract from metadata
            bill_title: r.bill_title,
            bill_number: r.bill_number,
            section: r.chunk_identifier,
            score: format!("{:.2}", r.score),
        })
        .collect();

    Ok(results)
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
pub async fn create_router() -> Router {
    let db_pool = db::create_pool().await.expect("Failed to create database pool");

    let state = Arc::new(AppState { db_pool });

    Router::new()
        // Main pages
        .route("/", get(index))
        .route("/login", get(login_page).post(login_handler))
        .route("/register", get(register_page).post(register_handler))
        .route("/logout", get(logout_handler))
        .route("/u/:username", get(profile_handler).post(update_profile_handler))
        // API endpoints
        .route("/api/search", get(search_handler))
        .route("/api/bills", get(bills_list_handler))
        .route("/api/bill/:id/forum", get(bill_forum_handler))
        .route("/api/bill/:id/review", post(submit_review_handler))
        .route("/api/review/:id/upvote", post(upvote_handler))
        .route("/api/review/:id/downvote", post(downvote_handler))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
