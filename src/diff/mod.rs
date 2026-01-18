/// Diff module for comparing file contents and generating unified diffs
///
/// This module provides functionality for:
/// - Computing diffs between two text files (Myers algorithm)
/// - Generating unified diff format output
/// - Detecting binary files
/// - Comparing working tree, index, and commit states

pub mod algorithm;
pub mod format;
pub mod types;

// Re-export main types and functions
pub use algorithm::{compute_diff, DiffAlgorithm};
pub use format::{format_unified_diff, UnifiedDiffOptions};
pub use types::{DiffHunk, DiffLine, DiffLineType, FileDiff};

use std::fs;
use std::io;
use std::path::Path;

/// Compare two text contents and return a structured diff
pub fn diff_texts(old_content: &str, new_content: &str) -> FileDiff {
    compute_diff(old_content, new_content, DiffAlgorithm::Myers)
}

/// Compare two files and return a structured diff
pub fn diff_files(old_path: &Path, new_path: &Path) -> io::Result<FileDiff> {
    let old_content = fs::read(old_path)?;
    let new_content = fs::read(new_path)?;

    // Check if files are binary
    if is_binary(&old_content) || is_binary(&new_content) {
        return Ok(FileDiff {
            old_path: old_path.to_string_lossy().to_string(),
            new_path: new_path.to_string_lossy().to_string(),
            is_binary: true,
            hunks: vec![],
        });
    }

    let old_text = String::from_utf8_lossy(&old_content);
    let new_text = String::from_utf8_lossy(&new_content);

    let mut diff = compute_diff(&old_text, &new_text, DiffAlgorithm::Myers);
    diff.old_path = old_path.to_string_lossy().to_string();
    diff.new_path = new_path.to_string_lossy().to_string();

    Ok(diff)
}

/// Check if content appears to be binary
pub fn is_binary(content: &[u8]) -> bool {
    // Simple heuristic: if we find a null byte in the first 8KB, it's likely binary
    let check_size = content.len().min(8192);
    content[..check_size].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary() {
        assert!(!is_binary(b"Hello, world!"));
        assert!(!is_binary(b"Text with\nnewlines\nand\ttabs"));
        assert!(is_binary(b"Binary\0content"));
        assert!(is_binary(&[0xFF, 0xD8, 0xFF, 0xE0])); // JPEG header
    }

    #[test]
    fn test_diff_texts_simple() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\n";

        let diff = diff_texts(old, new);
        assert!(!diff.is_binary);
        assert!(!diff.hunks.is_empty());
    }

    #[test]
    fn test_diff_texts_additions() {
        let old = "line 1\nline 2\n";
        let new = "line 1\nline 2\nline 3\n";

        let diff = diff_texts(old, new);
        assert_eq!(diff.hunks.len(), 1);
    }

    #[test]
    fn test_diff_texts_deletions() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 3\n";

        let diff = diff_texts(old, new);
        assert_eq!(diff.hunks.len(), 1);
    }
}
