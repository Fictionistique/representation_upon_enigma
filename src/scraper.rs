use anyhow::{Context, Result};
use scraper::{Html, Selector};
use crate::models::Bill;

/// Fetches recent bills from PRS India website
pub async fn fetch_recent_bills(count: usize) -> Result<Vec<Bill>> {
    tracing::info!("Fetching bills from PRS Legislative Research...");
    
    // PRS India's bill tracking page
    let url = "https://prsindia.org/billtrack/recent-bills";
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch PRS bills page")?;
    
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    
    // Parse the HTML to extract bill information
    // Note: This is a simplified implementation. The actual PRS website structure may vary.
    let bills = parse_bills_from_html(&document, count)?;
    
    // If we can't scrape from PRS (due to structure changes), return mock data for demo
    if bills.is_empty() {
        tracing::warn!("Could not scrape bills from PRS. Using demo data instead.");
        return Ok(create_demo_bills(count));
    }
    
    Ok(bills)
}

fn parse_bills_from_html(document: &Html, count: usize) -> Result<Vec<Bill>> {
    let mut bills = Vec::new();
    
    // Selector for bill rows (this is approximate - actual structure may differ)
    let row_selector = Selector::parse("tr.bill-row, .bill-item, article.bill").unwrap();
    let title_selector = Selector::parse(".bill-title, h3, .title").unwrap();
    let link_selector = Selector::parse("a[href*='pdf'], a[href*='bill']").unwrap();
    
    for element in document.select(&row_selector).take(count) {
        if let Some(title_elem) = element.select(&title_selector).next() {
            let title = title_elem.text().collect::<String>().trim().to_string();
            
            if let Some(link_elem) = element.select(&link_selector).next() {
                if let Some(href) = link_elem.value().attr("href") {
                    let pdf_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://prsindia.org{}", href)
                    };
                    
                    // Extract bill number from title or use a generated one
                    let bill_number = extract_bill_number(&title);
                    
                    bills.push(Bill::new(
                        title,
                        bill_number,
                        2024, // Current year
                        pdf_url,
                    ));
                }
            }
        }
    }
    
    Ok(bills)
}

fn extract_bill_number(title: &str) -> String {
    // Try to extract bill number from title
    // Common patterns: "Bill No. 123 of 2024", "The XYZ Bill, 2024"
    
    let re = regex::Regex::new(r"(?i)bill\s*(?:no\.?)?\s*(\d+)\s*of\s*(\d{4})").unwrap();
    if let Some(caps) = re.captures(title) {
        return format!("{}/{}", &caps[1], &caps[2]);
    }
    
    let re2 = regex::Regex::new(r"(\d{4})").unwrap();
    if let Some(caps) = re2.captures(title) {
        let year = &caps[1];
        let hash_bytes = md5::compute(title.as_bytes());
        let hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        return format!("{}/{}", &hash[..6], year);
    }
    
    // Fallback: generate from title hash
    let hash_bytes = md5::compute(title.as_bytes());
    let hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    format!("{}/2024", &hash[..8])
}

/// Creates demo bills for testing when scraping isn't available
fn create_demo_bills(count: usize) -> Vec<Bill> {
    let demo_data = vec![
        (
            "The Digital Personal Data Protection Bill, 2023",
            "DPDP/2023",
            2023,
            "https://www.meity.gov.in/writereaddata/files/Digital%20Personal%20Data%20Protection%20Bill%2C%202023.pdf"
        ),
        (
            "The Criminal Procedure (Identification) Bill, 2022",
            "CPI/2022",
            2022,
            "https://prsindia.org/files/bills_acts/bills_parliament/2022/Criminal%20Procedure%20(Identification)%20Bill,%202022.pdf"
        ),
        (
            "The Telecommunications Bill, 2023",
            "TELE/2023",
            2023,
            "https://dot.gov.in/sites/default/files/2023_09_20%20Telecom%20Bill%202023%20AS%20INTRODUCED_0.pdf"
        ),
        (
            "The Constitution (One Hundred and Twenty-Eighth Amendment) Bill, 2023",
            "CONST128/2023",
            2023,
            "https://sansad.in/getFile/loksabhaquestions/annex/1711/AS20.pdf"
        ),
        (
            "The Multi-State Co-operative Societies (Amendment) Bill, 2022",
            "MSCS/2022",
            2022,
            "https://prsindia.org/files/bills_acts/bills_parliament/2022/Multi-State%20Cooperative%20Societies%20(Amendment)%20Bill,%202022.pdf"
        ),
    ];
    
    demo_data
        .into_iter()
        .take(count)
        .map(|(title, number, year, url)| {
            Bill::new(title.to_string(), number.to_string(), year, url.to_string())
        })
        .collect()
}

// Mock MD5 implementation for bill number generation
mod md5 {
    pub fn compute(input: &[u8]) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        hash.to_le_bytes().to_vec()
    }
}

