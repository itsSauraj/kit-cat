use crate::index::{read_index_binary, write_index};
use crate::models::IndexEntry;
use crate::object::{get_commit_tree, read_tree};
use crate::repo::{read_head, write_head};
use std::fs;
use std::io;
use std::path::Path;

/// Checkout a branch, commit, or restore files
pub fn checkout(target: &str, force: bool) -> io::Result<()> {
    // Check if target is a branch
    let branch_path = format!(".kitkat/refs/heads/{}", target);

    if Path::new(&branch_path).exists() {
        // Checkout branch
        checkout_branch(target, force)
    } else if target.len() >= 7 && target.chars().all(|c| c.is_ascii_hexdigit()) {
        // Looks like a commit hash
        checkout_commit(target, force)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Branch or commit '{}' not found", target),
        ))
    }
}

/// Checkout a branch by name
fn checkout_branch(branch_name: &str, force: bool) -> io::Result<()> {
    let branch_path = format!(".kitkat/refs/heads/{}", branch_name);

    // Check if branch exists
    if !Path::new(&branch_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Branch '{}' does not exist", branch_name),
        ));
    }

    // Check for uncommitted changes unless force
    if !force && has_uncommitted_changes()? {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "You have uncommitted changes. Commit or stash them, or use --force",
        ));
    }

    // Get commit hash from branch
    let commit_hash = fs::read_to_string(&branch_path)?.trim().to_string();

    // Get tree from commit
    let tree_hash = get_commit_tree(&commit_hash)?;

    // Update working directory
    checkout_tree(&tree_hash)?;

    // Update index
    update_index_from_tree(&tree_hash)?;

    // Update HEAD to point to branch
    write_head(&format!("ref: refs/heads/{}", branch_name));

    println!("Switched to branch '{}'", branch_name);
    Ok(())
}

/// Checkout a commit by hash (detached HEAD)
fn checkout_commit(commit_hash: &str, force: bool) -> io::Result<()> {
    // Check for uncommitted changes unless force
    if !force && has_uncommitted_changes()? {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "You have uncommitted changes. Commit or stash them, or use --force",
        ));
    }

    // Find full commit hash (support short hashes)
    let full_hash = find_commit_hash(commit_hash)?;

    // Get tree from commit
    let tree_hash = get_commit_tree(&full_hash)?;

    // Update working directory
    checkout_tree(&tree_hash)?;

    // Update index
    update_index_from_tree(&tree_hash)?;

    // Update HEAD to point directly to commit (detached)
    write_head(&full_hash);

    println!("HEAD is now at {} (detached)", &full_hash[0..7]);
    Ok(())
}

/// Restore a specific file from the index
pub fn checkout_file(file_path: &str) -> io::Result<()> {
    let entries = read_index_binary()?;

    // Find file in index
    let entry = entries.iter().find(|e| e.path == file_path);

    match entry {
        Some(entry) => {
            // Read blob from object store
            let hash_hex = &entry.hash;
            let obj_dir = &hash_hex[0..2];
            let obj_file = &hash_hex[2..];
            let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

            let compressed = fs::read(&obj_path)?;
            let content = crate::utils::decompress(&compressed)?;

            // Find the null byte separating header from content
            let null_pos = content.iter().position(|&b| b == 0).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid blob format")
            })?;

            let file_content = &content[null_pos + 1..];

            // Write to working directory
            if let Some(parent) = Path::new(file_path).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, file_content)?;

            println!("Restored '{}' from index", file_path);
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not in index", file_path),
        )),
    }
}

