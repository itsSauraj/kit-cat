/// Merge command implementation
///
/// Combines branches using three-way merge algorithm

use crate::index::{read_index_binary, write_index_binary};
use crate::merge::{can_fast_forward, find_merge_base, get_commit_files, merge_trees};
use crate::models::IndexEntry;
use crate::object::{create_commit, get_commit_tree};
use crate::repo::read_head;
use std::fs;
use std::io;
use std::path::Path;

/// Merge options
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// Branch or commit to merge
    pub target: String,
    /// Abort an in-progress merge
    pub abort: bool,
    /// Continue after resolving conflicts
    pub r#continue: bool,
    /// No fast-forward
    pub no_ff: bool,
    /// Fast-forward only (fail if merge commit needed)
    pub ff_only: bool,
    /// Commit message for merge commit
    pub message: Option<String>,
}

impl Default for MergeOptions {
    fn default() -> Self {
        MergeOptions {
            target: String::new(),
            abort: false,
            r#continue: false,
            no_ff: false,
            ff_only: false,
            message: None,
        }
    }
}

/// Main merge command
pub fn merge(options: MergeOptions) -> io::Result<()> {
    if options.abort {
        return abort_merge();
    }

    if options.r#continue {
        return continue_merge(options.message.as_deref());
    }

    // Get current HEAD commit
    let head_content = read_head();
    let our_commit = resolve_head(&head_content)?;
    let our_branch = if head_content.starts_with("ref:") {
        head_content
            .trim_start_matches("ref: refs/heads/")
            .trim()
            .to_string()
    } else {
        "HEAD".to_string()
    };

    // Resolve target to commit hash
    let their_commit = resolve_ref(&options.target)?;

    // Check if already up to date
    if our_commit == their_commit {
        println!("Already up to date.");
        return Ok(());
    }

    // Check for fast-forward
    let can_ff = can_fast_forward(&our_commit, &their_commit)?;

    if can_ff && !options.no_ff {
        // Fast-forward merge
        println!("Fast-forwarding...");
        return fast_forward_merge(&head_content, &their_commit);
    }

    if options.ff_only && !can_ff {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot fast-forward - merge commit required",
        ));
    }

    // Find merge base
    let base_commit = find_merge_base(&our_commit, &their_commit)?;

    if base_commit.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No common ancestor found - refusing to merge unrelated histories",
        ));
    }

    let base_commit = base_commit.unwrap();

    println!("Merge base: {}", base_commit);
    println!("Merging {} into {}", options.target, our_branch);

    // Get file trees for three-way merge
    let base_files = get_commit_files(&base_commit)?;
    let our_files = get_commit_files(&our_commit)?;
    let their_files = get_commit_files(&their_commit)?;

    // Perform three-way merge
    let merge_result = merge_trees(&base_files, &our_files, &their_files)?;

    if merge_result.has_conflicts() {
        // Save merge state
        save_merge_state(&our_commit, &their_commit, &our_branch, &options.target)?;

        // Write conflicted files
        for conflict in &merge_result.conflicts {
            let content = conflict.generate_conflict_markers(&our_branch, &options.target);
            fs::write(&conflict.path, content)?;
            println!("CONFLICT in {}", conflict.path);
        }

        // Write successfully merged files
        for (path, content) in &merge_result.merged_files {
            fs::write(path, content)?;
        }

        println!("\nAutomatic merge failed; fix conflicts and then run 'kitkat merge --continue'");
        return Ok(());
    }

    // No conflicts - write merged files and create merge commit
    for (path, content) in &merge_result.merged_files {
        fs::write(path, content)?;
    }

    // Update index with merged files
    update_index_with_merged_files(&merge_result)?;

    // Create merge commit
    let message = options.message.unwrap_or_else(|| {
        format!("Merge {} into {}", options.target, our_branch)
    });

    let merge_commit = create_merge_commit(&message, &our_commit, &their_commit)?;

    // Update HEAD
    if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);
        fs::write(&branch_path, &merge_commit)?;
    } else {
        fs::write(".kitkat/HEAD", &merge_commit)?;
    }

    println!("Merge completed successfully");
    println!("Merge commit: {}", merge_commit);

    Ok(())
}

/// Fast-forward merge
fn fast_forward_merge(head_content: &str, target_commit: &str) -> io::Result<()> {
    if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);
        fs::write(&branch_path, target_commit)?;
    } else {
        fs::write(".kitkat/HEAD", target_commit)?;
    }

    // Update working directory to match target
    let tree_hash = get_commit_tree(target_commit)?;
    checkout_tree(&tree_hash)?;

    println!("Updating {}..{}", &target_commit[..7], &target_commit[..7]);
    println!("Fast-forward");

    Ok(())
}

/// Create a merge commit with two parents
fn create_merge_commit(message: &str, parent1: &str, parent2: &str) -> io::Result<String> {
    // First create tree from current index
    let tree_hash = crate::commands::write_tree();

    // Create commit object with two parents
    let commit_hash = create_commit(
        &tree_hash,
        &[parent1.to_string(), parent2.to_string()],
        message,
    )?;

    Ok(commit_hash)
}

/// Resolve HEAD to commit hash
fn resolve_head(head_content: &str) -> io::Result<String> {
    if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !Path::new(&branch_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HEAD points to non-existent branch",
            ));
        }

        Ok(fs::read_to_string(&branch_path)?.trim().to_string())
    } else {
        Ok(head_content.trim().to_string())
    }
}

