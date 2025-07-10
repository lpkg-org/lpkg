use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn calculate_sha256(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    let mut file = File::open(path).context(format!(
        "Failed to open file for checksum calculation: {}",
        file_path
    ))?;

    calculate_sha256_from_reader(&mut file)
}

pub fn calculate_sha256_from_reader<R: Read>(reader: &mut R) -> Result<String> {
    let mut sha256 = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = reader
            .read(&mut buffer)
            .context("Failed to read from reader for checksum")?;
        if n == 0 {
            break;
        }
        sha256.update(&buffer[..n]);
    }

    let result = sha256.finalize();
    Ok(format!("{:x}", result))
}

#[allow(dead_code)]
pub fn verify_checksum(file_path: &str, expected_checksum: &str) -> Result<bool> {
    let calculated =
        calculate_sha256(file_path).context("Failed to calculate checksum for verification")?;
    Ok(calculated == expected_checksum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::fs;

    #[test]
    fn test_calculate_sha256_from_reader() {
        let data = b"hello world";
        let mut reader = Cursor::new(data);
        let checksum = calculate_sha256_from_reader(&mut reader).unwrap();
        // SHA256 hash of "hello world"
        assert_eq!(checksum, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_calculate_sha256_for_file() {
        let file_path = "test_file_for_checksum.txt";
        let content = "This is a test file.";
        fs::write(file_path, content).unwrap();

        let checksum = calculate_sha256(file_path).unwrap();
        // SHA256 hash of "This is a test file."
        assert_eq!(checksum, "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de");

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_verify_checksum_success() {
        let file_path = "test_file_for_verify_success.txt";
        let content = "Verify me!";
        let _expected_checksum = "f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de";
        fs::write(file_path, content).unwrap();

        let actual_checksum = calculate_sha256(file_path).unwrap();
        assert!(verify_checksum(file_path, &actual_checksum).unwrap());

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_verify_checksum_failure() {
        let file_path = "test_file_for_verify_failure.txt";
        let content = "Verify me!";
        fs::write(file_path, content).unwrap();

        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!verify_checksum(file_path, wrong_checksum).unwrap());

        fs::remove_file(file_path).unwrap();
    }
}
