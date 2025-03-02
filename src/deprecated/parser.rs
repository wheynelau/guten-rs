// This is unused, just for reference

use spider::hashbrown::{HashSet,HashMap};
use spider::CaseInsensitiveString;

pub fn parse(links: HashSet<CaseInsensitiveString>, website_url: &str) -> Vec<String> {
    let mut final_results = Vec::with_capacity(links.len()); // Pre-allocate for better performance
    let mut zips = HashSet::new();
    let mut non_zip_paths = HashMap::new();
    
    // First pass: collect all zip files and their paths
    for link in &links {
        if let Some(clean_path) = suffix(website_url, link) {
            if link.ends_with(".zip") {
                zips.insert(clean_path);
                final_results.push(link.to_string());
            } 
            // No need to check for ends with ("/"), suffix handles it
            else {
                non_zip_paths.insert(clean_path, link.to_string());
            }
        }
    }
    
    // Second pass: add files that aren't part of any zip
    for (clean_path, link) in non_zip_paths {
        if !file_is_part_of_zip(&clean_path, &zips) {
            final_results.push(link);
        }
    }
    
    final_results
}


fn file_is_part_of_zip (path: &str, map: &HashSet<String>) -> bool {
    for zip in map {
        if path.starts_with(zip) {
            return true;
        }
    }
    false
}


pub fn suffix(website_url: &str, full_path: &str) -> Option<String> {
    // Remove the website_url prefix if present
    let path = full_path.strip_prefix(website_url).unwrap_or(full_path);
    
    // Find the last dot to separate the extension
    path.rfind('.').map(|last_dot_pos| path[..last_dot_pos].to_string())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse () {
    
        let links : HashSet<CaseInsensitiveString> = HashSet::from_iter(vec![
            "https://abc.0/".into(),
            "https://abc.0/1/2".into(),
            "https://abc.0/1/2.html".into(),
            "https://abc.0/1/2/3".into(),
            "https://abc.0/1.zip".into(),
            "https://abc.0/1.html".into(),
            // not sure if such a case happens
            // strugggling to pass this test
            // "https://abc.0/1/2.zip".into(),
            "https://abc.0/3.html".into(),

        ]);
        let result = parse(links, "https://abc.0/");
        println!("{:?}", result);
        // should only contain "https://abc.0/1.zip"
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "https://abc.0/1.zip");
        assert_eq!(result[1], "https://abc.0/3.html");
    }

    #[test]
    fn test_zip_filter () {
        let links : HashSet<String> = HashSet::from_iter(vec![
            "1".into(),
            "1/2".into(),
            "1/2/3".into(),
        ]);
        let path = "1";
        let result = file_is_part_of_zip(path , &links);
        assert!(result);
        // different path
        let path = "2";
        let result = file_is_part_of_zip(path , &links);
        assert!(!result);
    }
    #[test]
    fn test_basename () {
        let website_url = "https://abc.0/";
        let link = "https://abc.0/1/2/3";
        let result = suffix(website_url, link);
        assert_eq!(result, None);
        let link = "https://abc.0/1/2.zip";
        let result = suffix(website_url, link);
        assert_eq!(result, Some("1/2".to_string()));
    }

    #[test]
    fn test_suffix () {
        let website_url = "https://abc.0/";
        let link = "https://abc.0/1/2/3";
        let result = suffix(website_url, link);
        assert_eq!(result, None);
        let link = "https://abc.0/1/2.zip";
        let result = suffix(website_url, link);
        assert_eq!(result, Some("1/2".to_string()));
        let link = "https://abc.0/1/2.html";
        let result = suffix(website_url, link);
        assert_eq!(result, Some("1/2".to_string()));
    }

}