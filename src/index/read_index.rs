use std::fs;
use std::path::Path;

use crate::models::IndexEntry;

/// Read the index and return a list of entries
pub fn read_index() -> Vec<IndexEntry> {
    if !Path::new(".kitkat/index").exists() {
        return vec![];
    }
    let data = fs::read_to_string(".kitkat/index").unwrap();
    data.lines()
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            IndexEntry {
                hash: parts[0].to_string(),
                path: parts[1].to_string(),
                _mode: "100644".to_string(),
            }
        })
        .collect()
}
