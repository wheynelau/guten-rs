// There is not much IO, so rayon can be utilized here
use guten_rs::postprocess;
use guten_rs::cleaner;
use guten_rs::config;
use rayon::prelude::*;

use glob::glob;
fn main () -> Result<(), anyhow::Error> {

    let config = config::get_config();
    let download_dir = config.download_path;
    // Stage 1: Unzip all the files into their own folders

    let pattern = match download_dir {
        Some(ref path) => format!("{}/**/*.zip", path),
        None => String::from("./download/**/*.zip")
    };
    let zip_files: Vec<_> = glob(&pattern).expect("Failed to read glob pattern").collect();

    zip_files.par_iter().for_each(|entry| {
        match entry {
            Ok(path) => {
                let path_str = path.display().to_string();
                let _ = postprocess::unarchive::unzip(&path_str, false);
            },
           Err(e) => println!("{:?}", e), 
        }
    });
    // Now match all non zips
    let pattern = match download_dir {
        Some(ref path) => format!("{}/**/*.txt", path),
        None => String::from("./download/**/*.txt")
    };

    let txt_files: Vec<_> = glob(&pattern).expect("Failed to read glob pattern").collect();
    // Stage 2: Start the data processing
    txt_files.par_iter().for_each(|entry| {
        match entry {
            Ok(path) => {
                let path_str = path.display().to_string();
                let _ = postprocess::unarchive::unzip(&path_str, false);
            },
           Err(e) => println!("{:?}", e), 
        }
    })

    Ok(())
}