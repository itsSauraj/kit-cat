/// Types for merge operations

use std::fmt;

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Standard three-way merge
    ThreeWay,
    /// Ours: in conflict, take our version
    Ours,
    /// Theirs: in conflict, take their version
    Theirs,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::ThreeWay
    }
}

/// Represents a file with merge conflicts
#[derive(Debug, Clone)]
pub struct FileConflict {
    /// Path to the conflicted file
    pub path: String,
    /// Base version content (from common ancestor)
    pub base_content: Option<Vec<u8>>,
    /// Our version content (HEAD)
    pub our_content: Option<Vec<u8>>,
    /// Their version content (branch being merged)
    pub their_content: Option<Vec<u8>>,
    /// Whether the file is binary
    pub is_binary: bool,
}

impl FileConflict {
    /// Create a new file conflict
    pub fn new(path: String) -> Self {
        FileConflict {
            path,
            base_content: None,
            our_content: None,
            their_content: None,
            is_binary: false,
        }
    }

    /// Generate content with conflict markers
    pub fn generate_conflict_markers(&self, our_branch: &str, their_branch: &str) -> Vec<u8> {
        if self.is_binary {
            return format!(
                "Binary file conflict in {}\n\
                 Use 'kitkat checkout --ours {}' or 'kitkat checkout --theirs {}'\n",
                self.path, self.path, self.path
            )
            .into_bytes();
        }

        let mut result = Vec::new();

        let our_text = self
            .our_content
            .as_ref()
            .map(|c| String::from_utf8_lossy(c).to_string())
            .unwrap_or_default();

        let their_text = self
            .their_content
            .as_ref()
            .map(|c| String::from_utf8_lossy(c).to_string())
            .unwrap_or_default();

        // Generate conflict markers
        result.extend_from_slice(format!("<<<<<<< {}\n", our_branch).as_bytes());
        result.extend_from_slice(our_text.as_bytes());
        if !our_text.ends_with('\n') {
            result.push(b'\n');
        }
        result.extend_from_slice(b"=======\n");
        result.extend_from_slice(their_text.as_bytes());
        if !their_text.ends_with('\n') {
            result.push(b'\n');
        }
        result.extend_from_slice(format!(">>>>>>> {}\n", their_branch).as_bytes());

        result
    }
}

impl fmt::Display for FileConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_binary {
            write!(f, "Binary conflict in {}", self.path)
        } else {
            write!(f, "Conflict in {}", self.path)
        }
    }
}

/// Conflict marker for tracking conflict state
#[derive(Debug, Clone)]
pub struct ConflictMarker {
    /// Start of conflict section
    pub start_line: usize,
    /// End of conflict section
    pub end_line: usize,
    /// Our version lines
    pub our_lines: Vec<String>,
    /// Their version lines
    pub their_lines: Vec<String>,
}

impl ConflictMarker {
    /// Create a new conflict marker
    pub fn new(start_line: usize) -> Self {
        ConflictMarker {
            start_line,
            end_line: 0,
            our_lines: Vec::new(),
            their_lines: Vec::new(),
        }
    }

    /// Check if content has unresolved conflicts
    pub fn has_conflicts(content: &str) -> bool {
        content.contains("<<<<<<<") && content.contains(">>>>>>>")
    }
}

/// Result of a file merge operation
#[derive(Debug, Clone)]
pub enum FileMergeResult {
    /// File merged successfully
    Success { content: Vec<u8> },
    /// File has conflicts
    Conflict { conflict: FileConflict },
    /// No changes needed
    Unchanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_markers() {
        let mut conflict = FileConflict::new("test.txt".to_string());
        conflict.our_content = Some(b"our version\n".to_vec());
        conflict.their_content = Some(b"their version\n".to_vec());

        let result = conflict.generate_conflict_markers("HEAD", "feature");
        let result_str = String::from_utf8(result).unwrap();

        assert!(result_str.contains("<<<<<<< HEAD"));
        assert!(result_str.contains("======="));
        assert!(result_str.contains(">>>>>>> feature"));
        assert!(result_str.contains("our version"));
        assert!(result_str.contains("their version"));
    }

    #[test]
    fn test_has_conflicts() {
        let content = "<<<<<<< HEAD\nour\n=======\ntheir\n>>>>>>> branch";
        assert!(ConflictMarker::has_conflicts(content));

        let clean = "no conflicts here";
        assert!(!ConflictMarker::has_conflicts(clean));
    }
}
