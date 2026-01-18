/// Diff algorithms implementation
///
/// Provides Myers diff algorithm for computing line-by-line diffs

use crate::diff::types::{DiffHunk, DiffLine, FileDiff};

/// Supported diff algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffAlgorithm {
    /// Myers algorithm (default, efficient for most cases)
    Myers,
}

/// Compute diff between two texts using specified algorithm
pub fn compute_diff(old_text: &str, new_text: &str, algorithm: DiffAlgorithm) -> FileDiff {
    match algorithm {
        DiffAlgorithm::Myers => myers_diff(old_text, new_text),
    }
}

/// Myers diff algorithm implementation
///
/// This is a simplified implementation of the Myers algorithm that works line-by-line.
/// For production use, consider using the `similar` crate which has an optimized implementation.
fn myers_diff(old_text: &str, new_text: &str) -> FileDiff {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    // Compute the edit script using Myers algorithm
    let edits = compute_edit_script(&old_lines, &new_lines);

    // Convert edit script to hunks
    let hunks = edits_to_hunks(&edits, &old_lines, &new_lines);

    FileDiff {
        old_path: String::new(),
        new_path: String::new(),
        is_binary: false,
        hunks,
    }
}

/// Edit operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edit {
    Keep,
    Delete,
    Insert,
}

/// Compute the shortest edit script using Myers algorithm
fn compute_edit_script(old_lines: &[&str], new_lines: &[&str]) -> Vec<Edit> {
    let n = old_lines.len();
    let m = new_lines.len();

    // Special cases
    if n == 0 {
        return vec![Edit::Insert; m];
    }
    if m == 0 {
        return vec![Edit::Delete; n];
    }

    // Dynamic programming approach (simplified Myers)
    // dp[i][j] = minimum edit distance between old[0..i] and new[0..j]
    let mut dp = vec![vec![0; m + 1]; n + 1];
    let mut ops = vec![vec![Edit::Keep; m + 1]; n + 1];

    // Initialize base cases
    for i in 1..=n {
        dp[i][0] = i;
        ops[i][0] = Edit::Delete;
    }
    for j in 1..=m {
        dp[0][j] = j;
        ops[0][j] = Edit::Insert;
    }

    // Fill DP table
    for i in 1..=n {
        for j in 1..=m {
            if old_lines[i - 1] == new_lines[j - 1] {
                // Lines match, keep them
                dp[i][j] = dp[i - 1][j - 1];
                ops[i][j] = Edit::Keep;
            } else {
                // Lines differ, choose minimum cost operation
                let delete_cost = dp[i - 1][j] + 1;
                let insert_cost = dp[i][j - 1] + 1;

                if delete_cost < insert_cost {
                    dp[i][j] = delete_cost;
                    ops[i][j] = Edit::Delete;
                } else {
                    dp[i][j] = insert_cost;
                    ops[i][j] = Edit::Insert;
                }
            }
        }
    }

    // Backtrack to build edit script
    let mut edits = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        match ops[i][j] {
            Edit::Keep => {
                edits.push(Edit::Keep);
                i -= 1;
                j -= 1;
            }
            Edit::Delete => {
                edits.push(Edit::Delete);
                i -= 1;
            }
            Edit::Insert => {
                edits.push(Edit::Insert);
                j -= 1;
            }
        }
    }

    edits.reverse();
    edits
}

/// Convert edit script to diff hunks with context
fn edits_to_hunks(edits: &[Edit], old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffHunk> {
    const CONTEXT_LINES: usize = 3;

    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;

    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut context_buffer: Vec<DiffLine> = Vec::new();

    for &edit in edits {
        match edit {
            Edit::Keep => {
                let line = DiffLine::context(
                    old_idx + 1,
                    new_idx + 1,
                    old_lines[old_idx].to_string(),
                );

                if current_hunk.is_some() {
                    // We're in a hunk, add as context
                    context_buffer.push(line);

                    // If we have enough trailing context, close the hunk
                    if context_buffer.len() > CONTEXT_LINES * 2 {
                        // Add the first CONTEXT_LINES to the current hunk
                        if let Some(ref mut hunk) = current_hunk {
                            for _ in 0..CONTEXT_LINES {
                                if let Some(ctx) = context_buffer.first().cloned() {
                                    hunk.add_line(ctx);
                                    context_buffer.remove(0);
                                }
                            }
                        }

                        // Close the hunk
                        if let Some(hunk) = current_hunk.take() {
                            hunks.push(hunk);
                        }

                        // Keep remaining context for next hunk
                        context_buffer.drain(0..context_buffer.len().saturating_sub(CONTEXT_LINES));
                    }
                } else {
                    // Not in a hunk, buffer for leading context
                    context_buffer.push(line);
                    if context_buffer.len() > CONTEXT_LINES {
                        context_buffer.remove(0);
                    }
                }

                old_idx += 1;
                new_idx += 1;
            }
            Edit::Delete | Edit::Insert => {
                // Start a new hunk if needed
                if current_hunk.is_none() {
                    let old_start = old_idx.saturating_sub(context_buffer.len()) + 1;
                    let new_start = new_idx.saturating_sub(context_buffer.len()) + 1;
                    let mut hunk = DiffHunk::new(old_start, new_start);

                    // Add leading context
                    for ctx in context_buffer.drain(..) {
                        hunk.add_line(ctx);
                    }

                    current_hunk = Some(hunk);
                } else {
                    // Add buffered context to current hunk
                    if let Some(ref mut hunk) = current_hunk {
                        for ctx in context_buffer.drain(..) {
                            hunk.add_line(ctx);
                        }
                    }
                }

                // Add the actual change
                if let Some(ref mut hunk) = current_hunk {
                    match edit {
                        Edit::Delete => {
                            let line = DiffLine::deletion(old_idx + 1, old_lines[old_idx].to_string());
                            hunk.add_line(line);
                            old_idx += 1;
                        }
                        Edit::Insert => {
                            let line = DiffLine::addition(new_idx + 1, new_lines[new_idx].to_string());
                            hunk.add_line(line);
                            new_idx += 1;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    // Close any remaining hunk
    if let Some(mut hunk) = current_hunk {
        // Add remaining context
        for ctx in context_buffer {
            hunk.add_line(ctx);
        }
        hunks.push(hunk);
    }

    hunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myers_diff_simple() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\n";

        let diff = myers_diff(old, new);
        assert!(!diff.hunks.is_empty());
    }

    #[test]
    fn test_myers_diff_additions() {
        let old = "line 1\nline 2\n";
        let new = "line 1\nline 2\nline 3\nline 4\n";

        let diff = myers_diff(old, new);
        assert_eq!(diff.additions(), 2);
        assert_eq!(diff.deletions(), 0);
    }

    #[test]
    fn test_myers_diff_deletions() {
        let old = "line 1\nline 2\nline 3\nline 4\n";
        let new = "line 1\nline 4\n";

        let diff = myers_diff(old, new);
        assert_eq!(diff.deletions(), 2);
    }

    #[test]
    fn test_compute_edit_script() {
        let old_lines = vec!["a", "b", "c"];
        let new_lines = vec!["a", "x", "c"];

        let edits = compute_edit_script(&old_lines, &new_lines);

        // Should be: Keep(a), Delete(b), Insert(x), Keep(c)
        assert!(edits.contains(&Edit::Keep));
        assert!(edits.contains(&Edit::Delete));
        assert!(edits.contains(&Edit::Insert));
    }
}
