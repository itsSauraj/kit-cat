use crate::models::IndexEntry;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Read the index from binary format
pub fn read_index_binary() -> io::Result<Vec<IndexEntry>> {
    let index_path = Path::new(".kitkat/index");

    if !index_path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read(index_path)?;
    parse_index(&data)
}

/// Parse the binary index data
fn parse_index(data: &[u8]) -> io::Result<Vec<IndexEntry>> {
    if data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Index file too small",
        ));
    }

    // Check header
    if &data[0..4] != b"DIRC" {
        // Try reading as legacy text format for backward compatibility
        return read_text_format(data);
    }

    // Read version
    let version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported index version: {}", version),
        ));
    }

    // Read number of entries
    let num_entries = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let mut entries = Vec::with_capacity(num_entries);
    let mut offset = 12;

    // Read each entry
    for _ in 0..num_entries {
        let (entry, entry_size) = parse_entry(&data[offset..])?;
        entries.push(entry);
        offset += entry_size;
    }

    Ok(entries)
}

/// Parse a single index entry
fn parse_entry(data: &[u8]) -> io::Result<(IndexEntry, usize)> {
    if data.len() < 62 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Entry data too small",
        ));
    }

    let mut offset = 0;

    // Read fixed-size fields
    let ctime_sec = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    offset += 4;

    let ctime_nsec = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    offset += 4;

    let mtime_sec = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    offset += 4;

    let mtime_nsec = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
    offset += 4;

    let dev = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    offset += 4;

    let ino = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    offset += 4;

    let mode = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
    offset += 4;

    let uid = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
    offset += 4;

    let gid = u32::from_be_bytes([data[32], data[33], data[34], data[35]]);
    offset += 4;

    let size = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);
    offset += 4;

    // Read hash (20 bytes)
    let hash = bytes_to_hex(&data[40..60]);
    offset += 20;

    // Read flags
    let flags = u16::from_be_bytes([data[60], data[61]]);
    offset += 2;

    // Read path (null-terminated)
    let path_end = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Path not null-terminated"))?;

    let path = String::from_utf8_lossy(&data[offset..offset + path_end]).to_string();
    offset += path_end + 1; // +1 for null terminator

    // Skip padding to 8-byte boundary
    let entry_size = 62 + path_end + 1;
    let padding = (8 - (entry_size % 8)) % 8;
    offset += padding;

    let entry = IndexEntry {
        ctime_sec,
        ctime_nsec,
        mtime_sec,
        mtime_nsec,
        dev,
        ino,
        mode,
        uid,
        gid,
        size,
        hash,
        flags,
        path,
    };

    Ok((entry, offset))
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Legacy text format reader (for backward compatibility)
fn read_text_format(data: &[u8]) -> io::Result<Vec<IndexEntry>> {
    let text = String::from_utf8_lossy(data);
    let entries = text
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(IndexEntry {
                    ctime_sec: 0,
                    ctime_nsec: 0,
                    mtime_sec: 0,
                    mtime_nsec: 0,
                    dev: 0,
                    ino: 0,
                    mode: 0o100644,
                    uid: 0,
                    gid: 0,
                    size: 0,
                    hash: parts[0].to_string(),
                    flags: std::cmp::min(parts[1].len(), 0xFFF) as u16,
                    path: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(entries)
}

/// Read the index (wrapper function for compatibility)
pub fn read_index() -> Vec<IndexEntry> {
    read_index_binary().unwrap_or_default()
}
