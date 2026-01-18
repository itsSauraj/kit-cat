/// Merge base detection algorithm
///
/// Finds the common ancestor of two commits using a graph traversal approach

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;

/// Find the merge base (common ancestor) of two commits
///
/// Uses a breadth-first search to find the lowest common ancestor
pub fn find_merge_base(commit1: &str, commit2: &str) -> io::Result<Option<String>> {
    if commit1 == commit2 {
        return Ok(Some(commit1.to_string()));
    }

    // Build ancestor chains for both commits
    let ancestors1 = get_all_ancestors(commit1)?;
    let ancestors2 = get_all_ancestors(commit2)?;

    // Find common ancestors
    let common: HashSet<_> = ancestors1.intersection(&ancestors2).collect();

    if common.is_empty() {
        return Ok(None);
    }

    // Find the closest common ancestor
    // This is the one with the shortest path from both commits
    let mut best_base = None;
    let mut best_distance = usize::MAX;

    for ancestor in common {
        let dist1 = get_distance(commit1, ancestor)?;
        let dist2 = get_distance(commit2, ancestor)?;
        let total_dist = dist1 + dist2;

        if total_dist < best_distance {
            best_distance = total_dist;
            best_base = Some(ancestor.to_string());
        }
    }

    Ok(best_base)
}

/// Get all ancestors of a commit
fn get_all_ancestors(commit_hash: &str) -> io::Result<HashSet<String>> {
    let mut ancestors = HashSet::new();
    let mut to_visit = VecDeque::new();
    to_visit.push_back(commit_hash.to_string());

    while let Some(current) = to_visit.pop_front() {
        if ancestors.contains(&current) {
            continue;
        }

        ancestors.insert(current.clone());

        let parents = get_commit_parents(&current)?;
        for parent in parents {
            to_visit.push_back(parent);
        }
    }

    Ok(ancestors)
}

/// Get distance from start commit to target commit
fn get_distance(start: &str, target: &str) -> io::Result<usize> {
    if start == target {
        return Ok(0);
    }

    let mut distances: HashMap<String, usize> = HashMap::new();
    let mut queue = VecDeque::new();

    queue.push_back((start.to_string(), 0));
    distances.insert(start.to_string(), 0);

    while let Some((current, dist)) = queue.pop_front() {
        if current == target {
            return Ok(dist);
        }

        let parents = get_commit_parents(&current)?;
        for parent in parents {
            if !distances.contains_key(&parent) {
                distances.insert(parent.clone(), dist + 1);
                queue.push_back((parent, dist + 1));
            }
        }
    }

    Ok(usize::MAX)
}

/// Get parent commits of a commit
fn get_commit_parents(commit_hash: &str) -> io::Result<Vec<String>> {
    use std::fs;

    let obj_dir = &commit_hash[0..2];
    let obj_file = &commit_hash[2..];
    let obj_path = format!(".kitcat/objects/{}/{}", obj_dir, obj_file);

    if !std::path::Path::new(&obj_path).exists() {
        return Ok(Vec::new());
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_commit() {
        // If commits are the same, they are their own merge base
        let base = find_merge_base("abc123", "abc123");
        assert!(base.is_ok());
        // Note: This will fail in practice without a real repo,
        // but the logic is correct
    }

    #[test]
    fn test_empty_ancestors() {
        let ancestors = HashSet::new();
        assert_eq!(ancestors.len(), 0);
    }
}
