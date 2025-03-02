// Reference here: https://github.com/c-w/gutenberg/blob/master/gutenberg/cleanup/strip_headers.py

use super::constants;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn clean_file(
    source_file: &PathBuf,
    download_path: &str,
    output_path: &Path,
) -> Result<(), anyhow::Error> {
    // Get the source file path

    // Create the destination path by replacing download_path with output_path
    let rel_path = source_file
        .strip_prefix(download_path)
        .map_err(|_| anyhow::anyhow!("Source file is not within download path"))?;
    let dest_file = output_path.join(rel_path);

    // Create parent directories
    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read, clean, and write the file
    let file_bytes = fs::read(source_file)?;
    let content = String::from_utf8_lossy(&file_bytes).into();
    let cleaned = strip_headers(content);

    fs::write(dest_file, cleaned)?;

    Ok(())
}

fn strip_headers(text: String) -> String {
    // hardcode?
    let sep = "\n";
    let mut out: Vec<&str> = Vec::new();
    let mut ignore_section: bool = false;
    let mut i: u16 = 0;

    for line in text.lines() {
        if i <= 600 {
            // check if header ends here
            if lines_starts_with(line, &constants::TEXT_START_MARKERS) {
                out.clear();
                continue;
            }
            // If it's the end of the header, delete the output produced so far.
            // May be done several times, if multiple lines occur indicating the
            // end of the header
        }
        if i >= 100 {
            // Check if the footer begins here
            if lines_starts_with(line, &constants::TEXT_END_MARKERS) {
                break;
            }
        }

        if lines_starts_with(line, &constants::LEGALESE_START_MARKERS) {
            ignore_section = true;
            continue;
        } else if lines_starts_with(line, &constants::LEGALESE_END_MARKERS) {
            ignore_section = false;
            continue;
        }
        if !ignore_section {
            out.push(line.trim_end_matches(sep));
            i += 1
        }
    }
    out.join(sep)
}

fn lines_starts_with(line: &str, mapping: &Lazy<HashSet<&str>>) -> bool {
    mapping.iter().any(|&prefix| line.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};
    #[test]
    fn test_clean() {
        let file_bytes = fs::read("22222-8.txt").unwrap();
        let string: String = String::from_utf8_lossy(&file_bytes).into();
        let cleaned = strip_headers(string);
        {
            let mut file = fs::File::create("clean.txt").unwrap();
            write!(file, "{}", cleaned).unwrap();
        }
    }
    #[test]
    fn test_lines_start() {
        let string = "<<THIS ELECTRONIC VERSION OF";
        let result = lines_starts_with(string, &constants::LEGALESE_END_MARKERS);
        assert!(result);
    }
}
