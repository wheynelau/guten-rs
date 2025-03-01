// Reference here: https://github.com/c-w/gutenberg/blob/master/gutenberg/cleanup/strip_headers.py

use super::constants;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::path::Path;
use std::fs;
use std::io::Write;

pub fn clean_file (path:&str, download_path:&str, output_path: &Path) -> Result<(), anyhow::Error> {
    
    let fname = Path::new(path);
    let root_folder = fname.parent().unwrap();
    let root_folder_str = root_folder.to_str().unwrap(); 
    let folder_to_make_str = root_folder_str.replace(download_path, output_path.to_str().unwrap()); // replacement
    let folder_to_make = Path::new(&folder_to_make_str);

    fs::create_dir_all(folder_to_make)?; // Corrected method name
    let file_bytes = fs::read(fname)?;
    let string:String = String::from_utf8_lossy(&file_bytes).into();

    let cleaned = strip_headers(string);
        {
            let mut file = fs::File::create("clean.txt").unwrap();
            write!(file, "{}", cleaned);
        }
    
}

fn strip_headers (text:String) -> String {

    // hardcode?
    let sep = "\n";
    let mut out: Vec<&str> = Vec::new();
    let mut footer_found: bool = false;
    let mut ignore_section: bool = false;
    let mut reset = false;
    let mut i:u16 = 0;

    for line in text.lines() {
        // println!("{:?}", &line);
        reset = false;

        if i <= 600 {
            // check if header ends here
            if lines_starts_with(line, &constants::TEXT_START_MARKERS) {
                reset = true
            }
            if reset {
                out.clear();
                continue
            }
            // If it's the end of the header, delete the output produced so far.
            // May be done several times, if multiple lines occur indicating the
            // end of the header
        }
        if i>=100 {
            // Check if the footer begins here
            if lines_starts_with(line, &constants::TEXT_END_MARKERS) {                
                footer_found = true
            }
            if footer_found {
                break
            }
        }
        
        if lines_starts_with(line,&constants::LEGALESE_START_MARKERS) {
            ignore_section = true;
            continue
        }
        else if lines_starts_with(line, &constants::LEGALESE_END_MARKERS) {
            ignore_section = false;
            continue
        }
        if ! ignore_section {
            out.push(line.trim_end_matches(sep));
            i += 1
        }
    };
    out.join(sep)
}

fn lines_starts_with(line: &str, mapping: &Lazy<HashSet<&str>>) -> bool {
    mapping.iter().any(|&prefix| line.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};
    use super::*;
    #[test]
    fn test_clean() {
        let file_bytes = fs::read("22222-8.txt").unwrap();
        let string:String = String::from_utf8_lossy(&file_bytes).into();
        let cleaned = strip_headers(string);
        {
            let mut file = fs::File::create("clean.txt").unwrap();
            write!(file, "{}", cleaned);
        }
    }
    #[test]
    fn test_lines_start () {
        let string = "<<THIS ELECTRONIC VERSION OF";
        let result = lines_starts_with(string, &constants::LEGALESE_END_MARKERS);
        assert!(result);
    }
}
