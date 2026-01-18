/// Type definitions for diff operations

use std::fmt;

/// Type of a diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Line exists in both old and new (context)
    Context,
    /// Line was added in new version
    Addition,
    /// Line was deleted from old version
    Deletion,
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Type of this line (context, addition, deletion)
    pub line_type: DiffLineType,
    /// Line number in old file (None for additions)
    pub old_line_no: Option<usize>,
    /// Line number in new file (None for deletions)
    pub new_line_no: Option<usize>,
    /// Content of the line (without newline)
    pub content: String,
}

impl DiffLine {
    /// Create a context line
    pub fn context(old_no: usize, new_no: usize, content: String) -> Self {
        DiffLine {
            line_type: DiffLineType::Context,
            old_line_no: Some(old_no),
            new_line_no: Some(new_no),
            content,
        }
    }

    /// Create an addition line
    pub fn addition(new_no: usize, content: String) -> Self {
        DiffLine {
            line_type: DiffLineType::Addition,
            old_line_no: None,
            new_line_no: Some(new_no),
            content,
        }
    }

    /// Create a deletion line
    pub fn deletion(old_no: usize, content: String) -> Self {
        DiffLine {
            line_type: DiffLineType::Deletion,
            old_line_no: Some(old_no),
            new_line_no: None,
            content,
        }
    }

    /// Get the prefix for this line type (-, +, or space)
    pub fn prefix(&self) -> char {
        match self.line_type {
            DiffLineType::Context => ' ',
            DiffLineType::Addition => '+',
            DiffLineType::Deletion => '-',
        }
    }
}

impl fmt::Display for DiffLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix(), self.content)
    }
}

/// A contiguous block of changes in a diff (a "hunk")
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Starting line in old file (1-indexed)
    pub old_start: usize,
    /// Number of lines in old file
    pub old_count: usize,
    /// Starting line in new file (1-indexed)
    pub new_start: usize,
    /// Number of lines in new file
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Create a new empty hunk
    pub fn new(old_start: usize, new_start: usize) -> Self {
        DiffHunk {
            old_start,
            old_count: 0,
            new_start,
            new_count: 0,
            lines: Vec::new(),
        }
    }

    /// Add a line to this hunk and update counts
    pub fn add_line(&mut self, line: DiffLine) {
        match line.line_type {
            DiffLineType::Context => {
                self.old_count += 1;
                self.new_count += 1;
            }
            DiffLineType::Addition => {
                self.new_count += 1;
            }
            DiffLineType::Deletion => {
                self.old_count += 1;
            }
        }
        self.lines.push(line);
    }

    /// Get the hunk header (e.g., "@@ -1,3 +1,4 @@")
    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_count, self.new_start, self.new_count
        )
    }
}

impl fmt::Display for DiffHunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.header())?;
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

/// Complete diff of a file
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path to old file
    pub old_path: String,
    /// Path to new file
    pub new_path: String,
    /// Whether the file is binary
    pub is_binary: bool,
    /// Hunks of changes
    pub hunks: Vec<DiffHunk>,
}

impl FileDiff {
    /// Create a new empty file diff
    pub fn new(old_path: String, new_path: String) -> Self {
        FileDiff {
            old_path,
            new_path,
            is_binary: false,
            hunks: Vec::new(),
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        if self.is_binary {
            return true; // Binary files are always considered changed
        }
        !self.hunks.is_empty()
    }

    /// Count total additions
    pub fn additions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Addition)
            .count()
    }

    /// Count total deletions
    pub fn deletions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.line_type == DiffLineType::Deletion)
            .count()
    }
}

impl fmt::Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- {}", self.old_path)?;
        writeln!(f, "+++ {}", self.new_path)?;

        if self.is_binary {
            writeln!(f, "Binary files differ")?;
            return Ok(());
        }

        for hunk in &self.hunks {
            write!(f, "{}", hunk)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_line_prefix() {
        let context = DiffLine::context(1, 1, "line".to_string());
        assert_eq!(context.prefix(), ' ');

        let addition = DiffLine::addition(1, "line".to_string());
        assert_eq!(addition.prefix(), '+');

        let deletion = DiffLine::deletion(1, "line".to_string());
        assert_eq!(deletion.prefix(), '-');
    }

    #[test]
    fn test_hunk_header() {
        let hunk = DiffHunk {
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 4,
            lines: vec![],
        };
        assert_eq!(hunk.header(), "@@ -1,3 +1,4 @@");
    }

    #[test]
    fn test_file_diff_stats() {
        let mut diff = FileDiff::new("old.txt".to_string(), "new.txt".to_string());
        let mut hunk = DiffHunk::new(1, 1);

        hunk.add_line(DiffLine::context(1, 1, "line 1".to_string()));
        hunk.add_line(DiffLine::deletion(2, "line 2".to_string()));
        hunk.add_line(DiffLine::addition(2, "line 2 modified".to_string()));
        hunk.add_line(DiffLine::addition(3, "line 3".to_string()));

        diff.hunks.push(hunk);

        assert_eq!(diff.additions(), 2);
        assert_eq!(diff.deletions(), 1);
        assert!(diff.has_changes());
    }
}
