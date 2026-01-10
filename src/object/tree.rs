use crate::models::{IndexEntry, TreeEntry};
use crate::utils::{compress_data, compute_hash, decompress_data};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

/// Build a tree from the current index
pub fn write_tree_from_index(entries: &[IndexEntry]) -> io::Result<String> {
    // Build tree structure from flat index
    let root = build_tree_structure(entries);

    // Write the root tree recursively
    write_tree_recursive(&root)
}

/// Represents a node in the tree structure
#[derive(Debug)]
struct TreeNode {
    /// Child files (name -> hash)
    files: BTreeMap<String, (u32, String)>, // (mode, hash)
    /// Child directories (name -> TreeNode)
    directories: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
        }
    }
}

/// Build a tree structure from flat index entries
fn build_tree_structure(entries: &[IndexEntry]) -> TreeNode {
    let mut root = TreeNode::new();

    for entry in entries {
        let path_parts: Vec<&str> = entry.path.split('/').collect();
        insert_entry(&mut root, &path_parts, entry);
    }

    root
}

/// Insert an entry into the tree structure
fn insert_entry(node: &mut TreeNode, path_parts: &[&str], entry: &IndexEntry) {
    if path_parts.len() == 1 {
        // This is a file in the current directory
        node.files.insert(
            path_parts[0].to_string(),
            (entry.mode, entry.hash.clone()),
        );
    } else {
        // This is in a subdirectory
        let dir_name = path_parts[0];
        let child = node
            .directories
            .entry(dir_name.to_string())
            .or_insert_with(TreeNode::new);
        insert_entry(child, &path_parts[1..], entry);
    }
}

/// Write a tree node recursively and return its hash
fn write_tree_recursive(node: &TreeNode) -> io::Result<String> {
    let mut entries = Vec::new();

    // First, write all subdirectories and get their hashes
    for (name, child) in &node.directories {
        let child_hash = write_tree_recursive(child)?;
        entries.push((0o040000u32, name.clone(), child_hash)); // 040000 = directory mode
    }

    // Then add all files
    for (name, (mode, hash)) in &node.files {
        entries.push((*mode, name.clone(), hash.clone()));
    }

    // Sort entries by name (Git requirement)
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    // Build tree content
    let mut content = Vec::new();
    for (mode, name, hash) in entries {
        // Format: "<mode> <name>\0<20-byte-hash>"
        content.extend_from_slice(format!("{:o} {}\0", mode, name).as_bytes());

        // Convert hex hash to bytes
        let hash_bytes = hex_to_bytes(&hash);
        content.extend_from_slice(&hash_bytes);
    }

    // Create tree object: "tree <size>\0<content>"
    let header = format!("tree {}\0", content.len());
    let mut full_content = header.as_bytes().to_vec();
    full_content.extend_from_slice(&content);

    // Compute hash
    let hash = compute_hash(&full_content);

    // Store object
    store_object(&hash, &full_content)?;

    Ok(hash)
}

/// Store an object in the .kitkat/objects directory
fn store_object(hash: &str, content: &[u8]) -> io::Result<()> {
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];

    let dir_path = Path::new(".kitkat/objects").join(dir_name);
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(file_name);

    // Don't overwrite if object already exists
    if file_path.exists() {
        return Ok(());
    }

    // Compress and write
    let compressed = compress_data(content);
    fs::write(file_path, compressed)?;

    Ok(())
}

/// Read a tree object and return its entries
pub fn read_tree(hash: &str) -> io::Result<Vec<TreeEntry>> {
    let content = read_object_content(hash)?;
    parse_tree_content(&content)
}

/// Read an object's content
fn read_object_content(hash: &str) -> io::Result<Vec<u8>> {
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];

    let file_path = Path::new(".kitkat/objects")
        .join(dir_name)
        .join(file_name);

    if !file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Object {} not found", hash),
        ));
    }

    let compressed = fs::read(file_path)?;
    let decompressed = decompress_data(&compressed);

    Ok(decompressed)
}

/// Parse tree content into entries
fn parse_tree_content(data: &[u8]) -> io::Result<Vec<TreeEntry>> {
    // Find the null byte after the header
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid tree object"))?;

    // Verify this is a tree object
    let header = String::from_utf8_lossy(&data[0..null_pos]);
    if !header.starts_with("tree ") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a tree object",
        ));
    }

    let mut entries = Vec::new();
    let mut offset = null_pos + 1;

    while offset < data.len() {
        // Parse mode and name (format: "<mode> <name>\0")
        let space_pos = data[offset..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid tree entry"))?;

        let mode = String::from_utf8_lossy(&data[offset..offset + space_pos]).to_string();
        offset += space_pos + 1;

        let null_pos = data[offset..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid tree entry"))?;

        let name = String::from_utf8_lossy(&data[offset..offset + null_pos]).to_string();
        offset += null_pos + 1;

        // Read 20-byte hash
        if offset + 20 > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incomplete hash in tree entry",
            ));
        }

        let mut hash = [0u8; 20];
        hash.copy_from_slice(&data[offset..offset + 20]);
        offset += 20;

        let is_tree = mode.starts_with("40000") || mode.starts_with("040000");

        entries.push(TreeEntry {
            mode,
            name,
            hash,
            is_tree,
        });
    }

    Ok(entries)
}

/// List the contents of a tree (for debugging/display)
pub fn list_tree(hash: &str, prefix: &str) -> io::Result<()> {
    let entries = read_tree(hash)?;

    for entry in entries {
        let hash_hex = bytes_to_hex(&entry.hash);
        let entry_type = if entry.is_tree { "tree" } else { "blob" };

        println!(
            "{}{} {} {} {}",
            prefix, entry.mode, entry_type, hash_hex, entry.name
        );

        // Recursively list subdirectories
        if entry.is_tree {
            let new_prefix = format!("{}  ", prefix);
            list_tree(&hash_hex, &new_prefix)?;
        }
    }

    Ok(())
}

/// Convert hex string to bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Restore a tree to the working directory
pub fn checkout_tree(hash: &str, target_dir: &Path) -> io::Result<()> {
    let entries = read_tree(hash)?;

    for entry in entries {
        let entry_path = target_dir.join(&entry.name);
        let hash_hex = bytes_to_hex(&entry.hash);

        if entry.is_tree {
            // Create directory and recursively checkout
            fs::create_dir_all(&entry_path)?;
            checkout_tree(&hash_hex, &entry_path)?;
        } else {
            // Restore file
            let blob_content = read_blob_content(&hash_hex)?;
            fs::write(&entry_path, blob_content)?;

            // Set file permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = u32::from_str_radix(&entry.mode, 8) {
                    let perms = fs::Permissions::from_mode(mode);
                    fs::set_permissions(&entry_path, perms)?;
                }
            }
        }
    }

    Ok(())
}

/// Read blob content (file data)
fn read_blob_content(hash: &str) -> io::Result<Vec<u8>> {
    let content = read_object_content(hash)?;

    // Find the null byte after the header
    let null_pos = content
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid blob object"))?;

    // Return content after the header
    Ok(content[null_pos + 1..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new();
        assert_eq!(node.files.len(), 0);
        assert_eq!(node.directories.len(), 0);
    }

    #[test]
    fn test_hex_conversion() {
        let hex = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3";
        let bytes = hex_to_bytes(hex);
        assert_eq!(bytes.len(), 20);

        let hex2 = bytes_to_hex(&bytes);
        assert_eq!(hex, hex2);
    }
}
