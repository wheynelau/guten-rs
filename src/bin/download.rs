use anyhow::Error;

use std::fs;
use std::io::BufRead;
use std::io::BufReader;

use guten_rs::config;
use guten_rs::downloader;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::get_config();

    // Load from cache
    let links_path = BufReader::new(fs::File::open(".cache/files.txt")?);
    let download_path = match &config.download_path {
        Some(path) => path,
        None => "./download",
    };
    let parsed_links: Vec<String> = links_path.lines().map_while(Result::ok).collect();

    downloader::download(parsed_links, download_path, &config).await?;
    Ok(())
}
