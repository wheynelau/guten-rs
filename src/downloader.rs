use std::path::PathBuf;
use anyhow::{anyhow,Error};
use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use spider::{tokio::{self, io::AsyncWriteExt}, url::Url};
use rand::prelude::*;
use trauma::{download::Download, downloader::DownloaderBuilder};


use crate::config::Config;

pub async fn _download(full_url: Vec<String>, 
    website_url : &str, 
    download_path: &str,
    config: &Config) -> Result<(), Error> {

    let _mirrors = &config.mirrors;
    let concurrency = config.download_settings.concurrency;
    let client = reqwest::Client::new();

    let pb = ProgressBar::new(full_url.len() as u64);
    pb.tick();
    let downloads = stream::iter(
        full_url.into_iter().map(|url| {
            let pb = pb.clone();
            let client = client.clone();
            async move {
                let result = download_one_file(
                    client,
                    &url, 
                    &website_url, 
                    download_path).await;
                pb.inc(1);
                result
            }
        })
    )
    .buffer_unordered(concurrency)
    .collect::<Vec<Result<(), Error>>>()
    .await;

    // Process results
    let errors: Vec<_> = downloads.into_iter()
        .filter_map(|r| r.err())
        .collect();

    // print all errors
    for error in errors.iter() {
        println!("Error: {}", error);
    }
    
    if errors.is_empty() {
        pb.finish_with_message("All downloads completed successfully");
        Ok(())
    } else {
        let message = format!("Completed with {} errors", errors.len());
        pb.finish_with_message(message);
        // You might want to return the first error or a custom error with all failures
        Err(anyhow!("Failed to download {} files", errors.len()))
    }
}

#[allow(dead_code)]
fn generate_mirrored_url(url :&str, website_url : &str, mirrors: &Vec<String>) -> (String, String) {

    // do a random on the mirrors
    let mut rng = rand::rng();
    let random_mirror = mirrors.choose(&mut rng).unwrap();
    let replace = url.replace(website_url, random_mirror);
    (replace, random_mirror.to_string(), )
}

async fn fetch_with_retry(client: &reqwest::Client, url: &str, max_retries: usize) -> Result<reqwest::Response, Error> {
    let mut attempts = 0;
    
    loop {
        let response = client.get(url).send().await?;
        
        if response.status().is_success() {
            return Ok(response);
        }
        
        attempts += 1;
        if attempts >= max_retries {
            return Err(anyhow!("Failed to download after {} attempts: HTTP status {}", 
                              attempts, response.status()));
        }
        
        // Exponential backoff with jitter
        let backoff_duration = tokio::time::Duration::from_millis(
            (1 << attempts) * 100 + rand::random::<u64>() % 100
        );
        
        println!("Request failed with status {}, retrying in {:?}...", 
                response.status(), backoff_duration);
        
        tokio::time::sleep(backoff_duration).await;
    }
}


async fn download_one_file(
    client: reqwest::Client,
    url: &str, 
    website_url: &str, 
    download_folder: &str) -> Result<(), Error> {
    // Build the download path
    let path = build_download_path(url, website_url, download_folder)?;

    let parent = path
    .parent()
    .ok_or_else(|| anyhow!("No parent directory"))?;
    
    // Create the directory if it doesn't exist
    tokio::fs::create_dir_all(parent).await?;
    
    // Construct the full URL
    // let full_url = if url.starts_with("http") {
    //     url.to_string()
    // } else {
    //     format!("{}{}", website_url, url)
    // };
    
    // Download the file
    let max_retries = 32;
    let response = fetch_with_retry(&client, url, max_retries).await?;
    
    // Ensure the request was successful
    if !response.status().is_success() {
        return Err(anyhow!("Failed to download: HTTP status {}", response.status()));
    }
    
    // Create the file
    let mut file = tokio::fs::File::create(&path).await?;
    
    // Stream the response body and write it to the file chunk by chunk
    // Download the entire response body into memory
    let bytes = response.bytes().await?;

    // Write the entire content to the file at once
    file.write_all(&bytes).await?;

    // Ensure all data is written
    file.flush().await?;
    
    Ok(())
}


pub async fn download(full_url: Vec<String>, 
        website_url : &str, 
        download_path: &str,
        _config: &Config) -> Result<(), Error> {
    let downloads: Vec<Download> = full_url.into_iter()
    .filter_map(|url| {
        // Try to build the download path and handle the Result
        match build_download_path(&url, website_url, download_path) {
            Ok(path) => Some(Download {
                url: match Url::parse(&url) {
                    Ok(parsed_url) => parsed_url,
                    Err(_) => return None, // Filter out invalid URLs
                },
                filename: path.to_string_lossy().into_owned()
            }),
            Err(_) => None // Filter out URLs that fail to build a download path
        }
    })
    .collect();
    let downloader = DownloaderBuilder::new().build();
    downloader.download(&downloads).await;
    Ok(())
}

///
/// Role of this function is to build the folder for the 
/// download, not the file directly
/// 
fn build_download_path(url: &str, 
    website_url : &str,
    download_folder: &str) -> Result<PathBuf, Error> {
    let suffix = match url.strip_prefix(website_url)
    {
        Some(suffix) => suffix,
        None => return Err(anyhow!("The url didn't have the right prefix?"))
    };

    // Create the path
    let mut download_file = PathBuf::from(download_folder);
    download_file.push(suffix);
    
    // Convert back to string
    Ok(download_file)
}

#[cfg(test)]
mod tests {

    use std::{fs, io::Write};

    use super::*;

    #[test]
    fn test_build_download_path () {
        let url = "https://abc.0/1/2/3.zip";
        let website_url = "https://abc.0/";
        let download_folder = "output";

        let clean_name = build_download_path(url, website_url, download_folder).unwrap();
        assert_eq!(clean_name, PathBuf::from("output/1/2/3.zip"));

        let url = "https://abc.0/1.html";
        let download_folder = "download";
        let clean_name = build_download_path(url, website_url, download_folder).unwrap();
        assert_eq!(clean_name, PathBuf::from("download/1.html"));

        // wrong suffix
        let url = "https://abc.1/1.html";
        let download_folder = "download";
        let clean_name = build_download_path(url, website_url, download_folder);
        assert!(clean_name.is_err());

    }
    #[tokio::test]
    async fn test_raw_download() {
        // this is just a functional check on pure reqwest and the url
        let reqwest_rs = "https://gutenberg.pglaf.org/0/4/4.zip";
        // create folder
        {      
            std::fs::create_dir("test_dir").unwrap();
            let mut file = fs::File::create("test_dir/reqwest.zip").expect("Failed to create file");
            let response = reqwest::get(reqwest_rs).await.expect("Failed to get response");
            let bytes = response.bytes().await.expect("Failed to get response bytes");
            
            file.write_all(&bytes).expect("Failed to write to file");
        }
        let file_path = "test_dir/reqwest.zip";
        {
            let file = fs::File::open(file_path);
            assert!(file.is_ok());
        }
        // remove folder
        std::fs::remove_file(file_path).expect("Failed to remove file");
        // wait for awhile
        // remove folder
        std::fs::remove_dir_all("test_dir").expect("Failed to remove directory");
    }


}