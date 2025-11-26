use printpdf::*;
use printpdf::path::{PaintMode, WindingOrder};
use sqlx::PgPool;
use std::io::BufWriter;
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
        ORDER BY b.introduction_date DESC
        "#
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
        ORDER BY b.introduction_date DESC, p.created_at DESC
        "#
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
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "Constituency Report",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    let mut y_position = 270.0;
    
    // Title
    current_layer.use_text(
        format!("Constituency Report: {} ({})", constituency_name, state),
        18.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    
    y_position -= 10.0;
    
    current_layer.use_text(
        format!("Generated: {}", Utc::now().format("%Y-%m-%d %H:%M UTC")),
        10.0,
        Mm(20.0),
        Mm(y_position),
        &font,
    );
    
    y_position -= 15.0;
    
    // Summary section
    current_layer.use_text(
        "Summary of Constituency Stance",
        14.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    
    y_position -= 10.0;
    
    // Draw sentiment bars for each bill
    for sentiment in &sentiments {
        if y_position < 30.0 {
            // Add new page if needed
            let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
            y_position = 270.0;
        }
        
        // Bill title
        let title_text = if sentiment.bill_title.len() > 60 {
            format!("{}...", &sentiment.bill_title[..60])
        } else {
            sentiment.bill_title.clone()
        };
        
        current_layer.use_text(
            format!("{}: {}", sentiment.bill_number, title_text),
            11.0,
            Mm(20.0),
            Mm(y_position),
            &font_bold,
        );
        
        y_position -= 7.0;
        
        let total = sentiment.support_count + sentiment.oppose_count + sentiment.critique_count;
        
        if total > 0 {
            // Draw colored bars
            let bar_width = 150.0;
            let bar_height = 8.0;
            let x_start = 30.0;
            
            let support_width = (sentiment.support_count as f32 / total as f32) * bar_width;
            let oppose_width = (sentiment.oppose_count as f32 / total as f32) * bar_width;
            let critique_width = (sentiment.critique_count as f32 / total as f32) * bar_width;
            
            // Support bar (green)
            if support_width > 0.0 {
                let points = vec![
                    (Point::new(Mm(x_start), Mm(y_position)), false),
                    (Point::new(Mm(x_start + support_width), Mm(y_position)), false),
                    (Point::new(Mm(x_start + support_width), Mm(y_position - bar_height)), false),
                    (Point::new(Mm(x_start), Mm(y_position - bar_height)), false),
                ];
                
                let polygon = Polygon {
                    rings: vec![points],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                };
                
                current_layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.8, 0.2, None)));
                current_layer.add_polygon(polygon);
            }
            
            // Oppose bar (red)
            if oppose_width > 0.0 {
                let x_oppose = x_start + support_width;
                let points = vec![
                    (Point::new(Mm(x_oppose), Mm(y_position)), false),
                    (Point::new(Mm(x_oppose + oppose_width), Mm(y_position)), false),
                    (Point::new(Mm(x_oppose + oppose_width), Mm(y_position - bar_height)), false),
                    (Point::new(Mm(x_oppose), Mm(y_position - bar_height)), false),
                ];
                
                let polygon = Polygon {
                    rings: vec![points],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                };
                
                current_layer.set_fill_color(Color::Rgb(Rgb::new(0.9, 0.2, 0.2, None)));
                current_layer.add_polygon(polygon);
            }
            
            // Critique bar (yellow)
            if critique_width > 0.0 {
                let x_critique = x_start + support_width + oppose_width;
                let points = vec![
                    (Point::new(Mm(x_critique), Mm(y_position)), false),
                    (Point::new(Mm(x_critique + critique_width), Mm(y_position)), false),
                    (Point::new(Mm(x_critique + critique_width), Mm(y_position - bar_height)), false),
                    (Point::new(Mm(x_critique), Mm(y_position - bar_height)), false),
                ];
                
                let polygon = Polygon {
                    rings: vec![points],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                };
                
                current_layer.set_fill_color(Color::Rgb(Rgb::new(0.9, 0.8, 0.2, None)));
                current_layer.add_polygon(polygon);
            }
            
            y_position -= bar_height + 5.0;
            
            // Legend
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            current_layer.use_text(
                format!("Support: {} | Oppose: {} | Critique: {}", 
                    sentiment.support_count, sentiment.oppose_count, sentiment.critique_count),
                9.0,
                Mm(30.0),
                Mm(y_position),
                &font,
            );
        } else {
            current_layer.use_text(
                "No posts yet",
                9.0,
                Mm(30.0),
                Mm(y_position),
                &font,
            );
        }
        
        y_position -= 12.0;
    }
    
    // Detailed posts section
    y_position -= 10.0;
    
    if y_position < 30.0 {
        let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
        let _current_layer = doc.get_page(page_idx).get_layer(layer_idx);
        y_position = 270.0;
    }
    
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    current_layer.use_text(
        "Detailed Posts by Bill",
        14.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    
    y_position -= 10.0;
    
    let mut current_bill_id: Option<uuid::Uuid> = None;
    
    for post in &posts {
        if y_position < 40.0 {
            let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
            y_position = 270.0;
        }
        
        // New bill section
        if current_bill_id != Some(post.bill_id) {
            current_bill_id = Some(post.bill_id);
            
            y_position -= 5.0;
            
            let title_text = if post.bill_title.len() > 70 {
                format!("{}...", &post.bill_title[..70])
            } else {
                post.bill_title.clone()
            };
            
            let current_layer = doc.get_page(page1).get_layer(layer1);
            current_layer.use_text(
                format!("{}: {}", post.bill_number, title_text),
                12.0,
                Mm(20.0),
                Mm(y_position),
                &font_bold,
            );
            
            y_position -= 8.0;
        }
        
        // Post details
        let current_layer = doc.get_page(page1).get_layer(layer1);
        current_layer.use_text(
            format!("@{} - {} (↑{} ↓{})", 
                post.username, 
                post.stance.to_uppercase(),
                post.upvotes,
                post.downvotes
            ),
            9.0,
            Mm(25.0),
            Mm(y_position),
            &font_bold,
        );
        
        y_position -= 5.0;
        
        // Wrap post content
        let content_lines = wrap_text(&post.content, 85);
        for line in content_lines {
            if y_position < 30.0 {
                let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                let current_layer = doc.get_page(page_idx).get_layer(layer_idx);
                y_position = 270.0;
            }
            
            let current_layer = doc.get_page(page1).get_layer(layer1);
            current_layer.use_text(
                line,
                9.0,
                Mm(25.0),
                Mm(y_position),
                &font,
            );
            
            y_position -= 5.0;
        }
        
        y_position -= 3.0;
    }
    
    // Save to buffer
    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)?;
    
    Ok(buffer.into_inner()?)
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_chars {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

