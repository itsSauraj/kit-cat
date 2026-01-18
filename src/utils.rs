use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::path::Path;

/// Check if repo is initialized
pub fn is_repo_init() -> bool {
    let kitcat_path = Path::new(".kitcat");
    let head_path = Path::new(".kitcat/HEAD");
    kitcat_path.exists() && head_path.exists()
}

/// Compute SHA-1 hash of data
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compress data using Zlib
pub fn compress_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

/// Decompress data using Zlib
pub fn decompress_data(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).unwrap();
    out
}

/// Decompress data using Zlib (with Result for error handling)
pub fn decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}
