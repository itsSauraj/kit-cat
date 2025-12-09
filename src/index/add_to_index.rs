/// Add a file to the index
pub fn add_to_index(file: String) {
    let hash = crate::object::hash_object(file.clone());
    let cwd = std::env::current_dir().unwrap();
    let file_path = std::path::Path::new(&file);
    let abs_path = cwd.join(file_path);
    // TODO: Implement actual index writing
    println!("Added {} to index with hash {}", abs_path.display(), hash);
}
