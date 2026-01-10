use crate::object::{get_commit_parents, read_commit};
use chrono::{Local, TimeZone};
use std::collections::HashSet;
use std::io;

/// Display commit history
pub fn log(format: LogFormat, max_count: Option<usize>) -> io::Result<()> {
    // Get starting commit from HEAD
    let head_content = crate::repo::read_head();
    let start_commit = if head_content.starts_with("ref:") {
        // HEAD points to a branch
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !std::path::Path::new(&branch_path).exists() {
            println!("No commits yet.");
            return Ok(());
        }

        std::fs::read_to_string(&branch_path)?.trim().to_string()
    } else if head_content.len() == 40 {
        // Detached HEAD
        head_content
    } else {
        println!("No commits yet.");
        return Ok(());
    };

    // Walk the commit history
    walk_commits(&start_commit, format, max_count)?;

    Ok(())
}

/// Format options for log output
#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Full,
    Oneline,
    Short,
}

/// Walk the commit history and display commits
fn walk_commits(start: &str, format: LogFormat, max_count: Option<usize>) -> io::Result<()> {
    let mut visited = HashSet::new();
    let mut stack = vec![start.to_string()];
    let mut count = 0;

    while let Some(hash) = stack.pop() {
        // Skip if already visited
        if visited.contains(&hash) {
            continue;
        }
        visited.insert(hash.clone());

        // Check max count
        if let Some(max) = max_count {
            if count >= max {
                break;
            }
        }
        count += 1;

        // Read the commit
        let commit = match read_commit(&hash) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to read commit {}: {}", hash, e);
                continue;
            }
        };

        // Display the commit
        match format {
            LogFormat::Full => {
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
                let dt = Local.timestamp_opt(commit.author_time, 0).unwrap();
                println!("Date:   {}", dt.format("%a %b %d %H:%M:%S %Y %z"));
                println!();
                for line in commit.message.lines() {
                    println!("    {}", line);
                }
                println!();
            }
            LogFormat::Oneline => {
                let first_line = commit.message.lines().next().unwrap_or("");
                println!("{} {}", &hash[0..7], first_line);
            }
            LogFormat::Short => {
                println!("commit {}", hash);
                println!("Author: {}", commit.author);
                println!();
                let first_line = commit.message.lines().next().unwrap_or("");
                println!("    {}", first_line);
                println!();
            }
        }

        // Add parents to stack (in reverse order for correct traversal)
        for parent in commit.parents.iter().rev() {
            if !visited.contains(parent) {
                stack.push(parent.clone());
            }
        }
    }

    Ok(())
}

/// Log with filtering options
pub fn log_with_filter(
    format: LogFormat,
    max_count: Option<usize>,
    author_filter: Option<&str>,
    since: Option<i64>,
    until: Option<i64>,
) -> io::Result<()> {
    // Get starting commit from HEAD
    let head_content = crate::repo::read_head();
    let start_commit = if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);

        if !std::path::Path::new(&branch_path).exists() {
            println!("No commits yet.");
            return Ok(());
        }

        std::fs::read_to_string(&branch_path)?.trim().to_string()
    } else if head_content.len() == 40 {
        head_content
    } else {
        println!("No commits yet.");
        return Ok(());
    };

    // Walk commits with filtering
    let mut visited = HashSet::new();
    let mut stack = vec![start_commit];
    let mut count = 0;

    while let Some(hash) = stack.pop() {
        if visited.contains(&hash) {
            continue;
        }
        visited.insert(hash.clone());

        let commit = match read_commit(&hash) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Apply filters
        let mut should_display = true;

        if let Some(author) = author_filter {
            if !commit.author.contains(author) {
                should_display = false;
            }
        }

        if let Some(since_time) = since {
            if commit.author_time < since_time {
                should_display = false;
            }
        }

        if let Some(until_time) = until {
            if commit.author_time > until_time {
                should_display = false;
            }
        }

        if should_display {
            if let Some(max) = max_count {
                if count >= max {
                    break;
                }
            }
            count += 1;

            // Display the commit
            match format {
                LogFormat::Full => {
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
                    let dt = Local.timestamp_opt(commit.author_time, 0).unwrap();
                    println!("Date:   {}", dt.format("%a %b %d %H:%M:%S %Y %z"));
                    println!();
                    for line in commit.message.lines() {
                        println!("    {}", line);
                    }
                    println!();
                }
                LogFormat::Oneline => {
                    let first_line = commit.message.lines().next().unwrap_or("");
                    println!("{} {}", &hash[0..7], first_line);
                }
                LogFormat::Short => {
                    println!("commit {}", hash);
                    println!("Author: {}", commit.author);
                    println!();
                    let first_line = commit.message.lines().next().unwrap_or("");
                    println!("    {}", first_line);
                    println!();
                }
            }
        }

        // Add parents to stack
        for parent in commit.parents.iter().rev() {
            if !visited.contains(parent) {
                stack.push(parent.clone());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format() {
        // Simple format enum test
        let fmt = LogFormat::Oneline;
        assert!(matches!(fmt, LogFormat::Oneline));
    }
}
