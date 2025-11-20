use anyhow::{Context, Result};
use scraper::{Html, Selector};
use crate::models::Bill;

/// Fetches recent bills from PRS India website
pub async fn fetch_recent_bills(count: usize) -> Result<Vec<Bill>> {
    tracing::info!("Fetching bills from PRS Legislative Research...");
    
    // PRS India's bill tracking page
    let url = "https://prsindia.org/billtrack";
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch PRS bills page")?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error {}: Failed to fetch bills", response.status());
    }
    
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    
    // Parse the HTML to extract bill information
    let bills = parse_bills_from_html(&document, count, &client).await?;
    
    if bills.is_empty() {
        anyhow::bail!("No bills found on PRS website. The page structure may have changed.");
    }
    
    tracing::info!("Successfully fetched {} bills from PRS", bills.len());
    Ok(bills)
}

async fn parse_bills_from_html(document: &Html, count: usize, client: &reqwest::Client) -> Result<Vec<Bill>> {
    let mut bills = Vec::new();
    
    // PRS India structure: bills are in h3 tags with links to bill pages
    let h3_selector = Selector::parse("h3").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    
    for h3 in document.select(&h3_selector).take(count * 2) { // Take more to filter
        if let Some(link_elem) = h3.select(&a_selector).next() {
            let title = link_elem.text().collect::<String>().trim().to_string();
            
            // Skip empty or irrelevant titles
            if title.is_empty() || title.len() < 10 {
                continue;
            }
            
            // Get the bill detail page URL
            if let Some(href) = link_elem.value().attr("href") {
                let bill_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://prsindia.org{}", href)
                };
                
                tracing::debug!("Found bill: {} at {}", title, bill_url);
                
                // Try to find PDF link from the bill detail page
                let pdf_url = fetch_pdf_url_from_bill_page(&bill_url, client).await
                    .unwrap_or_else(|_| generate_placeholder_pdf_url(&title));
                
                // Extract year from title
                let year = extract_year_from_title(&title);
                
                // Extract bill number
                let bill_number = extract_bill_number(&title);
                
                bills.push(Bill::new(
                    title,
                    bill_number,
                    year,
                    pdf_url,
                ));
                
                if bills.len() >= count {
                    break;
                }
            }
        }
    }
    
    Ok(bills)
}

async fn fetch_pdf_url_from_bill_page(bill_url: &str, client: &reqwest::Client) -> Result<String> {
    tracing::debug!("Fetching PDF link from bill page: {}", bill_url);
    
    let response = client.get(bill_url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    // Look for PDF links
    let link_selector = Selector::parse("a[href*='.pdf'], a[href*='files'], a[href*='download']").unwrap();
    
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            // Prioritize actual PDF links
            if href.ends_with(".pdf") || href.contains(".pdf") {
                let pdf_url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("/") {
                    format!("https://prsindia.org{}", href)
                } else {
                    format!("https://prsindia.org/{}", href)
                };
                
                tracing::debug!("Found PDF URL: {}", pdf_url);
                return Ok(pdf_url);
            }
        }
    }
    
    anyhow::bail!("No PDF link found on bill page")
}

fn generate_placeholder_pdf_url(title: &str) -> String {
    // Generate a searchable URL - this will fail gracefully and use demo content
    let sanitized = title.replace(" ", "%20");
    format!("https://prsindia.org/files/bills_acts/bills_parliament/{}", sanitized)
}

fn extract_year_from_title(title: &str) -> i32 {
    // Extract year from title (e.g., "The XYZ Bill, 2024")
    let re = regex::Regex::new(r"(\d{4})").unwrap();
    if let Some(caps) = re.captures(title) {
        if let Ok(year) = caps[1].parse::<i32>() {
            if year >= 1990 && year <= 2030 {
                return year;
            }
        }
    }
    2024 // Default to current year
}

fn extract_bill_number(title: &str) -> String {
    // Try to extract bill number from title
    // Common patterns: "Bill No. 123 of 2024", "The XYZ Bill, 2024"
    
    let re = regex::Regex::new(r"(?i)bill\s*(?:no\.?)?\s*(\d+)\s*of\s*(\d{4})").unwrap();
    if let Some(caps) = re.captures(title) {
        return format!("{}/{}", &caps[1], &caps[2]);
    }
    
    // Extract amendment number if present
    let re_amendment = regex::Regex::new(r"(?i)\(.*?(\d+)(?:st|nd|rd|th)\s+Amendment\)").unwrap();
    if let Some(caps) = re_amendment.captures(title) {
        let year = extract_year_from_title(title);
        return format!("AMEND-{}/{}", &caps[1], year);
    }
    
    // Generate from title and year
    let year = extract_year_from_title(title);
    let hash_bytes = simple_hash(title.as_bytes());
    let hash_str = format!("{:x}", hash_bytes);
    format!("{}/{}", &hash_str[..6].to_uppercase(), year)
}

// Simple hash function for bill number generation
fn simple_hash(input: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

