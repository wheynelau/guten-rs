use std::collections::{HashMap, HashSet};

// Define a struct to hold both types of links
#[derive(Debug)]
pub struct ExtractedLinks {
    pub directories: Vec<String>, // Links ending with "/" for crawling
    pub files: Vec<String>,       // Links not ending with "/" (objects)
}

impl ExtractedLinks {
    pub fn len(&self) -> usize {
        self.directories.len() + self.files.len()
    }
    pub fn is_empty(&self) -> bool {
        self.directories.is_empty() && self.files.is_empty()
    }
}

// this filters the zip and the source files
pub fn filter_href(current_link: &str, href: Vec<&str>) -> ExtractedLinks {
    // if .zip and .txt exist, take .zip
    // if the zip is for the folder, remove the folder
    let mut base_to_path: HashMap<&str, &str> = HashMap::new();

    // First pass: collect all paths by their base name (without extension)
    for &path in &href {
        let base_name = if let Some(stripped) = path.strip_suffix('/') {
            // For directories, remove trailing slash
            stripped
        } else if let Some(last_dot_pos) = path.rfind('.') {
            // For files, remove extension
            &path[..last_dot_pos]
        } else {
            // No extension or trailing slash
            path
        };

        // Check if we already have an entry for this base name
        if let Some(existing_path) = base_to_path.get(base_name) {
            // If the existing path is not a zip and the current one is, replace it
            if !existing_path.ends_with(".zip") && path.ends_with(".zip") {
                base_to_path.insert(base_name, path);
            }
        } else {
            // No existing entry, add this one
            base_to_path.insert(base_name, path);
        }
    }

    // Second pass: handle the case where we have both a folder and a zip file for it
    let mut result = ExtractedLinks {
        directories: Vec::new(),
        files: Vec::new(),
    };
    let mut folder_bases = HashSet::new();

    // Collect all folder base names
    for &path in &href {
        if let Some(stripped) = path.strip_suffix('/') {
            folder_bases.insert(stripped);
        }
    }

    // Build the result, prioritizing zips over folders with the same name
    for &path in &href {
        let base = if let Some(stripped) = path.strip_suffix('/') {
            // For directories, remove trailing slash
            stripped
        } else if let Some(last_dot_pos) = path.rfind('.') {
            // For files, remove extension
            &path[..last_dot_pos]
        } else {
            // No extension or trailing slash
            path
        };

        // If this is a folder and we have a zip for it, skip the folder
        if path.ends_with('/') && href.iter().any(|&p| p == format!("{}.zip", base)) {
            continue;
        }

        // If this is a non-zip file and we have a zip for it, skip this file
        if !path.ends_with('/')
            && !path.ends_with(".zip")
            && href.iter().any(|&p| p == format!("{}.zip", base))
        {
            continue;
        }

        // Add to result if it's the preferred version for this base name
        if base_to_path.get(base) == Some(&path) {
            // if is a folder, add to directories
            let path = format! {"{}{}", current_link, path};
            if path.ends_with('/') {
                result.directories.push(path.to_string());
            } else {
                result.files.push(path.to_string());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_filter_href() {
        let hrefs = vec!["1.zip", "1.txt", "2.zip", "2/", "3/"];
        let filtered = filter_href("", hrefs);

        println!("{:?}", filtered);
        // contain 3/ only
        assert_eq!(filtered.directories.len(), 1);
        // contain 1.zip, 2.zip
        assert_eq!(filtered.files.len(), 2);
    }
}
