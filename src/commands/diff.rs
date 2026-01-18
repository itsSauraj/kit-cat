/// Diff command implementation
///
/// Provides functionality to compare:
/// - Working directory vs index (unstaged changes)
/// - Index vs HEAD (staged changes)
/// - Commit vs commit
/// - Working directory vs commit

use crate::diff::{diff_texts, is_binary};
use crate::diff::format::{format_diff_stats, format_unified_diff, UnifiedDiffOptions};
use crate::index::read_index_binary;
use crate::object::{get_commit_tree, read_tree};
use crate::repo::read_head;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Diff mode
#[derive(Debug, Clone, Copy)]
pub enum DiffMode {
    /// Compare working directory vs index (default)
    WorkingVsIndex,
    /// Compare index vs HEAD (--cached/--staged)
    IndexVsHead,
    /// Compare working directory vs specific commit
    WorkingVsCommit,
    /// Compare two commits
    CommitVsCommit,
}

/// Options for diff command
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// Diff mode
    pub mode: DiffMode,
    /// First commit hash (if comparing commits)
    pub commit1: Option<String>,
    /// Second commit hash (if comparing commits)
    pub commit2: Option<String>,
    /// Specific paths to diff (empty = all files)
    pub paths: Vec<String>,
    /// Use color output
    pub use_color: bool,
    /// Show statistics
    pub show_stats: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        DiffOptions {
            mode: DiffMode::WorkingVsIndex,
            commit1: None,
            commit2: None,
            paths: vec![],
            use_color: true,
            show_stats: false,
        }
    }
}

/// Main diff command
pub fn diff(options: DiffOptions) -> io::Result<()> {
    match options.mode {
        DiffMode::WorkingVsIndex => diff_working_vs_index(&options),
        DiffMode::IndexVsHead => diff_index_vs_head(&options),
        DiffMode::WorkingVsCommit => {
            let commit = options.commit1.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Commit hash required")
            })?;
            diff_working_vs_commit(commit, &options)
        }
        DiffMode::CommitVsCommit => {
            let commit1 = options.commit1.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "First commit hash required")
            })?;
            let commit2 = options.commit2.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Second commit hash required")
            })?;
            diff_commit_vs_commit(commit1, commit2, &options)
        }
    }
}

/// Compare working directory vs index
fn diff_working_vs_index(options: &DiffOptions) -> io::Result<()> {
    let index_entries = read_index_binary()?;
    let unified_opts = UnifiedDiffOptions {
        use_color: options.use_color,
        ..Default::default()
    };

    let mut any_changes = false;

    for entry in &index_entries {
        // Skip if not in requested paths
        if !options.paths.is_empty() && !options.paths.contains(&entry.path) {
            continue;
        }

        // Check if file exists in working directory
        if !Path::new(&entry.path).exists() {
            println!("deleted file: {}", entry.path);
            any_changes = true;
            continue;
        }

        // Read both versions
        let working_content = fs::read(&entry.path)?;
        let index_hash = &entry.hash;

        // Read from object store
        let obj_dir = &index_hash[0..2];
        let obj_file = &index_hash[2..];
        let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

        let compressed = fs::read(&obj_path)?;
        let object_content = crate::utils::decompress(&compressed)?;

        // Find null byte to separate header from content
        let null_pos = object_content
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid object"))?;

        let index_content = &object_content[null_pos + 1..];

        // Compare contents
        if working_content != index_content {
            any_changes = true;

            // Check if binary
            if is_binary(&working_content) || is_binary(index_content) {
                println!("Binary files a/{} and b/{} differ", entry.path, entry.path);
                continue;
            }

            // Compute and display diff
            let old_text = String::from_utf8_lossy(index_content);
            let new_text = String::from_utf8_lossy(&working_content);

            let mut diff = diff_texts(&old_text, &new_text);
            diff.old_path = format!("a/{}", entry.path);
            diff.new_path = format!("b/{}", entry.path);

            let output = format_unified_diff(&diff, &unified_opts);
            print!("{}", output);

            if options.show_stats {
                println!("{}", format_diff_stats(&diff, options.use_color));
            }
        }
    }

    if !any_changes {
        if options.use_color {
            println!("\x1b[32mNo changes\x1b[0m");
        } else {
            println!("No changes");
        }
    }

    Ok(())
}

/// Compare index vs HEAD
fn diff_index_vs_head(options: &DiffOptions) -> io::Result<()> {
    let index_entries = read_index_binary()?;
    let head_files = get_head_files()?;
    let unified_opts = UnifiedDiffOptions {
        use_color: options.use_color,
        ..Default::default()
    };

    let mut any_changes = false;

    // Check for modified and new files in index
    for entry in &index_entries {
        if !options.paths.is_empty() && !options.paths.contains(&entry.path) {
            continue;
        }

        let index_hash = &entry.hash;

        match head_files.get(&entry.path) {
            None => {
                // New file in index
                println!("new file: {}", entry.path);
                any_changes = true;
            }
            Some(head_hash) if head_hash != index_hash => {
                // Modified file
                any_changes = true;

                // Read both versions
                let index_content = read_object_content(index_hash)?;
                let head_content = read_object_content(head_hash)?;

                // Check if binary
                if is_binary(&index_content) || is_binary(&head_content) {
                    println!("Binary files a/{} and b/{} differ", entry.path, entry.path);
                    continue;
                }

                // Compute and display diff
                let old_text = String::from_utf8_lossy(&head_content);
                let new_text = String::from_utf8_lossy(&index_content);

                let mut diff = diff_texts(&old_text, &new_text);
                diff.old_path = format!("a/{}", entry.path);
                diff.new_path = format!("b/{}", entry.path);

                let output = format_unified_diff(&diff, &unified_opts);
                print!("{}", output);

                if options.show_stats {
                    println!("{}", format_diff_stats(&diff, options.use_color));
                }
            }
            _ => {
                // Unchanged
            }
        }
    }

    // Check for deleted files
    for (path, _) in &head_files {
        if !options.paths.is_empty() && !options.paths.contains(path) {
            continue;
        }

        if !index_entries.iter().any(|e| &e.path == path) {
            println!("deleted file: {}", path);
            any_changes = true;
        }
    }

    if !any_changes {
        if options.use_color {
            println!("\x1b[32mNo changes\x1b[0m");
        } else {
            println!("No changes");
        }
    }

    Ok(())
}

