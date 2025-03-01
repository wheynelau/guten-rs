use spider::tokio;
use spider::website::Website;
use anyhow::Error;

use std::time::Instant;
use env_logger::Env;

use guten_rs::config;
use guten_rs::parser;
use guten_rs::downloader;

#[tokio::main]
async fn main() -> Result<(), Error> {

    let config = config::get_config();
    let website_url = &config.url;
    let mut website: Website = Website::new(website_url);

    let download_path = match &config.download_path {
        Some(path) => path,
        None => "./download"
    };
    let _env = Env::default()
    .filter_or("RUST_LOG", "info")
    .write_style_or("RUST_LOG_STYLE", "always");

    // env_logger::init_from_env(_env);

    website.with_whitelist_url(
        Some(
            config.whitelist.iter().map(|url| 
            url.into()).collect()
        )
    );
    website.with_full_resources(true);

    let start = Instant::now();

    website.crawl().await;

    let duration = start.elapsed();
    let links = website.get_links();

    println!(
        "Time elapsed in website.crawl() is: {:?} for total pages: {:?}",
        duration,
        links.len()
    );
    let parsed_links = parser::parse(links, website_url);
    
    downloader::download(parsed_links, website_url, download_path, &config).await?;
    Ok(())
}