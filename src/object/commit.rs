use crate::config::Config;
use crate::models::Commit;
use crate::utils::{compress_data, compute_hash, decompress_data};
use chrono::{DateTime, Local, TimeZone};
use std::fs;
use std::io;
use std::path::Path;

/// Create a commit object
pub fn create_commit(
    tree_hash: &str,
    parent_hashes: &[String],
    message: &str,
) -> io::Result<String> {
    let config = Config::read()?;
    let author = config.get_user_string();
    let timestamp = Local::now().timestamp();
    let timezone = get_timezone_offset();

    // Build commit content
    let mut content = String::new();
    content.push_str(&format!("tree {}\n", tree_hash));

    for parent in parent_hashes {
        content.push_str(&format!("parent {}\n", parent));
    }

    content.push_str(&format!("author {} {} {}\n", author, timestamp, timezone));
    content.push_str(&format!(
        "committer {} {} {}\n",
        author, timestamp, timezone
    ));
    content.push_str("\n");
    content.push_str(message);
    content.push_str("\n");

    // Create commit object with header
    let header = format!("commit {}\0", content.len());
    let mut full_content = header.as_bytes().to_vec();
    full_content.extend_from_slice(content.as_bytes());

    // Compute hash
    let hash = compute_hash(&full_content);

    // Store object
    store_object(&hash, &full_content)?;

    Ok(hash)
}

/// Read a commit object
pub fn read_commit(hash: &str) -> io::Result<Commit> {
    let content = read_object_content(hash)?;
    parse_commit(&content)
}

/// Parse commit content
fn parse_commit(data: &[u8]) -> io::Result<Commit> {
    // Find the null byte after the header
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid commit object"))?;

    // Verify this is a commit object
    let header = String::from_utf8_lossy(&data[0..null_pos]);
    if !header.starts_with("commit ") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a commit object",
        ));
    }

    let content = String::from_utf8_lossy(&data[null_pos + 1..]);
    let lines: Vec<&str> = content.lines().collect();

    let mut tree = String::new();
    let mut parents = Vec::new();
    let mut author = String::new();
    let mut author_time = 0i64;
    let mut committer = String::new();
    let mut committer_time = 0i64;
    let mut message_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            message_start = i + 1;
            break;
        }

        if let Some(tree_hash) = line.strip_prefix("tree ") {
            tree = tree_hash.to_string();
        } else if let Some(parent_hash) = line.strip_prefix("parent ") {
            parents.push(parent_hash.to_string());
        } else if let Some(author_line) = line.strip_prefix("author ") {
            let parts: Vec<&str> = author_line.rsplitn(3, ' ').collect();
            if parts.len() >= 2 {
                author_time = parts[1].parse().unwrap_or(0);
                author = parts[2].to_string();
            }
        } else if let Some(committer_line) = line.strip_prefix("committer ") {
            let parts: Vec<&str> = committer_line.rsplitn(3, ' ').collect();
            if parts.len() >= 2 {
                committer_time = parts[1].parse().unwrap_or(0);
                committer = parts[2].to_string();
            }
        }
    }

    let message = lines[message_start..].join("\n");

    Ok(Commit {
        tree,
        parents,
        author,
        author_time,
        committer,
        committer_time,
        message,
    })
}

/// Store an object in the .kitkat/objects directory
fn store_object(hash: &str, content: &[u8]) -> io::Result<()> {
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];

    let dir_path = Path::new(".kitkat/objects").join(dir_name);
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(file_name);

    // Don't overwrite if object already exists
    if file_path.exists() {
        return Ok(());
    }

    // Compress and write
    let compressed = compress_data(content);
    fs::write(file_path, compressed)?;

    Ok(())
}

/// Read an object's content
fn read_object_content(hash: &str) -> io::Result<Vec<u8>> {
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];

    let file_path = Path::new(".kitkat/objects")
        .join(dir_name)
        .join(file_name);

    if !file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Object {} not found", hash),
        ));
    }

    let compressed = fs::read(file_path)?;
    let decompressed = decompress_data(&compressed);

    Ok(decompressed)
}

/// Get timezone offset string (e.g., "+0530" or "-0800")
fn get_timezone_offset() -> String {
    let now = Local::now();
    let offset = now.offset().local_minus_utc();
    let hours = offset / 3600;
    let minutes = (offset.abs() % 3600) / 60;
    format!("{:+03}{:02}", hours, minutes)
}

/// Display a commit in a human-readable format
pub fn show_commit(hash: &str) -> io::Result<()> {
    let commit = read_commit(hash)?;

    println!("commit {}", hash);

    if !commit.parents.is_empty() {
        if commit.parents.len() == 1 {
            println!("Parent: {}", commit.parents[0]);
        } else {
            println!("Merge:");
            for parent in &commit.parents {
                println!("  {}", parent);
            }
        }
    }

    println!("Author: {}", commit.author);

    // Format timestamp
    let dt = Local.timestamp_opt(commit.author_time, 0).unwrap();
    println!("Date:   {}", dt.format("%a %b %d %H:%M:%S %Y %z"));

    println!();
    for line in commit.message.lines() {
        println!("    {}", line);
    }

    Ok(())
}

/// Get commit parents (for traversal)
pub fn get_commit_parents(hash: &str) -> io::Result<Vec<String>> {
    let commit = read_commit(hash)?;
    Ok(commit.parents)
}

/// Get commit tree hash
pub fn get_commit_tree(hash: &str) -> io::Result<String> {
    let commit = read_commit(hash)?;
    Ok(commit.tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_offset() {
        let tz = get_timezone_offset();
        // Should be in format +HHMM or -HHMM
        assert!(tz.len() >= 5);
        assert!(tz.starts_with('+') || tz.starts_with('-'));
    }

    #[test]
    fn test_parse_commit_format() {
        let content = b"commit 200\0tree abc123\nparent def456\nauthor John Doe <john@example.com> 1234567890 +0000\ncommitter John Doe <john@example.com> 1234567890 +0000\n\nInitial commit\n";
        let commit = parse_commit(content).unwrap();
        assert_eq!(commit.tree, "abc123");
        assert_eq!(commit.parents.len(), 1);
        assert_eq!(commit.parents[0], "def456");
        assert!(commit.message.contains("Initial commit"));
    }
}
