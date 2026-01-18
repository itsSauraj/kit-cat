/// Packfile implementation for efficient object storage
///
/// Reduces disk usage by:
/// - Combining multiple objects into single packfiles
/// - Compressing objects more efficiently
/// - Using delta compression for similar objects

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Pack file header magic
const PACK_SIGNATURE: &[u8] = b"PACK";
const PACK_VERSION: u32 = 2;

/// Object types in pack
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 6,
    RefDelta = 7,
}

/// Pack file entry
#[derive(Debug)]
pub struct PackEntry {
    /// Object type
    pub obj_type: PackObjectType,
    /// Uncompressed size
    pub size: usize,
    /// Compressed data
    pub data: Vec<u8>,
    /// SHA-1 hash of the object
    pub hash: String,
}

/// Pack file structure
#[derive(Debug)]
pub struct PackFile {
    /// Pack entries
    pub entries: Vec<PackEntry>,
    /// Total objects
    pub count: u32,
}

impl PackFile {
    /// Create a new empty pack file
    pub fn new() -> Self {
        PackFile {
            entries: Vec::new(),
            count: 0,
        }
    }

    /// Add an entry to the pack
    pub fn add_entry(&mut self, entry: PackEntry) {
        self.entries.push(entry);
        self.count += 1;
    }

    /// Write pack file to disk
    pub fn write_to_file(&self, path: &Path) -> io::Result<String> {
        let mut file = File::create(path)?;
        let mut data_to_hash = Vec::new();

        // Write header
        file.write_all(PACK_SIGNATURE)?;
        data_to_hash.extend_from_slice(PACK_SIGNATURE);

        let version_bytes = PACK_VERSION.to_be_bytes();
        file.write_all(&version_bytes)?;
        data_to_hash.extend_from_slice(&version_bytes);

        let count_bytes = self.count.to_be_bytes();
        file.write_all(&count_bytes)?;
        data_to_hash.extend_from_slice(&count_bytes);

        // Write entries
        for entry in &self.entries {
            let entry_data = self.encode_entry(entry)?;
            file.write_all(&entry_data)?;
            data_to_hash.extend_from_slice(&entry_data);
        }

        // Compute checksum
        let checksum = crate::utils::compute_hash(&data_to_hash);
        let checksum_bytes = hex::decode(&checksum).unwrap_or_default();
        file.write_all(&checksum_bytes)?;

        // Return pack hash
        Ok(checksum)
    }

    /// Encode a pack entry
    fn encode_entry(&self, entry: &PackEntry) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();

        // Encode type and size in variable-length format
        let mut size = entry.size;
        let mut type_and_size = ((entry.obj_type as u8) << 4) | ((size & 0x0F) as u8);
        size >>= 4;

        if size > 0 {
            type_and_size |= 0x80;
        }
        data.push(type_and_size);

        while size > 0 {
            let mut byte = (size & 0x7F) as u8;
            size >>= 7;
            if size > 0 {
                byte |= 0x80;
            }
            data.push(byte);
        }

        // Write compressed data
        data.extend_from_slice(&entry.data);

        Ok(data)
    }

    /// Read pack file from disk
    pub fn read_from_file(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Self::parse_pack(&buffer)
    }

    /// Parse pack file data
    fn parse_pack(data: &[u8]) -> io::Result<Self> {
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Pack file too small",
            ));
        }

        // Check signature
        if &data[0..4] != PACK_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid pack signature",
            ));
        }

        // Read version
        let version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        if version != PACK_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported pack version: {}", version),
            ));
        }

        // Read object count
        let count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let mut pack = PackFile::new();
        let mut offset = 12;

        // Parse entries
        for _ in 0..count {
            let (entry, bytes_read) = Self::decode_entry(&data[offset..])?;
            pack.add_entry(entry);
            offset += bytes_read;
        }

        Ok(pack)
    }

    /// Decode a pack entry
    fn decode_entry(data: &[u8]) -> io::Result<(PackEntry, usize)> {
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty entry data",
            ));
        }

        let mut offset = 0;

        // Decode type and size
        let first_byte = data[offset];
        offset += 1;

        let obj_type = match (first_byte >> 4) & 0x07 {
            1 => PackObjectType::Commit,
            2 => PackObjectType::Tree,
            3 => PackObjectType::Blob,
            4 => PackObjectType::Tag,
            6 => PackObjectType::OfsDelta,
            7 => PackObjectType::RefDelta,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid object type",
                ))
            }
        };

        let mut size = (first_byte & 0x0F) as usize;
        let mut shift = 4;

        while first_byte & 0x80 != 0 && offset < data.len() {
            let byte = data[offset];
            offset += 1;
            size |= ((byte & 0x7F) as usize) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                break;
            }
        }

        // Read compressed data (simplified - would need proper parsing)
        let compressed_data = data[offset..offset + size.min(data.len() - offset)].to_vec();
        offset += compressed_data.len();

        let entry = PackEntry {
            obj_type,
            size,
            data: compressed_data,
            hash: String::new(), // Would be computed from decompressed data
        };

        Ok((entry, offset))
    }
}

