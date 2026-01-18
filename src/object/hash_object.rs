use crate::utils::{compress_data, compute_hash};
use std::fs;

/// Create a blob object from a file and return its hash
pub fn hash_object(file: String) -> String {
    let data = std::fs::read(&file).expect("Failed to read file");
    let header = format!("blob {}\0", data.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(&data);

    let hash = compute_hash(&store);
    let dir = format!(".kitcat/objects/{}", &hash[..2]);
    let file_path = format!("{}/{}", dir, &hash[2..]);
    fs::create_dir_all(&dir).unwrap();
    fs::write(&file_path, compress_data(&store)).unwrap();

    hash
}
