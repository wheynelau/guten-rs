use anyhow::{anyhow, Error};
use reqwest::Url;
use std::path::PathBuf;
use std::time::Instant;
use trauma::{download::Download, downloader::DownloaderBuilder};

use crate::config::Config;

pub async fn download(
    full_url: Vec<String>,
    download_path: &str,
    config: &Config,
) -> Result<(), Error> {
    let start = Instant::now();
    let concurrency = config.download_settings.concurrency;
    let downloads: Vec<Download> = full_url
        .into_iter()
        .filter_map(|url| {
            // Try to build the download path and handle the Result
            match build_download_path(&url, &config.url, download_path) {
                Ok(path) => Some(Download {
                    url: match Url::parse(&url) {
                        Ok(parsed_url) => parsed_url,
                        Err(_) => return None, // Filter out invalid URLs
                    },
                    filename: path.to_string_lossy().into_owned(),
                }),
                Err(_) => None, // Filter out URLs that fail to build a download path
            }
        })
        .collect();
    let downloader = DownloaderBuilder::new()
        .concurrent_downloads(concurrency)
        .build();
    downloader.download(&downloads).await;

    println!("Download time: {:?}", start.elapsed());
    Ok(())
}

///
/// Role of this function is to build the folder for the
/// download, not the file directly
///
fn build_download_path(
    url: &str,
    website_url: &str,
    download_folder: &str,
) -> Result<PathBuf, Error> {
    let suffix = match url.strip_prefix(website_url) {
        Some(suffix) => suffix,
        None => return Err(anyhow!("The url didn't have the right prefix?")),
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
    fn test_build_download_path() {
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
            let response = reqwest::get(reqwest_rs)
                .await
                .expect("Failed to get response");
            let bytes = response
                .bytes()
                .await
                .expect("Failed to get response bytes");

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