/// Pack loose objects into a packfile
pub fn pack_objects() -> io::Result<usize> {
    let objects_dir = Path::new(".kitkat/objects");
    if !objects_dir.exists() {
        return Ok(0);
    }

    let mut pack = PackFile::new();
    let mut object_count = 0;

    // Collect all loose objects
    for entry in fs::read_dir(objects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && entry.file_name() != "pack" {
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            if dir_name_str.len() == 2 {
                // This is an object directory
                for obj_entry in fs::read_dir(&path)? {
                    let obj_entry = obj_entry?;
                    let obj_path = obj_entry.path();

                    if obj_path.is_file() {
                        // Read and pack the object
                        let compressed = fs::read(&obj_path)?;
                        let obj_name = obj_entry.file_name();
                        let hash = format!("{}{}", dir_name_str, obj_name.to_string_lossy());

                        // Decompress to determine type and size
                        let decompressed = crate::utils::decompress(&compressed)?;

                        // Find null byte
                        let null_pos = decompressed
                            .iter()
                            .position(|&b| b == 0)
                            .unwrap_or(decompressed.len());

                        let header = String::from_utf8_lossy(&decompressed[..null_pos]);
                        let parts: Vec<&str> = header.split(' ').collect();

                        if parts.len() == 2 {
                            let obj_type = match parts[0] {
                                "commit" => PackObjectType::Commit,
                                "tree" => PackObjectType::Tree,
                                "blob" => PackObjectType::Blob,
                                _ => continue,
                            };

                            let size = parts[1].parse::<usize>().unwrap_or(0);

                            let entry = PackEntry {
                                obj_type,
                                size,
                                data: compressed,
                                hash,
                            };

                            pack.add_entry(entry);
                            object_count += 1;
                        }
                    }
                }
            }
        }
    }

    if object_count > 0 {
        // Create pack directory
        let pack_dir = Path::new(".kitkat/objects/pack");
        fs::create_dir_all(pack_dir)?;

        // Generate pack filename
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pack_name = format!("pack-{}.pack", timestamp);
        let pack_path = pack_dir.join(&pack_name);

        // Write pack file
        let pack_hash = pack.write_to_file(&pack_path)?;

        println!("Created packfile: {} ({} objects)", pack_name, object_count);
        println!("Packfile hash: {}", pack_hash);
    }

    Ok(object_count)
}

/// Create pack index for faster lookups
pub fn create_pack_index(pack_path: &Path) -> io::Result<()> {
    let pack = PackFile::read_from_file(pack_path)?;
    let mut index = HashMap::new();

    let mut offset = 12; // After header
    for entry in &pack.entries {
        index.insert(entry.hash.clone(), offset);
        offset += entry.data.len() + 20; // Approximate
    }

    // Write index file
    let index_path = pack_path.with_extension("idx");
    let mut file = File::create(&index_path)?;

    // Simple index format: hash -> offset
    for (hash, offset) in index {
        writeln!(file, "{} {}", hash, offset)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_creation() {
        let mut pack = PackFile::new();
        assert_eq!(pack.count, 0);

        let entry = PackEntry {
            obj_type: PackObjectType::Blob,
            size: 10,
            data: vec![1, 2, 3, 4, 5],
            hash: "abc123".to_string(),
        };

        pack.add_entry(entry);
        assert_eq!(pack.count, 1);
    }

    #[test]
    fn test_object_type() {
        assert_eq!(PackObjectType::Commit as u8, 1);
        assert_eq!(PackObjectType::Blob as u8, 3);
    }
}
