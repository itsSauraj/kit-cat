/// Three-way merge algorithm
///
/// Merges two versions of a file given their common ancestor

use super::types::{FileConflict, FileMergeResult};
use std::collections::HashMap;
use std::fs;
use std::io;

/// Result of merging two trees
#[derive(Debug)]
pub struct MergeResult {
    /// Successfully merged files (path -> content)
    pub merged_files: HashMap<String, Vec<u8>>,
    /// Files with conflicts
    pub conflicts: Vec<FileConflict>,
    /// Files that were deleted
    pub deleted_files: Vec<String>,
}

impl MergeResult {
    /// Create a new empty merge result
    pub fn new() -> Self {
        MergeResult {
            merged_files: HashMap::new(),
            conflicts: Vec::new(),
            deleted_files: Vec::new(),
        }
    }

    /// Check if merge has any conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}

/// Merge two file trees given a common base
pub fn merge_trees(
    base_files: &HashMap<String, String>,
    our_files: &HashMap<String, String>,
    their_files: &HashMap<String, String>,
) -> io::Result<MergeResult> {
    let mut result = MergeResult::new();

    // Collect all file paths
    let mut all_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_paths.extend(base_files.keys().cloned());
    all_paths.extend(our_files.keys().cloned());
    all_paths.extend(their_files.keys().cloned());

    for path in all_paths {
        let base_hash = base_files.get(&path);
        let our_hash = our_files.get(&path);
        let their_hash = their_files.get(&path);

        match merge_file(&path, base_hash, our_hash, their_hash)? {
            FileMergeResult::Success { content } => {
                result.merged_files.insert(path, content);
            }
            FileMergeResult::Conflict { conflict } => {
                result.conflicts.push(conflict);
            }
            FileMergeResult::Unchanged => {
                // File unchanged, no action needed
            }
        }
    }

    Ok(result)
}

/// Merge a single file using three-way merge
fn merge_file(
    path: &str,
    base_hash: Option<&String>,
    our_hash: Option<&String>,
    their_hash: Option<&String>,
) -> io::Result<FileMergeResult> {
    match (base_hash, our_hash, their_hash) {
        // All three versions exist
        (Some(base), Some(ours), Some(theirs)) => {
            if ours == theirs {
                // Both sides made the same change
                return Ok(FileMergeResult::Unchanged);
            }

            if ours == base {
                // Only they modified it, take their version
                let content = read_object_content(theirs)?;
                return Ok(FileMergeResult::Success { content });
            }

            if theirs == base {
                // Only we modified it, keep our version
                let content = read_object_content(ours)?;
                return Ok(FileMergeResult::Success { content });
            }

            // Both modified differently - need to merge content
            merge_file_contents(path, base, ours, theirs)
        }

        // File added by both sides
        (None, Some(ours), Some(theirs)) => {
            if ours == theirs {
                // Same content, no conflict
                let content = read_object_content(ours)?;
                Ok(FileMergeResult::Success { content })
            } else {
                // Different content - conflict
                create_add_add_conflict(path, ours, theirs)
            }
        }

        // File deleted by one side
        (Some(_base), None, Some(theirs)) => {
            // We deleted, they modified - conflict
            create_delete_modify_conflict(path, None, Some(theirs))
        }

        (Some(_base), Some(ours), None) => {
            // They deleted, we modified - conflict
            create_delete_modify_conflict(path, Some(ours), None)
        }

        (Some(_base), None, None) => {
            // Both deleted - no conflict
            Ok(FileMergeResult::Unchanged)
        }

        // File added by one side only
        (None, Some(ours), None) => {
            // We added it
            let content = read_object_content(ours)?;
            Ok(FileMergeResult::Success { content })
        }

        (None, None, Some(theirs)) => {
            // They added it
            let content = read_object_content(theirs)?;
            Ok(FileMergeResult::Success { content })
        }

        // File doesn't exist anywhere - shouldn't happen
        (None, None, None) => Ok(FileMergeResult::Unchanged),
    }
}

/// Merge file contents when both sides modified
fn merge_file_contents(
    path: &str,
    base_hash: &str,
    our_hash: &str,
    their_hash: &str,
) -> io::Result<FileMergeResult> {
    let base_content = read_object_content(base_hash)?;
    let our_content = read_object_content(our_hash)?;
    let their_content = read_object_content(their_hash)?;

    // Check if any version is binary
    if is_binary(&base_content) || is_binary(&our_content) || is_binary(&their_content) {
        return create_binary_conflict(path, &base_content, &our_content, &their_content);
    }

    // Convert to text
    let base_text = String::from_utf8_lossy(&base_content);
    let our_text = String::from_utf8_lossy(&our_content);
    let their_text = String::from_utf8_lossy(&their_content);

    // Try to merge line by line
    match merge_text_contents(&base_text, &our_text, &their_text) {
        Some(merged) => Ok(FileMergeResult::Success {
            content: merged.into_bytes(),
        }),
        None => {
            // Content merge failed - create conflict
            let mut conflict = FileConflict::new(path.to_string());
            conflict.base_content = Some(base_content);
            conflict.our_content = Some(our_content);
            conflict.their_content = Some(their_content);
            conflict.is_binary = false;
            Ok(FileMergeResult::Conflict { conflict })
        }
    }
}

/// Merge text contents line by line
fn merge_text_contents(base: &str, ours: &str, theirs: &str) -> Option<String> {
    let base_lines: Vec<&str> = base.lines().collect();
    let our_lines: Vec<&str> = ours.lines().collect();
    let their_lines: Vec<&str> = theirs.lines().collect();

    // Simple line-by-line merge
    // If both sides changed the same line differently, return None (conflict)
    let mut merged_lines = Vec::new();
    let max_len = base_lines.len().max(our_lines.len()).max(their_lines.len());

    for i in 0..max_len {
        let base_line = base_lines.get(i).copied();
        let our_line = our_lines.get(i).copied();
        let their_line = their_lines.get(i).copied();

        match (base_line, our_line, their_line) {
            (Some(b), Some(o), Some(t)) => {
                if o == t {
                    // Same on both sides
                    merged_lines.push(o);
                } else if o == b {
                    // Only they changed it
                    merged_lines.push(t);
                } else if t == b {
                    // Only we changed it
                    merged_lines.push(o);
                } else {
                    // Both changed differently - conflict
                    return None;
                }
            }
            (None, Some(o), Some(t)) => {
                if o == t {
                    merged_lines.push(o);
                } else {
                    // Both added different lines
                    return None;
                }
            }
            (Some(_b), None, None) => {
                // Both deleted - ok
            }
            (None, Some(o), None) => {
                merged_lines.push(o);
            }
            (Some(_b), Some(o), None) => {
                merged_lines.push(o);
            }
            (None, None, Some(t)) => {
                merged_lines.push(t);
            }
            (Some(_b), None, Some(t)) => {
                merged_lines.push(t);
            }
            (None, None, None) => {}
        }
    }

    Some(merged_lines.join("\n") + "\n")
}

/// Create conflict for files added by both sides with different content
fn create_add_add_conflict(
    path: &str,
    our_hash: &str,
    their_hash: &str,
) -> io::Result<FileMergeResult> {
    let our_content = read_object_content(our_hash)?;
    let their_content = read_object_content(their_hash)?;

    let is_binary = is_binary(&our_content) || is_binary(&their_content);

    let mut conflict = FileConflict::new(path.to_string());
    conflict.base_content = None;
    conflict.our_content = Some(our_content);
    conflict.their_content = Some(their_content);
    conflict.is_binary = is_binary;

    Ok(FileMergeResult::Conflict { conflict })
}

/// Create conflict for delete/modify scenarios
fn create_delete_modify_conflict(
    path: &str,
    our_hash: Option<&String>,
    their_hash: Option<&String>,
) -> io::Result<FileMergeResult> {
    let our_content = our_hash.map(|h| read_object_content(h)).transpose()?;
    let their_content = their_hash.map(|h| read_object_content(h)).transpose()?;

    let is_binary = our_content
        .as_ref()
        .map(|c| is_binary(c))
        .or_else(|| their_content.as_ref().map(|c| is_binary(c)))
        .unwrap_or(false);

    let mut conflict = FileConflict::new(path.to_string());
    conflict.base_content = None;
    conflict.our_content = our_content;
    conflict.their_content = their_content;
    conflict.is_binary = is_binary;

    Ok(FileMergeResult::Conflict { conflict })
}

/// Create conflict for binary files
fn create_binary_conflict(
    path: &str,
    base_content: &[u8],
    our_content: &[u8],
    their_content: &[u8],
) -> io::Result<FileMergeResult> {
    let mut conflict = FileConflict::new(path.to_string());
    conflict.base_content = Some(base_content.to_vec());
    conflict.our_content = Some(our_content.to_vec());
    conflict.their_content = Some(their_content.to_vec());
    conflict.is_binary = true;

    Ok(FileMergeResult::Conflict { conflict })
}

/// Read object content from object store
fn read_object_content(hash: &str) -> io::Result<Vec<u8>> {
    let obj_dir = &hash[0..2];
    let obj_file = &hash[2..];
    let obj_path = format!(".kitcat/objects/{}/{}", obj_dir, obj_file);

    let compressed = fs::read(&obj_path)?;
    let content = crate::utils::decompress(&compressed)?;

    // Find null byte to separate header from content
    let null_pos = content
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid object"))?;

    Ok(content[null_pos + 1..].to_vec())
}

/// Check if content is binary
fn is_binary(content: &[u8]) -> bool {
    let check_size = content.len().min(8192);
    content[..check_size].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_text_no_conflict() {
        let base = "line 1\nline 2\nline 3\n";
        let ours = "line 1\nline 2 modified\nline 3\n";
        let theirs = "line 1\nline 2\nline 3 modified\n";

        let result = merge_text_contents(base, ours, theirs);
        assert!(result.is_some());

        let merged = result.unwrap();
        assert!(merged.contains("line 2 modified"));
        assert!(merged.contains("line 3 modified"));
    }

    #[test]
    fn test_merge_text_conflict() {
        let base = "line 1\nline 2\nline 3\n";
        let ours = "line 1\nline 2 our version\nline 3\n";
        let theirs = "line 1\nline 2 their version\nline 3\n";

        let result = merge_text_contents(base, ours, theirs);
        assert!(result.is_none()); // Should conflict
    }

    #[test]
    fn test_is_binary() {
        let text = b"Hello, world!";
        assert!(!is_binary(text));

        let binary = b"Hello\x00world";
        assert!(is_binary(binary));
    }
}
