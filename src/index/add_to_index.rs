use crate::index::write_index::add_file_to_index;

/// Add a file to the index
pub fn add_to_index(file: String) {
    // Hash the file content and create the blob object
    let hash = crate::object::hash_object(file.clone());

    // Add the file to the index with its hash
    match add_file_to_index(&file, &hash) {
        Ok(_) => {
            println!("Added {} with hash {}", file, hash);
        }
        Err(e) => {
            eprintln!("Error adding file to index: {}", e);
            std::process::exit(1);
        }
    }
}
