// For reference, custom downloading

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