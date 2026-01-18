use crate::models::IndexEntry;
use crate::utils::compute_hash;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Write the index to disk in binary DIRC format
///
/// Format:
/// - Header: "DIRC" (4 bytes)
/// - Version: 2 (4 bytes, big-endian)
/// - Number of entries (4 bytes, big-endian)
/// - Entries (sorted by path)
/// - SHA-1 checksum of entire index (20 bytes)
pub fn write_index(entries: &[IndexEntry]) -> io::Result<()> {
    let index_path = Path::new(".kitcat/index");

    // Create a temporary file first for atomic write
    let temp_path = Path::new(".kitcat/index.lock");

    // Open the lock file with exclusive access
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp_path)?;

    // Lock the file for exclusive access
    file.lock_exclusive()?;

    let mut buffer = Vec::new();

    // Write header
    buffer.extend_from_slice(b"DIRC");

    // Write version (2)
    buffer.extend_from_slice(&2u32.to_be_bytes());

    // Write number of entries
    buffer.extend_from_slice(&(entries.len() as u32).to_be_bytes());

    // Sort entries by path (Git requirement)
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by(|a, b| a.path.cmp(&b.path));

    // Write each entry
    for entry in &sorted_entries {
        write_entry(&mut buffer, entry)?;
    }

    // Compute SHA-1 checksum of the entire index content
    let checksum = compute_hash(&buffer);
    let checksum_bytes = hex_to_bytes(&checksum);
    buffer.extend_from_slice(&checksum_bytes);

    // Write all data to the lock file
    file.write_all(&buffer)?;
    file.sync_all()?;

    // Unlock and close the file
    file.unlock()?;
    drop(file);

    // Atomically rename the lock file to the index file
    std::fs::rename(temp_path, index_path)?;

    Ok(())
}

/// Write a single index entry to the buffer
fn write_entry(buffer: &mut Vec<u8>, entry: &IndexEntry) -> io::Result<()> {
    // ctime seconds
    buffer.extend_from_slice(&entry.ctime_sec.to_be_bytes());
    // ctime nanoseconds
    buffer.extend_from_slice(&entry.ctime_nsec.to_be_bytes());

    // mtime seconds
    buffer.extend_from_slice(&entry.mtime_sec.to_be_bytes());
    // mtime nanoseconds
    buffer.extend_from_slice(&entry.mtime_nsec.to_be_bytes());

    // dev
    buffer.extend_from_slice(&entry.dev.to_be_bytes());
    // ino
    buffer.extend_from_slice(&entry.ino.to_be_bytes());

    // mode
    buffer.extend_from_slice(&entry.mode.to_be_bytes());

    // uid
    buffer.extend_from_slice(&entry.uid.to_be_bytes());
    // gid
    buffer.extend_from_slice(&entry.gid.to_be_bytes());

    // size
    buffer.extend_from_slice(&entry.size.to_be_bytes());

    // hash (20 bytes)
    let hash_bytes = hex_to_bytes(&entry.hash);
    buffer.extend_from_slice(&hash_bytes);

    // flags
    buffer.extend_from_slice(&entry.flags.to_be_bytes());

    // path (null-terminated)
    buffer.extend_from_slice(entry.path.as_bytes());
    buffer.push(0); // null terminator

    // Padding to align to 8-byte boundary
    // Entry size = 62 bytes (fixed) + path length + 1 (null)
    let entry_size = 62 + entry.path.len() + 1;
    let padding = (8 - (entry_size % 8)) % 8;
    for _ in 0..padding {
        buffer.push(0);
    }

    Ok(())
}

/// Convert a hex string to bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Add or update a single file in the index
pub fn add_file_to_index(file_path: &str, hash: &str) -> io::Result<()> {
    use crate::index::read_index::read_index_binary;

    // Read existing index
    let mut entries = match read_index_binary() {
        Ok(entries) => entries,
        Err(_) => Vec::new(), // No existing index
    };

    // Get file metadata
    let metadata = std::fs::metadata(file_path)?;

    // Normalize path to be relative to repository root
    let repo_root = std::env::current_dir()?;
    let abs_path = repo_root.join(file_path);
    let rel_path = abs_path
        .strip_prefix(&repo_root)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Create new entry
    let new_entry = IndexEntry::from_file(rel_path.clone(), hash.to_string(), &metadata);

    // Remove existing entry for this path if it exists
    entries.retain(|e| e.path != rel_path);

    // Add new entry
    entries.push(new_entry);

    // Write updated index
    write_index(&entries)?;

    Ok(())
}
