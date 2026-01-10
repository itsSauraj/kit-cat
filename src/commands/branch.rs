use std::fs;
use std::io;
use std::path::Path;

/// List all branches, highlighting the current one
pub fn list_branches() -> io::Result<()> {
    let refs_dir = Path::new(".kitkat/refs/heads");

    if !refs_dir.exists() {
        println!("No branches yet.");
        return Ok(());
    }

    let current_branch = get_current_branch()?;

    let mut branches = Vec::new();
    for entry in fs::read_dir(refs_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                branches.push(name.to_string());
            }
        }
    }

    branches.sort();

    for branch in branches {
        if Some(&branch) == current_branch.as_ref() {
            println!("* {}", branch); // Current branch
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}

/// Get the current branch name
pub fn get_current_branch() -> io::Result<Option<String>> {
    let head_content = crate::repo::read_head();

    if head_content.starts_with("ref: refs/heads/") {
        let branch = head_content
            .trim_start_matches("ref: refs/heads/")
            .trim()
            .to_string();
        Ok(Some(branch))
    } else {
        Ok(None) // Detached HEAD
    }
}

/// Create a new branch at the current HEAD
pub fn create_branch(name: &str) -> io::Result<()> {
    // Validate branch name
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Branch name cannot be empty",
        ));
    }

    if name.contains("..") || name.contains(" ") || name.starts_with('-') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid branch name",
        ));
    }

    let branch_path = Path::new(".kitkat/refs/heads").join(name);

    // Check if branch already exists
    if branch_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Branch '{}' already exists", name),
        ));
    }

    // Get current HEAD commit
    let head_content = crate::repo::read_head();
    let commit_hash = if head_content.starts_with("ref:") {
        // HEAD points to a branch - get that branch's commit
        let branch_ref = head_content.trim_start_matches("ref: ").trim();
        let branch_file = format!(".kitkat/{}", branch_ref);

        if !Path::new(&branch_file).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cannot create branch: no commits yet",
            ));
        }

        fs::read_to_string(&branch_file)?.trim().to_string()
    } else if head_content.len() == 40 {
        // Detached HEAD - use the commit hash directly
        head_content
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Cannot create branch: no commits yet",
        ));
    };

    // Validate commit hash
    if commit_hash.len() != 40 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid commit hash",
        ));
    }

    // Create the branch file
    fs::create_dir_all(".kitkat/refs/heads")?;
    fs::write(&branch_path, &commit_hash)?;

    println!("Created branch '{}'", name);
    Ok(())
}

/// Delete a branch
pub fn delete_branch(name: &str, force: bool) -> io::Result<()> {
    let branch_path = Path::new(".kitkat/refs/heads").join(name);

    // Check if branch exists
    if !branch_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Branch '{}' not found", name),
        ));
    }

    // Check if trying to delete current branch
    if let Ok(Some(current)) = get_current_branch() {
        if current == name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot delete the current branch '{}'", name),
            ));
        }
    }

    // TODO: In the future, add check for unmerged commits unless force=true
    if !force {
        // For now, we'll just delete it
        // Later: check if branch is merged into current branch
    }

    fs::remove_file(&branch_path)?;
    println!("Deleted branch '{}'", name);
    Ok(())
}

/// Switch to a different branch
pub fn switch_branch(name: &str) -> io::Result<()> {
    let branch_path = Path::new(".kitkat/refs/heads").join(name);

    // Check if branch exists
    if !branch_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Branch '{}' not found", name),
        ));
    }

    // Check if already on this branch
    if let Ok(Some(current)) = get_current_branch() {
        if current == name {
            println!("Already on '{}'", name);
            return Ok(());
        }
    }

    // TODO: Check for uncommitted changes in working tree/index
    // For now, we'll just switch

    // Get the commit hash for the branch
    let commit_hash = fs::read_to_string(&branch_path)?.trim().to_string();

    // TODO: Checkout the tree for this commit
    // For now, just update HEAD

    // Update HEAD to point to the new branch
    let new_head = format!("ref: refs/heads/{}", name);
    crate::repo::write_head(&new_head);

    println!("Switched to branch '{}'", name);
    Ok(())
}

/// Show the current branch
pub fn show_current_branch() -> io::Result<()> {
    match get_current_branch()? {
        Some(branch) => println!("{}", branch),
        None => {
            let head = crate::repo::read_head();
            println!("HEAD detached at {}", &head[0..7]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name_validation() {
        assert!(create_branch("").is_err());
        assert!(create_branch("feature..test").is_err());
        assert!(create_branch("feature test").is_err());
        assert!(create_branch("-feature").is_err());
    }
}
