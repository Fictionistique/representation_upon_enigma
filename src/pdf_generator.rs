use genpdf::elements;
use genpdf::fonts;
use genpdf::style;
use genpdf::Element;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct BillSentiment {
    pub bill_id: uuid::Uuid,
    pub bill_number: String,
    pub bill_title: String,
    pub introduction_date: Option<DateTime<Utc>>,
    pub support_count: i64,
    pub oppose_count: i64,
    pub critique_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ConstituencyPost {
    pub bill_id: uuid::Uuid,
    pub bill_number: String,
    pub bill_title: String,
    pub username: String,
    pub content: String,
    pub stance: String,
    pub created_at: DateTime<Utc>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub introduction_date: Option<chrono::NaiveDate>,
}

pub async fn get_constituency_sentiment(
    pool: &PgPool,
    constituency_id: i32,
) -> Result<Vec<BillSentiment>, sqlx::Error> {
    sqlx::query_as::<_, BillSentiment>(
        r#"
        SELECT 
            b.id as bill_id,
            b.bill_number,
            b.title as bill_title,
            b.introduction_date,
            COUNT(CASE WHEN p.stance = 'Support' THEN 1 END) as support_count,
            COUNT(CASE WHEN p.stance = 'Oppose' THEN 1 END) as oppose_count,
            COUNT(CASE WHEN p.stance = 'Critique' THEN 1 END) as critique_count
        FROM bills b
        INNER JOIN posts p ON b.id = p.bill_id
        INNER JOIN users u ON p.user_id = u.id
        WHERE u.constituency_id = $1
        AND p.moderation_status = 'approved'
        GROUP BY b.id, b.bill_number, b.title, b.introduction_date
        HAVING COUNT(p.id) > 0
        ORDER BY b.introduction_date DESC
        "#,
    )
    .bind(constituency_id)
    .fetch_all(pool)
    .await
}

pub async fn get_constituency_posts(
    pool: &PgPool,
    constituency_id: i32,
) -> Result<Vec<ConstituencyPost>, sqlx::Error> {
    sqlx::query_as::<_, ConstituencyPost>(
        r#"
        SELECT 
            b.id as bill_id,
            b.bill_number,
            b.title as bill_title,
            b.introduction_date,
            u.username,
            p.content,
            p.stance,
            p.created_at,
            p.upvotes,
            p.downvotes
        FROM posts p
        INNER JOIN users u ON p.user_id = u.id
        INNER JOIN bills b ON p.bill_id = b.id
        WHERE u.constituency_id = $1
        AND p.moderation_status = 'approved'
        ORDER BY b.bill_number, p.created_at DESC
        "#,
    )
    .bind(constituency_id)
    .fetch_all(pool)
    .await
}

pub fn generate_constituency_report(
    constituency_name: &str,
    state: &str,
    sentiments: Vec<BillSentiment>,
    posts: Vec<ConstituencyPost>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Embed Liberation Sans font (public domain)
    // Download from: https://github.com/liberationfonts/liberation-fonts/releases
    let font_data = include_bytes!("../LiberationSans-Regular.ttf");
    let regular = fonts::FontData::new(font_data.to_vec(), None)?;
    
    let font_family = fonts::FontFamily {
        regular: regular.clone(),
        bold: regular.clone(),
        italic: regular.clone(),
        bold_italic: regular,
    };
    
    // Create document
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(format!("Constituency Report: {} ({})", constituency_name, state));
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);
    
    // Set page margins
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);
    
    // Title
    doc.push(
        elements::Paragraph::new(format!("Constituency Report: {} ({})", constituency_name, state))
            .styled(style::Style::new().bold().with_font_size(18))
    );
    
    doc.push(elements::Break::new(0.5));
    
    doc.push(
        elements::Paragraph::new(format!("Generated: {}", Utc::now().format("%Y-%m-%d %H:%M UTC")))
            .styled(style::Style::new().with_font_size(10))
    );
    
    doc.push(elements::Break::new(1.0));
    
    // Summary section
    doc.push(
        elements::Paragraph::new("Summary of Constituency Stance")
            .styled(style::Style::new().bold().with_font_size(14))
    );
    
    doc.push(elements::Break::new(0.5));
    
    // Sentiment bars for each bill
    for sentiment in &sentiments {
        let title_text = if sentiment.bill_title.len() > 70 {
            format!("{}...", &sentiment.bill_title[..70])
        } else {
            sentiment.bill_title.clone()
        };
        
        doc.push(
            elements::Paragraph::new(format!("{}: {}", sentiment.bill_number, title_text))
                .styled(style::Style::new().bold().with_font_size(11))
        );
        
        let total = sentiment.support_count + sentiment.oppose_count + sentiment.critique_count;
        
        if total > 0 {
            doc.push(
                elements::Paragraph::new(format!(
                    "  Support: {} | Oppose: {} | Critique: {}",
                    sentiment.support_count, sentiment.oppose_count, sentiment.critique_count
                ))
                .styled(style::Style::new().with_font_size(9))
            );
            
            // Create a simple text-based bar representation
            let support_pct = (sentiment.support_count as f32 / total as f32 * 100.0) as i32;
            let oppose_pct = (sentiment.oppose_count as f32 / total as f32 * 100.0) as i32;
            let critique_pct = 100 - support_pct - oppose_pct;
            
            doc.push(
                elements::Paragraph::new(format!(
                    "  [{}% Support | {}% Oppose | {}% Critique]",
                    support_pct, oppose_pct, critique_pct
                ))
                .styled(style::Style::new().with_font_size(9))
            );
        } else {
            doc.push(
                elements::Paragraph::new("  No posts yet")
                    .styled(style::Style::new().with_font_size(9))
            );
        }
        
        doc.push(elements::Break::new(0.5));
    }
    
    doc.push(elements::Break::new(1.0));
    
    // Detailed posts section
    doc.push(
        elements::Paragraph::new("Detailed Posts by Bill")
            .styled(style::Style::new().bold().with_font_size(14))
    );
    
    doc.push(elements::Break::new(0.5));
    
    let mut current_bill_id: Option<uuid::Uuid> = None;
    
    for post in &posts {
        // New bill section
        if current_bill_id != Some(post.bill_id) {
            current_bill_id = Some(post.bill_id);
            
            doc.push(elements::Break::new(0.5));
            
            let title_text = if post.bill_title.len() > 80 {
                format!("{}...", &post.bill_title[..80])
            } else {
                post.bill_title.clone()
            };
            
            doc.push(
                elements::Paragraph::new(format!("{}: {}", post.bill_number, title_text))
                    .styled(style::Style::new().bold().with_font_size(12))
            );
            
            doc.push(elements::Break::new(0.3));
        }
        
        // Post header
        doc.push(
            elements::Paragraph::new(format!(
                "@{} - {} (↑{} ↓{})",
                post.username,
                post.stance.to_uppercase(),
                post.upvotes,
                post.downvotes
            ))
            .styled(style::Style::new().bold().with_font_size(9))
        );
        
        // Post content
        doc.push(
            elements::Paragraph::new(format!("  {}", post.content))
                .styled(style::Style::new().with_font_size(9))
        );
        
        doc.push(elements::Break::new(0.3));
    }
    
    // Render to bytes
    let mut buffer = Vec::new();
    doc.render(&mut buffer)?;
    Ok(buffer)
}
