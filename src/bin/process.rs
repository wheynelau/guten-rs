// There is not much IO, so rayon can be utilized here
use guten_rs::postprocess::processor::unzip;

fn main () -> Result<(), anyhow::Error> {

    let file = "/shared/aisingapore/users/wayne/source_files/guten-rs/download/0/1/1-h.zip";
    unzip(file, false);
    Ok(())
}