/// Compare working directory vs specific commit
fn diff_working_vs_commit(commit_hash: &str, options: &DiffOptions) -> io::Result<()> {
    let commit_files = get_commit_files(commit_hash)?;
    let unified_opts = UnifiedDiffOptions {
        use_color: options.use_color,
        ..Default::default()
    };

    let mut any_changes = false;

    // Check modified files
    for (path, commit_hash_val) in &commit_files {
        if !options.paths.is_empty() && !options.paths.contains(path) {
            continue;
        }

        if !Path::new(path).exists() {
            println!("deleted file: {}", path);
            any_changes = true;
            continue;
        }

        let working_content = fs::read(path)?;
        let commit_content = read_object_content(commit_hash_val)?;

        if working_content[..] != commit_content[..] {
            any_changes = true;

            if is_binary(&working_content) || is_binary(&commit_content) {
                println!("Binary files a/{} and b/{} differ", path, path);
                continue;
            }

            let old_text = String::from_utf8_lossy(&commit_content);
            let new_text = String::from_utf8_lossy(&working_content);

            let mut diff = diff_texts(&old_text, &new_text);
            diff.old_path = format!("a/{}", path);
            diff.new_path = format!("b/{}", path);

            let output = format_unified_diff(&diff, &unified_opts);
            print!("{}", output);

            if options.show_stats {
                println!("{}", format_diff_stats(&diff, options.use_color));
            }
        }
    }

    if !any_changes {
        if options.use_color {
            println!("\x1b[32mNo changes\x1b[0m");
        } else {
            println!("No changes");
        }
    }

    Ok(())
}

/// Compare two commits
fn diff_commit_vs_commit(commit1: &str, commit2: &str, options: &DiffOptions) -> io::Result<()> {
    let files1 = get_commit_files(commit1)?;
    let files2 = get_commit_files(commit2)?;
    let unified_opts = UnifiedDiffOptions {
        use_color: options.use_color,
        ..Default::default()
    };

    let mut all_paths: std::collections::HashSet<String> = files1.keys().cloned().collect();
    all_paths.extend(files2.keys().cloned());

    let mut any_changes = false;

    for path in all_paths {
        if !options.paths.is_empty() && !options.paths.contains(&path) {
            continue;
        }

        let hash1 = files1.get(&path);
        let hash2 = files2.get(&path);

        match (hash1, hash2) {
            (None, Some(_)) => {
                println!("new file: {}", path);
                any_changes = true;
            }
            (Some(_), None) => {
                println!("deleted file: {}", path);
                any_changes = true;
            }
            (Some(h1), Some(h2)) if h1 != h2 => {
                any_changes = true;

                let content1 = read_object_content(h1)?;
                let content2 = read_object_content(h2)?;

                if is_binary(&content1) || is_binary(&content2) {
                    println!("Binary files a/{} and b/{} differ", path, path);
                    continue;
                }

                let text1 = String::from_utf8_lossy(&content1);
                let text2 = String::from_utf8_lossy(&content2);

                let mut diff = diff_texts(&text1, &text2);
                diff.old_path = format!("a/{}", path);
                diff.new_path = format!("b/{}", path);

                let output = format_unified_diff(&diff, &unified_opts);
                print!("{}", output);

                if options.show_stats {
                    println!("{}", format_diff_stats(&diff, options.use_color));
                }
            }
            _ => {}
        }
    }

    if !any_changes {
        if options.use_color {
            println!("\x1b[32mNo changes\x1b[0m");
        } else {
            println!("No changes");
        }
    }

    Ok(())
}

/// Get all files from HEAD commit
fn get_head_files() -> io::Result<HashMap<String, String>> {
    let head_content = read_head();

    let commit_hash = if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !Path::new(&branch_path).exists() {
            return Ok(HashMap::new());
        }

        fs::read_to_string(&branch_path)?.trim().to_string()
    } else if head_content.len() == 40 {
        head_content
    } else {
        return Ok(HashMap::new());
    };

    get_commit_files(&commit_hash)
}

/// Get all files from a specific commit
fn get_commit_files(commit_hash: &str) -> io::Result<HashMap<String, String>> {
    let tree_hash = get_commit_tree(commit_hash)?;
    let mut files = HashMap::new();
    collect_tree_files(&tree_hash, "", &mut files)?;
    Ok(files)
}

/// Recursively collect all files from a tree
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

/// Read object content from object store
fn read_object_content(hash: &str) -> io::Result<Vec<u8>> {
    let obj_dir = &hash[0..2];
    let obj_file = &hash[2..];
    let obj_path = format!(".kitkat/objects/{}/{}", obj_dir, obj_file);

    let compressed = fs::read(&obj_path)?;
    let content = crate::utils::decompress(&compressed)?;

    // Find null byte to separate header from content
    let null_pos = content
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid object"))?;

    Ok(content[null_pos + 1..].to_vec())
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
