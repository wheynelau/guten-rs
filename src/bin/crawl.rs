use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Maximum number of concurrent tasks
const MAX_CONCURRENT_TASKS: usize = 10;
// Optional rate limiting
const RATE_LIMIT_MS: u64 = 100;

async fn crawl_with_workers(start_url: &str) -> Result<(), reqwest::Error> {
    let client = Client::builder()
        .user_agent("My Web Crawler")
        .build()?;
    
    // Use Arc<Mutex<>> to share state between tasks
    let visited_links = Arc::new(Mutex::new(HashSet::new()));
    let pending_links = Arc::new(Mutex::new(vec![start_url.to_string()]));
    
    // Main crawling loop
    loop {
        let mut tasks = FuturesUnordered::new();
        
        // Get batch of URLs to process
        let urls_to_process = {
            let mut pending = pending_links.lock().await;
            let count = std::cmp::min(pending.len(), MAX_CONCURRENT_TASKS);
            pending.drain(0..count).collect::<Vec<_>>()
        };
        
        if urls_to_process.is_empty() {
            break; // No more URLs to process
        }
        
        // Create tasks for each URL
        for url in urls_to_process {
            let client = client.clone();
            let visited_links = Arc::clone(&visited_links);
            let pending_links = Arc::clone(&pending_links);
            
            tasks.push(tokio::spawn(async move {
                // Check if already visited
                {
                    let visited = visited_links.lock().await;
                    if visited.contains(&url) {
                        return Ok(());
                    }
                }
                
                // Mark as visited
                {
                    let mut visited = visited_links.lock().await;
                    visited.insert(url.clone());
                }
                
                // Optional rate limiting
                sleep(Duration::from_millis(RATE_LIMIT_MS)).await;
                
                // Fetch the page
                let res = client.get(&url).send().await?;
                if !res.status().is_success() {
                    return Ok(());
                }
                
                let body = res.text().await?;
                let document = Document::from(body.as_str());
                
                // Extract links
                let mut new_links = Vec::new();
                for link in document.find(Name("a")) {
                    if let Some(href) = link.attr("href") {
                        if href.starts_with("http") {
                            new_links.push(href.to_string());
                        }
                    }
                }
                
                // Add new links to pending queue
                {
                    let mut visited = visited_links.lock().await;
                    let mut pending = pending_links.lock().await;
                    
                    for link in new_links {
                        if !visited.contains(&link) {
                            pending.push(link);
                        }
                    }
                }
                
                // Process the page content here
                println!("Processed: {}", url);
                
                Ok::<(), reqwest::Error>(())
            }));
        }
        
        // Wait for all tasks in this batch to complete
        while let Some(result) = tasks.next().await {
            if let Err(e) = result {
                eprintln!("Task panicked: {}", e);
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let start_url = "https://example.com";
    crawl_with_workers(start_url).await
}
