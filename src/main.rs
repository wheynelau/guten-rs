use spider::tokio;
use spider::website::Website;
use std::time::Instant;
use env_logger::Env;

pub mod parser;

#[tokio::main]
async fn main() {
    let website_url = "https://mirrors.xmission.com/gutenberg/";
    let mut website: Website = Website::new(website_url);

    let urls = vec![
        "/0",
    ];

    let env = Env::default()
    .filter_or("RUST_LOG", "info")
    .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    website.with_whitelist_url(
        Some(
            urls.into_iter().map(|url| 
            url.into()).collect()
        )
    );
    website.with_full_resources(true);

    let start = Instant::now();

    website.crawl().await;

    let duration = start.elapsed();
    let links = website.get_links();

    for link in links.iter() {
        println!("- {:?}", link.as_ref());
    }
    println!(
        "Time elapsed in website.crawl() is: {:?} for total pages: {:?}",
        duration,
        links.len()
    );
    let parsed_links = parser::parse(links, website_url);
    println!("Parsed links: {:?}", parsed_links);
    println!("Parsed links: {:?}", parsed_links.len());
}