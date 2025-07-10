use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::Path;
use tar::Archive;
use zstd::stream::read::Decoder as ZstdDecoder;
use zstd::stream::write::Encoder as ZstdEncoder;

pub fn extract_archive(file: &str, destination: &str) -> Result<()> {
    let path = Path::new(file);
    let dest_path = Path::new(destination);

    // Ensure the destination directory exists
    fs::create_dir_all(dest_path).context(format!(
        "Failed to create destination directory: {}",
        destination
    ))?;

    // Open the .lpkg file
    let file = File::open(path).context(format!("Failed to open archive file: {}", file))?;

    // Decompress using zstd
    let mut decoder = ZstdDecoder::new(file).context("Failed to create zstd decoder")?;

    // Unpack the tar archive
    let mut archive = Archive::new(&mut decoder);
    archive
        .unpack(dest_path)
        .context("Failed to unpack tar archive")?;

    Ok(())
}

pub fn create_archive(files_dir: &Path, meta_path: &Path, output_file: &str) -> Result<()> {
    let output_path = Path::new(output_file);

    // Create the output file
    let output_file = File::create(output_path)
        .context(format!("Failed to create output file: {}", output_file))?;

    // Create a zstd encoder
    let mut encoder = ZstdEncoder::new(output_file, 3)
        .context("Failed to create zstd encoder")?
        .auto_finish();

    // Create a tar builder
    let mut builder = tar::Builder::new(&mut encoder);

    // Add the contents of the files directory to the tar archive
    // Manually append each file to ensure correct path handling
    use walkdir::WalkDir;
    for entry in WalkDir::new(files_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let rel_path = path.strip_prefix(files_dir).context(format!(
                "Failed to strip prefix from path {}",
                path.display()
            ))?;
            builder.append_path_with_name(path, &format!("files/{}", rel_path.display()))?;
        }
    }

    // Add meta.toml to the root of the archive
    let mut header = tar::Header::new_gnu();
    header.set_path("meta.toml")?;
    header.set_size(fs::metadata(meta_path)?.len());
    header.set_mode(0o644); // Standard file permissions
    header.set_cksum(); // Calculate checksum for the header

    let mut file = File::open(meta_path)?;
    builder.append(&header, &mut file)?;

    // Finish the tar archive
    builder.finish().context("Failed to finish tar archive")?;

    Ok(())
}

pub fn create_tar_and_checksum(source_dir: &Path) -> Result<String> {
    use crate::utils::checksum::calculate_sha256_from_reader;
    use walkdir::WalkDir;

    println!("DEBUG: create_tar_and_checksum called with source_dir: {}", source_dir.display());

    let mut tar_builder = tar::Builder::new(Vec::new());

    let mut found_files = false;
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            found_files = true;
            println!("DEBUG: Found file: {}", path.display());
            let rel_path = path.strip_prefix(source_dir).context(format!(
                "Failed to strip prefix from path {}",
                path.display()
            ))?;
            tar_builder.append_path_with_name(path, rel_path)?;
        }
    }

    if !found_files {
        println!("DEBUG: No files found in source_dir: {}", source_dir.display());
        return Err(anyhow::anyhow!("No files found in the directory to create checksum."));
    }

    let tar_data = tar_builder.into_inner().context("Failed to get tar data")?;

    calculate_sha256_from_reader(&mut &tar_data[..])
        .context("Failed to calculate checksum from tar data")
}