# Web Module Explanation (`src/web.rs`)

This document explains the structure and functionality of the `web.rs` module, which serves as the web server and HTTP request handler for the Representation Upon Enigma civic legislation platform.

## Overview

The `web.rs` module is built using the **Axum** web framework and handles all HTTP routes, user authentication, forum interactions, and PDF report generation for Members of Parliament (MPs).

---

## Core Components

### 1. Application State

```rust
pub struct AppState {
    pub db_pool: PgPool,
}
```

Shared application state containing the PostgreSQL database connection pool, accessible across all route handlers.

---

## Templates

The module uses **Askama** for server-side HTML templating. Each template struct corresponds to a page or component:

### Page Templates
- **`IndexTemplate`**: Homepage with recent bills and pagination
- **`ForumTemplate`**: HTMX partial for forum content (loaded dynamically)
- **`ForumPageTemplate`**: Full standalone forum page with sidebar
- **`LoginTemplate`**: User login page
- **`RegisterTemplate`**: User registration page with constituency selection
- **`ProfileTemplate`**: User profile page (editable for owner, viewable for others)
- **`BillsListTemplate`**: HTMX partial for paginated bills list
- **`SearchSuggestionsTemplate`**: HTMX partial for search results

---

## Data Structures

### User & Authentication
- **`CurrentUser`**: Minimal user info (id, username) for session tracking
- **`LoginForm`**: Login credentials (username, password)
- **`RegisterForm`**: Registration data including optional demographics and location
- **`ProfileUpdateForm`**: Profile editing form data

### Bills & Forum
- **`RecentBill`**: Bill summary for listings (id, title, number, year)
- **`BillInfo`**: Detailed bill information for forum pages
- **`Review`**: Forum post/review with user info, stance, votes
- **`UserPost`**: User's post history with bill context

### Location
- **`ConstituencyOption`**: Constituency data for dropdowns (id, name, state)
- **`ProfileData`**: Complete user profile with location and statistics

### Query Parameters
- **`SearchQuery`**: Search term from user input
- **`PaginationQuery`**: Page number for paginated content
- **`ReviewForm`**: New forum post submission (stance, content)
- **`MPReportQuery`**: Constituency ID for PDF report generation

---

## Route Handlers

### Main Pages

#### `index()` - Homepage
- **Route**: `GET /`
- **Function**: Displays homepage with paginated recent bills
- **Features**:
  - Fetches 5 bills per page
  - Shows current user if logged in
  - Provides pagination controls

#### `bills_list_handler()` - Bills List Partial
- **Route**: `GET /api/bills?page=N`
- **Function**: HTMX endpoint for loading paginated bills
- **Returns**: HTML partial with bill list and pagination

---

### Search

#### `search_handler()` - Semantic Search
- **Route**: `GET /api/search?query=...`
- **Function**: Performs vector-based semantic search across bill content
- **Process**:
  1. Generates embedding for search query
  2. Searches Qdrant vector database
  3. Looks up bill UUIDs from results
  4. Returns HTML partial with clickable results

---

### Forum Pages

#### `bill_forum_handler()` - Forum Partial (HTMX)
- **Route**: `GET /api/bill/:id/forum`
- **Function**: Loads forum content into homepage sidebar
- **Features**:
  - Displays bill details
  - Shows all approved posts with votes
  - Checks rate limits for posting
  - Provides post submission form

#### `forum_page_handler()` - Standalone Forum Page
- **Route**: `GET /f/:bill_id`
- **Function**: Full forum page with sidebar (persists on refresh)
- **Features**:
  - Same content as forum partial
  - Includes recent bills sidebar and search
  - Maintains URL on page reload

#### `submit_review_handler()` - Post Submission
- **Route**: `POST /api/bill/:id/review`
- **Function**: Creates new forum post with moderation
- **Process**:
  1. Validates user authentication
  2. Checks rate limit (posts per hour)
  3. Runs AI moderation (Ollama/Llama 3.2)
  4. Normalizes stance (Support/Oppose/Critique)
  5. Stores post with moderation status
  6. Records rate limit action
- **Moderation Results**:
  - **Falafel** → Approved immediately
  - **Popcorn** → Rejected (spam/toxic)
  - **Default** → Pending admin review

---

### Voting System

#### `upvote_handler()` & `downvote_handler()`
- **Routes**: 
  - `POST /api/review/:id/upvote`
  - `POST /api/review/:id/downvote`
- **Function**: Toggle vote on forum posts
- **Features**:
  - Prevents multiple votes from same user
  - Allows changing vote (upvote ↔ downvote)
  - Returns updated vote buttons with current counts
  - Uses HTMX for seamless updates without page reload

---

### Authentication

#### `login_page()` & `login_handler()`
- **Routes**: 
  - `GET /login` - Display login form
  - `POST /login` - Process login
- **Function**: User authentication with Argon2 password hashing
- **Process**:
  1. Validates credentials
  2. Creates session with 7-day expiry
  3. Sets HTTP-only session cookie
  4. Redirects to homepage

#### `register_page()` & `register_handler()`
- **Routes**:
  - `GET /register` - Display registration form
  - `POST /register` - Process registration
