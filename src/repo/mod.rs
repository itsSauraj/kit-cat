use std::fs;

/// Initialize the repository structure
pub fn init_repo() -> std::io::Result<()> {
    fs::create_dir_all(".kitcat/objects")?;
    fs::create_dir_all(".kitcat/refs/heads")?;
    fs::write(".kitcat/HEAD", "ref: refs/heads/master\n")?;
    Ok(())
}

/// Read the current HEAD reference or commit hash
pub fn read_head() -> String {
    let head = fs::read_to_string(".kitcat/HEAD").expect("Failed to read HEAD");
    head.trim().to_string()
}

/// Update the HEAD reference or commit hash
pub fn write_head(value: &str) {
    fs::write(".kitcat/HEAD", value).expect("Failed to write HEAD");
}

// Add more repo functions like checkout, branch handling here