/// Resolve a ref (branch name or commit hash) to commit hash
fn resolve_ref(ref_name: &str) -> io::Result<String> {
    // Check if it's a branch name
    let branch_path = format!(".kitkat/refs/heads/{}", ref_name);
    if Path::new(&branch_path).exists() {
        return Ok(fs::read_to_string(&branch_path)?.trim().to_string());
    }

    // Assume it's a commit hash
    if ref_name.len() == 40 {
        Ok(ref_name.to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot resolve '{}' to a commit", ref_name),
        ))
    }
}

/// Save merge state for conflict resolution
fn save_merge_state(
    our_commit: &str,
    their_commit: &str,
    our_branch: &str,
    their_branch: &str,
) -> io::Result<()> {
    fs::create_dir_all(".kitkat/merge")?;
    fs::write(".kitkat/MERGE_HEAD", their_commit)?;
    fs::write(".kitkat/MERGE_MODE", "merge")?;
    fs::write(".kitkat/merge/our_commit", our_commit)?;
    fs::write(".kitkat/merge/their_commit", their_commit)?;
    fs::write(".kitkat/merge/our_branch", our_branch)?;
    fs::write(".kitkat/merge/their_branch", their_branch)?;
    Ok(())
}

/// Continue merge after conflict resolution
fn continue_merge(message: Option<&str>) -> io::Result<()> {
    if !Path::new(".kitkat/MERGE_HEAD").exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No merge in progress",
        ));
    }

    // Check if there are still conflicts
    let index = read_index_binary()?;
    for entry in &index {
        if Path::new(&entry.path).exists() {
            let content = fs::read_to_string(&entry.path)?;
            if crate::merge::types::ConflictMarker::has_conflicts(&content) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unresolved conflict in {}", entry.path),
                ));
            }
        }
    }

    // Read merge state
    let their_commit = fs::read_to_string(".kitkat/MERGE_HEAD")?.trim().to_string();
    let our_commit = fs::read_to_string(".kitkat/merge/our_commit")?.trim().to_string();
    let our_branch = fs::read_to_string(".kitkat/merge/our_branch")?;
    let their_branch = fs::read_to_string(".kitkat/merge/their_branch")?;

    // Create merge commit
    let default_msg = format!("Merge {} into {}", their_branch.trim(), our_branch.trim());
    let msg = message.unwrap_or(&default_msg);
    let merge_commit = create_merge_commit(msg, &our_commit, &their_commit)?;

    // Update HEAD
    let head_content = read_head();
    if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);
        fs::write(&branch_path, &merge_commit)?;
    } else {
        fs::write(".kitkat/HEAD", &merge_commit)?;
    }

    // Clean up merge state
    fs::remove_file(".kitkat/MERGE_HEAD")?;
    fs::remove_file(".kitkat/MERGE_MODE")?;
    fs::remove_dir_all(".kitkat/merge")?;

    println!("Merge completed successfully");
    println!("Merge commit: {}", merge_commit);

    Ok(())
}

/// Abort merge and restore original state
fn abort_merge() -> io::Result<()> {
    if !Path::new(".kitkat/MERGE_HEAD").exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No merge in progress",
        ));
    }

    let our_commit = fs::read_to_string(".kitkat/merge/our_commit")?.trim().to_string();

    // Restore to our original commit
    let tree_hash = get_commit_tree(&our_commit)?;
    checkout_tree(&tree_hash)?;

    // Clean up merge state
    fs::remove_file(".kitkat/MERGE_HEAD")?;
    fs::remove_file(".kitkat/MERGE_MODE")?;
    fs::remove_dir_all(".kitkat/merge")?;

    println!("Merge aborted");

    Ok(())
}

/// Checkout a tree to working directory
fn checkout_tree(tree_hash: &str) -> io::Result<()> {
    let files = get_tree_files(tree_hash)?;

    for (path, hash) in files {
        let content = read_object_content(&hash)?;

        // Create parent directories
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)?;
    }

    Ok(())
}

/// Get all files from a tree recursively
fn get_tree_files(tree_hash: &str) -> io::Result<std::collections::HashMap<String, String>> {
    let mut files = std::collections::HashMap::new();
    collect_tree_files(tree_hash, "", &mut files)?;
    Ok(files)
}

/// Recursively collect files from a tree
fn collect_tree_files(
    tree_hash: &str,
    prefix: &str,
    files: &mut std::collections::HashMap<String, String>,
) -> io::Result<()> {
    let tree_entries = crate::object::read_tree(tree_hash)?;

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

/// Update index with merged files
fn update_index_with_merged_files(
    merge_result: &crate::merge::three_way::MergeResult,
) -> io::Result<()> {
    let mut index = read_index_binary()?;

    for (path, content) in &merge_result.merged_files {
        // Hash the content
        let hash = crate::utils::compute_hash(content);

        // Update or add to index
        if let Some(entry) = index.iter_mut().find(|e| e.path == *path) {
            entry.hash = hash.clone();
        } else {
            let metadata = fs::metadata(path)?;
            index.push(IndexEntry::from_file(path.clone(), hash, &metadata));
        }
    }

    write_index_binary(&index)?;
    Ok(())
}
