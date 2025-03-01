use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use guten_rs::config;
use guten_rs::parser::{filter_href, ExtractedLinks};

fn extract_links(current_link: &str, html: &str) -> ExtractedLinks {
    let document = Document::from(html);
    let mut href_vec: Vec<&str> = Vec::new();
    for link in document.find(Name("a")) {
        if let Some(href) = link.attr("href") {
            if href != "../" {
                // Skip parent directory links
                href_vec.push(href);
            }
        }
    }
    filter_href(current_link, href_vec)
}

fn handle_slashes(base_url: &str, url: &str) -> String {
    if url.starts_with('/') && base_url.ends_with('/') {
        let url = url.strip_prefix('/').unwrap();
        format!("{}{}", base_url, url)
    } else if url.starts_with('/') ^ base_url.ends_with('/') {
        format!("{}{}", base_url, url)
    } else {
        format!("{}/{}", base_url, url)
    }
}

async fn crawl(
    client: Arc<Client>,
    url: String,
    visited_links: Arc<Mutex<HashSet<String>>>,
    all_files: Arc<Mutex<Vec<String>>>,
    pb: ProgressBar,
) -> Result<Vec<String>, reqwest::Error> {
    // Check if we've already visited this URL
    {
        let mut visited = visited_links.lock().unwrap();
        if visited.contains(&url) {
            return Ok(Vec::new());
        }
        visited.insert(url.clone());
    }
    let res = fetch_with_retry(client, &url, 3).await?;
    if !res.status().is_success() {
        return Ok(Vec::new());
    }

    let body = res.text().await?;

    // Extract both directories and files
    let extracted = extract_links(&url, &body);
    pb.inc(extracted.files.len() as u64);
    // Store the files in our global collection
    {
        let mut files = all_files.lock().unwrap();
        files.extend(extracted.files);
    }

    // Return the directories for further crawling
    Ok(extracted.directories)
}

async fn fetch_with_retry(
    client: Arc<Client>,
    url: &str,
    max_retries: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut attempts = 0;
    loop {
        let response = client.get(url).send().await?;
        if response.status().is_success() {
            return Ok(response);
        }
        attempts += 1;
        if attempts >= max_retries {
            return Ok(response);
        }
        std::thread::sleep(std::time::Duration::from_secs(4 << attempts));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Client::new());
    let visited_links = Arc::new(Mutex::new(HashSet::new()));
    let all_files = Arc::new(Mutex::new(Vec::new()));
    let pb = ProgressBar::no_length();

    let config = config::get_config();

    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec:>5} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    // Create a queue of URLs to process
    let mut queue: Vec<String> = config
        .whitelist
        .iter()
        .map(|url| handle_slashes(&config.url, url))
        .collect();

    while let Some(url) = queue.pop() {
        let client_clone = Arc::clone(&client);
        let visited_clone = Arc::clone(&visited_links);
        let files_clone = Arc::clone(&all_files);

        // Only process if we haven't visited this URL
        {
            let visited = visited_links.lock().unwrap();
            if visited.contains(&url) {
                continue;
            }
        }

        // Process the URL and get new links
        match crawl(client_clone, url, visited_clone, files_clone, pb.clone()).await {
            Ok(new_links) => {
                // Add new links to the queue
                queue.extend(new_links);
            }
            Err(e) => eprintln!("Error crawling: {}", e),
        }
    }

    // Save the files to a file
    let files = all_files.lock().unwrap();
    println!("Found {} files:", files.len());
    // Create a cache folder
    if !Path::new(".cache").exists() {
        fs::create_dir(".cache")?;
    }
    let out_file = std::fs::File::create(".cache/files.txt")?;
    let mut out_buf = std::io::BufWriter::new(out_file);
    for file in files.iter() {
        writeln!(out_buf, "{}", file)?;
    }
    out_buf.flush()?;
    pb.finish_with_message("Done!");

    Ok(())
}
