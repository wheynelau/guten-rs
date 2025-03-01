use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
   pub url : String,
   pub whitelist: Vec<String>,
   pub download_path: Option<String>,
   pub mirrors: Vec<String>,
   pub download_settings: DownloadSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DownloadSettings {
   pub concurrency : usize,
}

// find a default config file
pub fn get_config() -> Config {
    let config_file = std::fs::read_to_string("config.toml")
    .expect("Failed to read config file");
    let config: Config = toml::from_str(&config_file).expect("Failed to parse config file");
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        let config_str = r#"
url = "https://testurl.com"
whitelist = ["/0"]
mirrors = [
"https://mirror.csclub.uwaterloo.ca/gutenberg/",
"https://aleph.gutenberg.org/"
]

[download_settings]
concurrency = 8
        "#;
        let config:Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.url, "https://testurl.com");
        assert_eq!(config.whitelist.len(), 2);
        println!("config: {:?}", config);
    }
}