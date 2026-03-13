use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use twox_hash::XxHash64;
use std::hash::Hasher;

/// Computes a fast sparse hash of a large media file by reading:
/// - First 1MB
/// - Middle 1MB
/// - Last 1MB
pub fn compute_sparse_hash(file_path: &Path) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    let mut hasher = XxHash64::with_seed(0);

    // We sample up to 1MB chunks
    let chunk_size: u64 = 1024 * 1024;
    let mut buffer = vec![0u8; chunk_size as usize];

    if file_size <= chunk_size * 3 {
        // If file is smaller than 3MB, just read the whole thing
        let mut full_buffer = Vec::new();
        file.read_to_end(&mut full_buffer)?;
        hasher.write(&full_buffer);
    } else {
        // 1. Read first 1MB
        let n1 = file.read(&mut buffer)?;
        hasher.write(&buffer[..n1]);

        // 2. Read middle 1MB
        let middle_offset = file_size / 2 - chunk_size / 2;
        file.seek(SeekFrom::Start(middle_offset))?;
        let n2 = file.read(&mut buffer)?;
        hasher.write(&buffer[..n2]);

        // 3. Read last 1MB
        let last_offset = file_size - chunk_size;
        file.seek(SeekFrom::Start(last_offset))?;
        let n3 = file.read(&mut buffer)?;
        hasher.write(&buffer[..n3]);
    }

    Ok(format!("{:016x}", hasher.finish()))
}
