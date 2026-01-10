use crate::models::IndexEntry;

/// Initialize a new repository
pub fn init() {
    if crate::utils::is_repo_init() {
        eprintln!("Repository already initialized.");
        return;
    }

    match crate::repo::init_repo() {
        Ok(_) => eprintln!("Initialized empty kitkat repository."),
        Err(e) => eprintln!("Failed to initialize repository: {}", e),
    }
}

/// Compute object ID and optionally create a blob from a file
pub fn hash_file(file: String) -> String {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return "".to_string();
    }
    crate::object::hash_object(file)
}

/// Read the content of an object
pub fn read_file(hash: String, pretty: bool) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }
    crate::object::read_object(hash, pretty)
}

/// Add file contents to the index
pub fn add_to_index(file: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }
    crate::index::add_to_index(file)
}

/// Read the index
pub fn read_index() -> Vec<IndexEntry> {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return vec![];
    }
    crate::index::read_index()
}

/// Write a value to HEAD
pub fn write_head(value: &str) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }
    crate::repo::write_head(value)
}

/// Read the current HEAD
pub fn read_head() -> String {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return "".to_string();
    }
    crate::repo::read_head()
}

/// Write a tree from the current index
pub fn write_tree() -> String {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return "".to_string();
    }

    let entries = crate::index::read_index();
    if entries.is_empty() {
        eprintln!("Nothing to commit (no files in index).");
        return "".to_string();
    }

    match crate::object::write_tree_from_index(&entries) {
        Ok(hash) => {
            println!("{}", hash);
            hash
        }
        Err(e) => {
            eprintln!("Failed to write tree: {}", e);
            "".to_string()
        }
    }
}

/// List the contents of a tree
pub fn list_tree(hash: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }

    match crate::object::list_tree(&hash, "") {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to list tree: {}", e),
    }
}

/// Create a commit
pub fn commit(message: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }

    // Get current index
    let entries = crate::index::read_index();
    if entries.is_empty() {
        eprintln!("Nothing to commit (no files in index).");
        eprintln!("Use 'kitkat add <file>' to add files to the index.");
        return;
    }

    // Write tree from index
    let tree_hash = match crate::object::write_tree_from_index(&entries) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Failed to write tree: {}", e);
            return;
        }
    };

    // Get parent commit (current HEAD)
    let head_content = crate::repo::read_head();
    let parents = if head_content.starts_with("ref:") {
        // HEAD points to a branch
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);
        if std::path::Path::new(&branch_path).exists() {
            vec![std::fs::read_to_string(&branch_path)
                .unwrap_or_default()
                .trim()
                .to_string()]
        } else {
            vec![] // First commit on this branch
        }
    } else if head_content.len() == 40 {
        // Detached HEAD
        vec![head_content]
    } else {
        vec![] // No parent (first commit)
    };

    // Create commit
    let commit_hash = match crate::object::create_commit(&tree_hash, &parents, &message) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Failed to create commit: {}", e);
            return;
        }
    };

    // Update HEAD/branch reference
    if head_content.starts_with("ref:") {
        let branch_name = head_content.trim_start_matches("ref: ").trim();
        let branch_path = format!(".kitkat/{}", branch_name);
        if let Err(e) = std::fs::write(&branch_path, &commit_hash) {
            eprintln!("Failed to update branch: {}", e);
            return;
        }
    } else {
        // Update HEAD directly (detached HEAD)
        crate::repo::write_head(&commit_hash);
    }

    println!("[{}] {}", &commit_hash[0..7], message.lines().next().unwrap_or(""));
}

/// Show a commit
pub fn show_commit_cmd(hash: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }

    match crate::object::show_commit(&hash) {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to show commit: {}", e),
    }
}

/// Set a config value
pub fn set_config_cmd(key: String, value: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }

    match crate::config::set_config(&key, &value) {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to set config: {}", e),
    }
}

/// Get a config value
pub fn get_config_cmd(key: String) {
    if !crate::utils::is_repo_init() {
        eprintln!("Repository not initialized.");
        return;
    }

    match crate::config::get_config(&key) {
        Ok(value) => println!("{}", value),
        Err(e) => eprintln!("Failed to get config: {}", e),
    }
}
