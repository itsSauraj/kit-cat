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