- **Function**: New user account creation
- **Features**:
  - Username uniqueness check
  - Optional demographics (real name, age, gender)
  - Location via pincode OR constituency dropdown
  - Automatic login after registration

#### `logout_handler()`
- **Route**: `GET /logout`
- **Function**: Destroys session and clears cookie

---

### User Profiles

#### `profile_handler()`
- **Route**: `GET /u/:username`
- **Function**: Display user profile page
- **Features**:
  - Shows user info and post history
  - Editable form for profile owner
  - Read-only view for other users
  - Lists all posts with moderation status

#### `update_profile_handler()`
- **Route**: `POST /u/:username`
- **Function**: Updates user profile information
- **Security**: Only allows editing own profile

---

### MP Dashboard (PDF Reports)

#### `constituencies_handler()`
- **Route**: `GET /api/constituencies`
- **Function**: Returns JSON list of all constituencies
- **Used By**: Modal dropdown for MP constituency selection

#### `mp_report_handler()`
- **Route**: `GET /api/mp/report?constituency_id=N`
- **Function**: Generates comprehensive PDF report for MPs
- **Report Contents**:
  1. **Summary Section**: Colored bar graphs showing Support/Oppose/Critique counts per bill
  2. **Detailed Posts**: All constituency posts organized by bill with:
     - Username and stance
     - Post content
     - Upvote/downvote counts
- **Output**: PDF file download

---

## Helper Functions

### `get_current_user()`
- Extracts session token from cookie
- Looks up user in database
- Returns `Option<User>` (None if not logged in)

### `perform_search()`
- Generates query embedding using Ollama
- Searches Qdrant vector store
- Maps results to bill UUIDs
- Returns structured search results

---

## Router Configuration

The `create_router()` function sets up all routes:

```rust
Router::new()
    // Main pages
    .route("/", get(index))
    .route("/login", get(login_page).post(login_handler))
    .route("/register", get(register_page).post(register_handler))
    .route("/logout", get(logout_handler))
    .route("/u/:username", get(profile_handler).post(update_profile_handler))
    
    // Forum pages
    .route("/f/:bill_id", get(forum_page_handler))
    
    // API endpoints
    .route("/api/search", get(search_handler))
    .route("/api/bills", get(bills_list_handler))
    .route("/api/bill/:id/forum", get(bill_forum_handler))
    .route("/api/bill/:id/review", post(submit_review_handler))
    .route("/api/review/:id/upvote", post(upvote_handler))
    .route("/api/review/:id/downvote", post(downvote_handler))
    .route("/api/constituencies", get(constituencies_handler))
    .route("/api/mp/report", get(mp_report_handler))
    
    // Static files (CSS, JS)
    .nest_service("/static", ServeDir::new("static"))
```

---

## Key Features

### 1. **Session Management**
- HTTP-only cookies for security
- 7-day session expiry
- Automatic cleanup of expired sessions

### 2. **Rate Limiting**
- Prevents forum spam
- Configurable posts per hour limit
- Per-user tracking in database

### 3. **Content Moderation**
- AI-powered moderation using Ollama/Llama 3.2
- Three-tier system: Approved, Rejected, Pending Review
- Fallback keyword-based moderation

### 4. **HTMX Integration**
- Dynamic content loading without page reloads
- Partial HTML responses for efficiency
- Smooth user experience with minimal JavaScript

### 5. **Pagination**
- Server-side pagination for bills
- 5 items per page
- Previous/Next navigation

### 6. **Semantic Search**
- Vector-based search using embeddings
- Searches across all bill content
- Relevance-scored results

### 7. **Voting System**
- One vote per user per post
- Vote switching allowed (upvote ↔ downvote)
- Real-time updates via HTMX

### 8. **PDF Generation**
- Comprehensive constituency reports for MPs
- Visual sentiment graphs (colored bars)
- Detailed post listings
- Professional formatting

---

## Dependencies

- **Axum**: Web framework
- **Askama**: Template engine
- **SQLx**: Database access (PostgreSQL)
- **Axum-extra**: Cookie handling
- **Tower-http**: Static file serving
- **Serde**: JSON serialization
- **UUID**: Unique identifiers

---

## Security Considerations

1. **Password Hashing**: Argon2 for secure password storage
2. **HTTP-only Cookies**: Prevents XSS attacks on session tokens
3. **CSRF Protection**: Form-based authentication
4. **Input Validation**: Username uniqueness, required fields
5. **Authorization Checks**: Users can only edit own profiles
6. **SQL Injection Prevention**: Parameterized queries via SQLx

---

## Error Handling

- Database errors return appropriate HTTP status codes
- Template rendering errors show user-friendly messages
- Invalid UUIDs return 400 Bad Request
- Missing resources return 404 Not Found
- Authentication failures redirect to login page

---

## Future Enhancements

Potential areas for expansion:
- Admin panel for moderation review
- Email verification for registration
- Password reset functionality
- Advanced search filters
- Real-time notifications
- API rate limiting
- CSRF token validation
- Two-factor authentication

