/// Merge functionality for combining branches
///
/// Implements three-way merge algorithm with conflict detection

pub mod base;
pub mod three_way;
pub mod types;

pub use base::find_merge_base;
pub use three_way::{merge_trees, MergeResult};
pub use types::{ConflictMarker, FileConflict, MergeStrategy};

use crate::object::{get_commit_tree, read_tree};
use std::collections::HashMap;
use std::io;

/// Represents the outcome of a merge operation
#[derive(Debug)]
pub enum MergeOutcome {
    /// Fast-forward merge (no merge commit needed)
    FastForward { from: String, to: String },
    /// Merge completed successfully without conflicts
    Success {
        merge_commit: String,
        files_changed: usize,
    },
    /// Merge has conflicts that need resolution
    Conflicts { conflicts: Vec<FileConflict> },
    /// Already up to date
    AlreadyUpToDate,
}

/// Check if fast-forward merge is possible
pub fn can_fast_forward(
    our_commit: &str,
    their_commit: &str,
) -> io::Result<bool> {
    // Check if our_commit is an ancestor of their_commit
    is_ancestor(our_commit, their_commit)
}

/// Check if commit1 is an ancestor of commit2
fn is_ancestor(ancestor: &str, descendant: &str) -> io::Result<bool> {
    if ancestor == descendant {
        return Ok(true);
    }

    // Walk back from descendant to see if we reach ancestor
    let mut to_visit = vec![descendant.to_string()];
    let mut visited = std::collections::HashSet::new();

    while let Some(current) = to_visit.pop() {
        if current == ancestor {
            return Ok(true);
        }

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        // Get parents of current commit
        let parents = get_commit_parents(&current)?;
        to_visit.extend(parents);
    }

    Ok(false)
}

/// Get parent commits of a commit
fn get_commit_parents(commit_hash: &str) -> io::Result<Vec<String>> {
    use std::fs;

    let obj_dir = &commit_hash[0..2];
    let obj_file = &commit_hash[2..];
    let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

    let compressed = fs::read(&obj_path)?;
    let content = crate::utils::decompress(&compressed)?;

    // Find null byte
    let null_pos = content
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid commit object"))?;

    let commit_data = String::from_utf8_lossy(&content[null_pos + 1..]);
    let mut parents = Vec::new();

    for line in commit_data.lines() {
        if line.starts_with("parent ") {
            parents.push(line[7..].to_string());
        } else if line.is_empty() {
            break;
        }
    }

    Ok(parents)
}

/// Get all files from a commit tree
pub fn get_commit_files(commit_hash: &str) -> io::Result<HashMap<String, String>> {
    let tree_hash = get_commit_tree(commit_hash)?;
    let mut files = HashMap::new();
    collect_tree_files(&tree_hash, "", &mut files)?;
    Ok(files)
}

/// Recursively collect files from a tree
fn collect_tree_files(
    tree_hash: &str,
    prefix: &str,
    files: &mut HashMap<String, String>,
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
            collect_tree_files(&hash_hex, &path, files)?;
        } else {
            files.insert(path, hash_hex);
        }
    }

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
    fn test_bytes_to_hex() {
        let bytes = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(bytes_to_hex(&bytes), "deadbeef");
    }
}
