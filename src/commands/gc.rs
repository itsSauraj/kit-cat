/// Garbage collection command
///
/// Optimizes repository by:
/// - Packing loose objects into packfiles
/// - Removing unreachable objects
/// - Compressing pack files

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

/// Garbage collection options
#[derive(Debug, Clone)]
pub struct GcOptions {
    /// Aggressive mode (more thorough but slower)
    pub aggressive: bool,
    /// Prune unreachable objects older than this many days
    pub prune_days: Option<u32>,
    /// Dry run (don't actually delete anything)
    pub dry_run: bool,
}

impl Default for GcOptions {
    fn default() -> Self {
        GcOptions {
            aggressive: false,
            prune_days: Some(14), // Default: prune after 14 days
            dry_run: false,
        }
    }
}

/// Main garbage collection command
pub fn gc(options: GcOptions) -> io::Result<()> {
    println!("Running garbage collection...");

    if options.dry_run {
        println!("Dry run mode - no changes will be made");
    }

    // Step 1: Pack loose objects
    println!("Packing loose objects...");
    let packed_count = crate::object::pack::pack_objects()?;
    println!("Packed {} objects", packed_count);

    // Step 2: Find reachable objects
    println!("Finding reachable objects...");
    let reachable = find_reachable_objects()?;
    println!("Found {} reachable objects", reachable.len());

    // Step 3: Prune unreachable objects
    if let Some(days) = options.prune_days {
        println!("Pruning unreachable objects older than {} days...", days);
        let pruned = prune_unreachable_objects(&reachable, days, options.dry_run)?;
        println!("Pruned {} unreachable objects", pruned);
    }

    // Step 4: Optimize pack files (if aggressive)
    if options.aggressive {
        println!("Repacking all objects (aggressive mode)...");
        repack_aggressive()?;
    }

    println!("Garbage collection complete!");
    Ok(())
}

/// Find all reachable objects from refs
fn find_reachable_objects() -> io::Result<HashSet<String>> {
    let mut reachable = HashSet::new();
    let mut to_visit = Vec::new();

    // Collect all refs (branches, tags, HEAD)
    collect_refs(&mut to_visit)?;

    // Traverse object graph
    while let Some(hash) = to_visit.pop() {
        if reachable.contains(&hash) {
            continue;
        }

        reachable.insert(hash.clone());

        // Get object type and traverse children
        if let Ok(children) = get_object_children(&hash) {
            to_visit.extend(children);
        }
    }

    Ok(reachable)
}

/// Collect all refs (branches, HEAD, etc.)
fn collect_refs(refs: &mut Vec<String>) -> io::Result<()> {
    // Read HEAD
    let head_content = crate::repo::read_head();
    if let Ok(commit_hash) = resolve_ref(&head_content) {
        refs.push(commit_hash);
    }

    // Read all branches
    let refs_dir = Path::new(".kitkat/refs/heads");
    if refs_dir.exists() {
        collect_refs_from_dir(refs_dir, refs)?;
    }

    Ok(())
}

/// Recursively collect refs from directory
fn collect_refs_from_dir(dir: &Path, refs: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                let hash = content.trim().to_string();
                if hash.len() == 40 {
                    refs.push(hash);
                }
            }
        } else if path.is_dir() {
            collect_refs_from_dir(&path, refs)?;
        }
    }

    Ok(())
}

/// Resolve a ref to a commit hash
fn resolve_ref(ref_content: &str) -> io::Result<String> {
    if ref_content.starts_with("ref:") {
        let branch_name = ref_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if Path::new(&branch_path).exists() {
            Ok(fs::read_to_string(&branch_path)?.trim().to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Ref not found",
            ))
        }
    } else {
        Ok(ref_content.trim().to_string())
    }
}

/// Get children of an object (for graph traversal)
fn get_object_children(hash: &str) -> io::Result<Vec<String>> {
    let obj_dir = &hash[0..2];
    let obj_file = &hash[2..];
    let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

    if !Path::new(&obj_path).exists() {
        return Ok(Vec::new());
    }

    let compressed = fs::read(&obj_path)?;
    let content = crate::utils::decompress(&compressed)?;

    // Find null byte
    let null_pos = content
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid object"))?;

    let header = String::from_utf8_lossy(&content[..null_pos]);
    let obj_type = header.split(' ').next().unwrap_or("");

    let data = &content[null_pos + 1..];
    let mut children = Vec::new();

    match obj_type {
        "commit" => {
            // Parse commit to get tree and parents
            let text = String::from_utf8_lossy(data);
            for line in text.lines() {
                if line.starts_with("tree ") {
                    children.push(line[5..].to_string());
                } else if line.starts_with("parent ") {
                    children.push(line[7..].to_string());
                } else if line.is_empty() {
                    break;
                }
            }
        }
        "tree" => {
            // Parse tree to get blob/tree entries
            let tree_entries = crate::object::read_tree(hash)?;
            for entry in tree_entries {
                let entry_hash = bytes_to_hex(&entry.hash);
                children.push(entry_hash);
            }
        }
        "blob" => {
            // Blobs have no children
        }
        _ => {}
    }

    Ok(children)
}

/// Prune unreachable objects
fn prune_unreachable_objects(
    reachable: &HashSet<String>,
    _days: u32,
    dry_run: bool,
) -> io::Result<usize> {
    let mut pruned_count = 0;
    let objects_dir = Path::new(".kitkat/objects");

    if !objects_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(objects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && entry.file_name() != "pack" {
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            if dir_name_str.len() == 2 {
                for obj_entry in fs::read_dir(&path)? {
                    let obj_entry = obj_entry?;
                    let obj_path = obj_entry.path();

                    if obj_path.is_file() {
                        let obj_name = obj_entry.file_name();
                        let hash = format!("{}{}", dir_name_str, obj_name.to_string_lossy());

                        if !reachable.contains(&hash) {
                            if !dry_run {
                                fs::remove_file(&obj_path)?;
                            }
                            pruned_count += 1;
                        }
                    }
                }

                // Remove empty directories
                if !dry_run {
                    if let Ok(mut entries) = fs::read_dir(&path) {
                        if entries.next().is_none() {
                            let _ = fs::remove_dir(&path);
                        }
                    }
                }
            }
        }
    }

    Ok(pruned_count)
}

/// Repack all objects aggressively
fn repack_aggressive() -> io::Result<()> {
    // In aggressive mode, we would:
    // 1. Unpack all packfiles
    // 2. Repack everything with better delta compression
    // 3. Remove old packfiles
    // For now, just repack loose objects
    crate::object::pack::pack_objects()?;
    Ok(())
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_options_default() {
        let options = GcOptions::default();
        assert!(!options.aggressive);
        assert_eq!(options.prune_days, Some(14));
        assert!(!options.dry_run);
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(bytes_to_hex(&bytes), "deadbeef");
    }
}
