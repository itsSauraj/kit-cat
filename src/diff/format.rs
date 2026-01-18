/// Unified diff format output

use crate::diff::types::FileDiff;

/// Options for formatting unified diffs
#[derive(Debug, Clone)]
pub struct UnifiedDiffOptions {
    /// Number of context lines to show around changes
    pub context_lines: usize,
    /// Use color in output
    pub use_color: bool,
    /// Show line numbers
    pub show_line_numbers: bool,
}

impl Default for UnifiedDiffOptions {
    fn default() -> Self {
        UnifiedDiffOptions {
            context_lines: 3,
            use_color: true,
            show_line_numbers: false,
        }
    }
}

/// ANSI color codes
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Format a file diff as unified diff output
pub fn format_unified_diff(diff: &FileDiff, options: &UnifiedDiffOptions) -> String {
    let mut output = String::new();

    // File headers
    if options.use_color {
        output.push_str(&format!(
            "{}{}--- {}{}",
            colors::BOLD,
            colors::RED,
            diff.old_path,
            colors::RESET
        ));
        output.push('\n');
        output.push_str(&format!(
            "{}{}+++ {}{}",
            colors::BOLD,
            colors::GREEN,
            diff.new_path,
            colors::RESET
        ));
        output.push('\n');
    } else {
        output.push_str(&format!("--- {}\n", diff.old_path));
        output.push_str(&format!("+++ {}\n", diff.new_path));
    }

    // Binary file check
    if diff.is_binary {
        output.push_str("Binary files differ\n");
        return output;
    }

    // Format each hunk
    for hunk in &diff.hunks {
        // Hunk header
        if options.use_color {
            output.push_str(&format!(
                "{}{}{}{}",
                colors::CYAN,
                hunk.header(),
                colors::RESET,
                "\n"
            ));
        } else {
            output.push_str(&format!("{}\n", hunk.header()));
        }

        // Hunk lines
        for line in &hunk.lines {
            let line_str = if options.use_color {
                match line.prefix() {
                    '+' => format!(
                        "{}{}{}{}",
                        colors::GREEN,
                        line.prefix(),
                        line.content,
                        colors::RESET
                    ),
                    '-' => format!(
                        "{}{}{}{}",
                        colors::RED,
                        line.prefix(),
                        line.content,
                        colors::RESET
                    ),
                    _ => format!("{}{}", line.prefix(), line.content),
                }
            } else {
                format!("{}{}", line.prefix(), line.content)
            };

            output.push_str(&line_str);
            output.push('\n');
        }
    }

    output
}

/// Format diff statistics (e.g., "3 insertions(+), 2 deletions(-)")
pub fn format_diff_stats(diff: &FileDiff, use_color: bool) -> String {
    let additions = diff.additions();
    let deletions = diff.deletions();

    if use_color {
        format!(
            "{}{} insertion{}(+){}, {}{} deletion{}(-){}",
            colors::GREEN,
            additions,
            if additions == 1 { "" } else { "s" },
            colors::RESET,
            colors::RED,
            deletions,
            if deletions == 1 { "" } else { "s" },
            colors::RESET
        )
    } else {
        format!(
            "{} insertion{}(+), {} deletion{}(-)",
            additions,
            if additions == 1 { "" } else { "s" },
            deletions,
            if deletions == 1 { "" } else { "s" }
        )
    }
}

/// Format a short diff summary (e.g., "README.md | 5 +++--")
pub fn format_diff_summary(diff: &FileDiff, use_color: bool) -> String {
    if diff.is_binary {
        return format!("{} | Binary file", diff.new_path);
    }

    let additions = diff.additions();
    let deletions = diff.deletions();
    let changes = additions + deletions;

    let bar = if use_color {
        let plus = format!("{}{}{}", colors::GREEN, "+".repeat(additions), colors::RESET);
        let minus = format!("{}{}{}", colors::RED, "-".repeat(deletions), colors::RESET);
        format!("{}{}", plus, minus)
    } else {
        format!("{}{}", "+".repeat(additions), "-".repeat(deletions))
    };

    format!("{} | {} {}", diff.new_path, changes, bar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{DiffHunk, DiffLine};

    #[test]
    fn test_format_diff_stats() {
        let mut diff = FileDiff::new("old.txt".to_string(), "new.txt".to_string());
        let mut hunk = DiffHunk::new(1, 1);

        hunk.add_line(DiffLine::addition(1, "line 1".to_string()));
        hunk.add_line(DiffLine::addition(2, "line 2".to_string()));
        hunk.add_line(DiffLine::deletion(1, "old line".to_string()));

        diff.hunks.push(hunk);

        let stats = format_diff_stats(&diff, false);
        assert!(stats.contains("2 insertions(+)"));
        assert!(stats.contains("1 deletion(-)"));
    }

    #[test]
    fn test_format_diff_summary() {
        let mut diff = FileDiff::new("old.txt".to_string(), "new.txt".to_string());
        let mut hunk = DiffHunk::new(1, 1);

        hunk.add_line(DiffLine::addition(1, "line 1".to_string()));
        hunk.add_line(DiffLine::deletion(1, "old line".to_string()));

        diff.hunks.push(hunk);

        let summary = format_diff_summary(&diff, false);
        assert!(summary.contains("new.txt"));
        assert!(summary.contains("2"));
        assert!(summary.contains("+"));
        assert!(summary.contains("-"));
    }

    #[test]
    fn test_format_unified_diff() {
        let mut diff = FileDiff::new("a/file.txt".to_string(), "b/file.txt".to_string());
        let mut hunk = DiffHunk::new(1, 1);

        hunk.add_line(DiffLine::context(1, 1, "line 1".to_string()));
        hunk.add_line(DiffLine::deletion(2, "line 2".to_string()));
        hunk.add_line(DiffLine::addition(2, "line 2 modified".to_string()));

        diff.hunks.push(hunk);

        let options = UnifiedDiffOptions {
            use_color: false,
            ..Default::default()
        };

        let output = format_unified_diff(&diff, &options);

        assert!(output.contains("--- a/file.txt"));
        assert!(output.contains("+++ b/file.txt"));
        assert!(output.contains("@@"));
        assert!(output.contains("-line 2"));
        assert!(output.contains("+line 2 modified"));
    }
}
