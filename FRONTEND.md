# Frontend Documentation

## Overview

A formal, minimalist black-and-white civic tech interface for Indian legislation.

## Features

### 1. **Semantic Search**
- Vector search with dedicated search button
- Loading bar animation during search execution
- Search results showing top 3 relevant bills
- Displays bill title, section, and relevance score

### 2. **Recent Bills Sidebar**
- Right column showing latest parliamentary bills
- Quick access to bill forums
- Bill metadata (number, year)

### 3. **Dynamic Forum Section**
- Initially hidden, appears when bill is selected
- Displays verified constituent reviews
- Shows stance (Support/Oppose/Critique)
- Upvote/downvote functionality
- Form to submit new reviews

### 4. **Design Philosophy**
- **Formal & Professional**: Georgia serif font for gravitas
- **Black & White**: High contrast, accessible
- **Minimal**: No distractions, content-first
- **Responsive**: Clean grid layout

## Tech Stack

- **Backend**: Axum (Rust web framework)
- **Templates**: Askama (type-safe HTML templates)
- **Interactivity**: HTMX (no heavy JavaScript)
- **Static Files**: Tower-HTTP for CSS/assets

## Running the Server

```bash
# Start the web server
cargo run -- serve --port 3000

# Visit http://localhost:3000
```

## API Endpoints

### `GET /`
Home page with search and recent bills

### `GET /api/search?query=...`
Returns HTMX-rendered search suggestions (top 3 results)

### `GET /api/bill/:id/forum`
Returns forum page with reviews for a specific bill

### `POST /api/bill/:id/review`
Submit a new review (form data: stance, content)

### `POST /api/review/:id/upvote`
Upvote a review

### `POST /api/review/:id/downvote`
Downvote a review

## File Structure

```
templates/
├── base.html              # Base layout
├── index.html             # Home page
├── search_suggestions.html # Search autocomplete
└── forum.html             # Bill discussion forum

static/
└── css/
    └── main.css           # Formal B&W styling

src/
└── web.rs                 # Web routes and handlers
```

## Design Tokens

```css
--bg-primary: #ffffff       /* Pure white */
--bg-secondary: #f8f8f8     /* Off-white */
--text-primary: #000000     /* Pure black */
--border-heavy: #000000     /* Heavy borders */
--accent: #1a1a1a           /* Near-black accent */
```

## Next Steps

1. **Database Integration**: Connect to PostgreSQL for persistent storage
2. **Authentication**: Implement blind hashing for voter verification
3. **Real-time Updates**: WebSocket support for live forums
4. **Constituency Filtering**: Middleware to filter by user's constituency
5. **AI Stance Classification**: Integrate zero-shot classifier for automatic stance detection

## Dummy Data

Currently uses hardcoded data for demonstration:
- 3 recent bills from 2025
- 3 sample reviews per bill
- Mock upvote/downvote counts

Replace with database queries once PostgreSQL is integrated.

