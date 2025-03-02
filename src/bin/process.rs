use std::path::Path;
use std::path::PathBuf;

// There is not much IO, so rayon can be utilized here
use guten_rs::config;
use guten_rs::postprocess;
use rayon::prelude::*;

use glob::glob;
fn main() -> Result<(), anyhow::Error> {
    let config = config::get_config();
    let download_dir = match config.download_path {
        Some(ref path) => path,
        None => "./download",
    };
    // Stage 1: Unzip all the files into their own folders
    let output_path = Path::new("./cleaned");

    let pattern = format!("{}/**/*.zip", download_dir);
    let zip_files: Vec<_> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .collect();

    zip_files.par_iter().for_each(|entry| match entry {
        Ok(path) => {
            let path_str = path.display().to_string();
            let _ = postprocess::unarchive::unzip(&path_str, false);
        }
        Err(e) => println!("{:?}", e),
    });
    // Now match all
    let pattern = format!("{}/**/*.txt", download_dir);
    // Only txt supported for now
    let txt_files: Vec<PathBuf> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    println!("Found {} files to process", txt_files.len());
    // Stage 2: Start the data processing
    txt_files.par_iter().for_each(|source_file| {
        postprocess::clean::clean_file(source_file, download_dir, output_path).unwrap();
    });
    Ok(())
}
