use anyhow::Result;
use std::fs::File;
use std::path::PathBuf;

pub fn run(archive_path: &str, output_dir: Option<&str>) -> Result<()> {
    let archive_file = File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);

    let output = output_dir.unwrap_or(".");

    println!("Extracting transport archive to: {}", output);

    archive.unpack(output)?;

    println!("Archive extracted successfully.");

    let gitehr_dir = PathBuf::from(output).join(".gitehr");
    if gitehr_dir.exists() {
        println!("GitEHR repository detected in extracted files.");
    }

    Ok(())
}
