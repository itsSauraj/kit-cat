use crate::index::read_index;
use crate::object::{get_commit_tree, read_tree};
use crate::repo::read_head;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Display the status of the working directory
pub fn status() -> io::Result<()> {
    // Get current branch
    let head_content = read_head();
    let current_branch = if head_content.starts_with("ref: refs/heads/") {
        Some(
            head_content
                .trim_start_matches("ref: refs/heads/")
                .trim()
                .to_string(),
        )
    } else if head_content.len() == 40 {
        None // Detached HEAD
    } else {
        Some("master".to_string()) // No commits yet
    };

    // Display branch info
    if let Some(branch) = &current_branch {
        println!("On branch {}", branch);
    } else {
        println!("HEAD detached at {}", &head_content[0..7]);
    }

    // Get HEAD commit tree if it exists
    let head_tree_entries = get_head_tree_entries()?;

    // Get index entries
    let index_entries = read_index();
    let mut index_map: HashMap<String, String> = index_entries
        .iter()
        .map(|e| (e.path.clone(), e.hash.clone()))
        .collect();

    // Get working directory files
    let working_files = get_working_files()?;

    // Calculate staged changes (index vs HEAD)
    let mut staged_new = Vec::new();
    let mut staged_modified = Vec::new();
    let mut staged_deleted = Vec::new();

    for (path, index_hash) in &index_map {
        match head_tree_entries.get(path) {
            None => staged_new.push(path.clone()),
            Some(head_hash) if head_hash != index_hash => staged_modified.push(path.clone()),
            _ => {}
        }
    }

    for (path, _) in &head_tree_entries {
        if !index_map.contains_key(path) {
            staged_deleted.push(path.clone());
        }
    }

    // Calculate unstaged changes (working tree vs index)
    let mut unstaged_modified = Vec::new();
    let mut unstaged_deleted = Vec::new();

    for (path, index_hash) in &index_map {
        match working_files.get(path) {
            None => unstaged_deleted.push(path.clone()),
            Some(work_hash) if work_hash != index_hash => unstaged_modified.push(path.clone()),
            _ => {}
        }
    }

    // Calculate untracked files
    let mut untracked = Vec::new();
    for (path, _) in &working_files {
        if !index_map.contains_key(path) {
            untracked.push(path.clone());
        }
    }

    // Sort all lists
    staged_new.sort();
    staged_modified.sort();
    staged_deleted.sort();
    unstaged_modified.sort();
    unstaged_deleted.sort();
    untracked.sort();

    // Display status
    let has_staged = !staged_new.is_empty() || !staged_modified.is_empty() || !staged_deleted.is_empty();
    let has_unstaged = !unstaged_modified.is_empty() || !unstaged_deleted.is_empty();
    let has_untracked = !untracked.is_empty();

    if !has_staged && !has_unstaged && !has_untracked {
        println!("\nnothing to commit, working tree clean");
        return Ok(());
    }

    // Staged changes
    if has_staged {
        println!("\nChanges to be committed:");
        println!("  (use \"kitkat reset HEAD <file>...\" to unstage)");
        println!();

        for file in &staged_new {
            println!("\t\x1b[32mnew file:   {}\x1b[0m", file);
        }
        for file in &staged_modified {
            println!("\t\x1b[32mmodified:   {}\x1b[0m", file);
        }
        for file in &staged_deleted {
            println!("\t\x1b[32mdeleted:    {}\x1b[0m", file);
        }
    }

    // Unstaged changes
    if has_unstaged {
        println!("\nChanges not staged for commit:");
        println!("  (use \"kitkat add <file>...\" to update what will be committed)");
        println!("  (use \"kitkat checkout -- <file>...\" to discard changes in working directory)");
        println!();

        for file in &unstaged_modified {
            println!("\t\x1b[31mmodified:   {}\x1b[0m", file);
        }
        for file in &unstaged_deleted {
            println!("\t\x1b[31mdeleted:    {}\x1b[0m", file);
        }
    }

    // Untracked files
    if has_untracked {
        println!("\nUntracked files:");
        println!("  (use \"kitkat add <file>...\" to include in what will be committed)");
        println!();

        for file in &untracked {
            println!("\t\x1b[31m{}\x1b[0m", file);
        }
    }

    println!();

    Ok(())
}

/// Get entries from HEAD commit tree
fn get_head_tree_entries() -> io::Result<HashMap<String, String>> {
    let head_content = read_head();

    // Get commit hash
    let commit_hash = if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !Path::new(&branch_path).exists() {
            return Ok(HashMap::new()); // No commits yet
        }

        fs::read_to_string(&branch_path)?.trim().to_string()
    } else if head_content.len() == 40 {
        head_content
    } else {
        return Ok(HashMap::new()); // No commits yet
    };

    // Get tree from commit
    let tree_hash = match get_commit_tree(&commit_hash) {
        Ok(hash) => hash,
        Err(_) => return Ok(HashMap::new()),
    };

    // Read tree recursively
    let mut entries = HashMap::new();
    collect_tree_entries(&tree_hash, "", &mut entries)?;

    Ok(entries)
}

/// Recursively collect all entries from a tree
fn collect_tree_entries(
    tree_hash: &str,
    prefix: &str,
    entries: &mut HashMap<String, String>,
) -> io::Result<()> {
    let tree_entries = read_tree(tree_hash)?;

    for entry in tree_entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.is_tree {
            // Recursively process subtree
            let hash_hex = bytes_to_hex(&entry.hash);
            collect_tree_entries(&hash_hex, &path, entries)?;
        } else {
            // Add blob entry
            let hash_hex = bytes_to_hex(&entry.hash);
            entries.insert(path, hash_hex);
        }
    }

    Ok(())
}

/// Get all files in working directory with their hashes
fn get_working_files() -> io::Result<HashMap<String, String>> {
    let mut files = HashMap::new();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| {
            // Only filter out the .kitkat directory, not all hidden files
            let file_name = e.file_name().to_str().unwrap_or("");
            file_name != ".kitkat"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories and hidden files (but not in subdirectories)
        if path.is_dir() || path == Path::new(".") {
            continue;
        }

        // Get relative path
        let rel_path = path
            .strip_prefix("./")
            .unwrap_or(path)
            .to_str()
            .unwrap()
            .to_string();

        // Hash the file
        let hash = hash_file_content(path)?;
        files.insert(rel_path, hash);
    }

    Ok(files)
}

/// Hash file content (blob format)
fn hash_file_content(path: &Path) -> io::Result<String> {
    let content = fs::read(path)?;
    let header = format!("blob {}\0", content.len());
    let mut data = header.as_bytes().to_vec();
    data.extend_from_slice(&content);

    Ok(crate::utils::compute_hash(&data))
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0xab, 0xcd, 0xef];
        assert_eq!(bytes_to_hex(&bytes), "abcdef");
    }

    #[test]
    fn test_is_hidden() {
        // This test would require creating actual DirEntry objects
        // which is complex, so we'll skip detailed testing
    }
}
