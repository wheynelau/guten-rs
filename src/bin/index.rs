//This helper indexes the GUTENBURG INDEX into structured file

use clap::Parser;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use guten_rs::index::index;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "INDEX",
    value_hint=clap::ValueHint::DirPath)]
    input_file: PathBuf,

    #[arg(short, long, value_name = "OUTPUT",
    value_hint=clap::ValueHint::DirPath)]
    output_file: Option<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let input_path = &args.input_file;
    let output_file = match args.output_file {
        Some(file) => file,
        None => PathBuf::from(".cache/index.json"),
    };
    let output_file = File::create(output_file)?;
    let string = fs::read_to_string(input_path).expect("Unable to find input file");
    let index: HashMap<u32, String> = index(&string);
    println!("{:?}", index.len());
    let toml_string = serde_json::to_string(&index)?;
    let mut writer = std::io::BufWriter::new(output_file);
    writer.write_all(toml_string.as_bytes())?;

    Ok(())
}