/// Check if there are uncommitted changes
fn has_uncommitted_changes() -> io::Result<bool> {
    let head_content = read_head();

    // Get HEAD commit (resolve if it's a branch reference)
    let commit_hash = if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !Path::new(&branch_path).exists() {
            return Ok(false); // No commits yet, no uncommitted changes
        }

        fs::read_to_string(&branch_path)?.trim().to_string()
    } else if head_content.len() == 40 {
        head_content
    } else {
        return Ok(false); // No commits yet
    };

    // Get tree from commit
    let tree_hash = match get_commit_tree(&commit_hash) {
        Ok(hash) => hash,
        Err(_) => return Ok(false),
    };

    // Get tree entries
    let head_entries = collect_tree_entries(&tree_hash)?;

    // Get index entries
    let index_entries = read_index_binary()?;

    // Compare index with HEAD
    if index_entries.len() != head_entries.len() {
        return Ok(true);
    }

    for entry in &index_entries {
        let entry_hash_hex = &entry.hash;
        match head_entries.get(&entry.path) {
            Some(head_hash) if head_hash == entry_hash_hex => continue,
            _ => return Ok(true),
        }
    }

    // Compare working directory with index
    for entry in &index_entries {
        if !Path::new(&entry.path).exists() {
            return Ok(true);
        }

        let file_content = fs::read(&entry.path)?;
        let work_hash = hash_blob_content(&file_content);
        let index_hash = &entry.hash;

        if work_hash != *index_hash {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Update working directory to match tree
fn checkout_tree(tree_hash: &str) -> io::Result<()> {
    // Remove all tracked files from working directory
    let current_entries = read_index_binary().unwrap_or_default();
    for entry in &current_entries {
        if Path::new(&entry.path).exists() {
            fs::remove_file(&entry.path)?;
        }
    }

    // Restore files from tree
    restore_tree_recursive(tree_hash, "")?;

    Ok(())
}

/// Recursively restore files from a tree
fn restore_tree_recursive(tree_hash: &str, prefix: &str) -> io::Result<()> {
    let tree_entries = read_tree(tree_hash)?;

    for entry in tree_entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.is_tree {
            // Create directory and recurse
            fs::create_dir_all(&path)?;
            let hash_hex = bytes_to_hex(&entry.hash);
            restore_tree_recursive(&hash_hex, &path)?;
        } else {
            // Restore blob
            let hash_hex = bytes_to_hex(&entry.hash);
            restore_blob(&hash_hex, &path)?;
        }
    }

    Ok(())
}

/// Restore a blob to a file path
fn restore_blob(hash: &str, path: &str) -> io::Result<()> {
    let obj_dir = &hash[0..2];
    let obj_file = &hash[2..];
    let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

    let compressed = fs::read(&obj_path)?;
    let content = crate::utils::decompress(&compressed)?;

    // Find the null byte separating header from content
    let null_pos = content.iter().position(|&b| b == 0).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid blob format")
    })?;

    let file_content = &content[null_pos + 1..];

    // Create parent directories if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, file_content)?;
    Ok(())
}

/// Update index to match tree
fn update_index_from_tree(tree_hash: &str) -> io::Result<()> {
    let mut entries = Vec::new();
    collect_index_entries(tree_hash, "", &mut entries)?;
    write_index(&entries)?;
    Ok(())
}

/// Recursively collect index entries from tree
fn collect_index_entries(
    tree_hash: &str,
    prefix: &str,
    entries: &mut Vec<IndexEntry>,
) -> io::Result<()> {
    let tree_entries = read_tree(tree_hash)?;

    for entry in tree_entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.is_tree {
            let hash_hex = bytes_to_hex(&entry.hash);
            collect_index_entries(&hash_hex, &path, entries)?;
        } else {
            // Get file metadata
            let metadata = fs::metadata(&path)?;
            let index_entry = IndexEntry::from_file(path, bytes_to_hex(&entry.hash), &metadata);
            entries.push(index_entry);
        }
    }

    Ok(())
}

/// Collect all entries from a tree recursively
fn collect_tree_entries(tree_hash: &str) -> io::Result<std::collections::HashMap<String, String>> {
    let mut entries = std::collections::HashMap::new();
    collect_tree_entries_recursive(tree_hash, "", &mut entries)?;
    Ok(entries)
}

fn collect_tree_entries_recursive(
    tree_hash: &str,
    prefix: &str,
    entries: &mut std::collections::HashMap<String, String>,
) -> io::Result<()> {
    let tree_entries = read_tree(tree_hash)?;

    for entry in tree_entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        let hash_hex = bytes_to_hex(&entry.hash);

        if entry.is_tree {
            collect_tree_entries_recursive(&hash_hex, &path, entries)?;
        } else {
            entries.insert(path, hash_hex);
        }
    }

    Ok(())
}

/// Find full commit hash from partial hash
fn find_commit_hash(partial: &str) -> io::Result<String> {
    if partial.len() == 40 {
        return Ok(partial.to_string());
    }

    // Look in objects directory
    let prefix = &partial[0..2];
    let rest = &partial[2..];
    let obj_dir = format!(".kitkat/objects/{}", prefix);

    if let Ok(entries) = fs::read_dir(&obj_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(rest) {
                return Ok(format!("{}{}", prefix, file_name));
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Commit '{}' not found", partial),
    ))
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Hash blob content
fn hash_blob_content(content: &[u8]) -> String {
    let header = format!("blob {}\0", content.len());
    let mut data = header.as_bytes().to_vec();
    data.extend_from_slice(content);
    crate::utils::compute_hash(&data)
}
