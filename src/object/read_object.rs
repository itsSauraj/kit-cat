// use crate::utils::decompress_data;
// use std::fs;

// /// Read an object by its hash
// pub fn read_object(hash: String, pretty: bool) {
//     let path = format!(".kitcat/objects/{}/{}", &hash[..2], &hash[2..]);
//     let compressed = fs::read(&path).expect("Object not found");
//     let decompressed = decompress_data(&compressed);

//     if let Some(pos) = decompressed.iter().position(|&b| b == 0) {
//         let _header = &decompressed[..pos];
//         let data = &decompressed[pos + 1..];
//         if pretty {
//             print!("{}", String::from_utf8_lossy(data));
//         } else {
//             println!(
//                 "Header: {:?}\nData:\n{}",
//                 String::from_utf8_lossy(_header),
//                 String::from_utf8_lossy(data)
//             );
//         }
//     } else {
//         panic!("Invalid object format");
//     }
// }

use crate::utils::decompress_data;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

struct ObjectInfo {
    hash: String,
    path: PathBuf,
    modified: SystemTime,
}

pub fn read_object(hash: String, pretty: bool) {
    // Search for matching objects
    let matches = find_matching_objects(&hash);

    if matches.is_empty() {
        eprintln!("No objects found matching hash prefix: {}", hash);
        std::process::exit(1);
    }

    let selected_object = if matches.len() == 1 {
        // Only one match, use it directly
        &matches[0]
    } else {
        // Multiple matches, let user choose
        println!("Found {} object(s) matching '{}':\n", matches.len(), hash);
        match select_object(&matches) {
            Some(obj) => obj,
            None => {
                println!("Invalid selection. Exiting.");
                std::process::exit(1);
            }
        }
    };

    // Read and display the selected object
    display_object(&selected_object.path, pretty);
}

fn find_matching_objects(hash_prefix: &str) -> Vec<ObjectInfo> {
    let mut matches = Vec::new();
    let objects_dir = ".kitcat/objects";

    if let Ok(entries) = fs::read_dir(objects_dir) {
        for entry in entries.flatten() {
            if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                let prefix = entry.file_name().to_string_lossy().to_string();

                for sub_entry in sub_entries.flatten() {
                    let filename = sub_entry.file_name().to_string_lossy().to_string();
                    let full_hash = format!("{}{}", prefix, filename);

                    // Check if this hash starts with the given prefix
                    if full_hash.starts_with(hash_prefix) {
                        if let Ok(metadata) = sub_entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                matches.push(ObjectInfo {
                                    hash: full_hash,
                                    path: sub_entry.path(),
                                    modified,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by modification time (newest first)
    matches.sort_by(|a, b| b.modified.cmp(&a.modified));
    matches
}

fn select_object(matches: &[ObjectInfo]) -> Option<&ObjectInfo> {
    let mut start_idx = 0;
    let page_size = 10;

    loop {
        let end_idx = (start_idx + page_size).min(matches.len());
        let current_page = &matches[start_idx..end_idx];

        // Display current page
        for (i, obj) in current_page.iter().enumerate() {
            let index = start_idx + i + 1;
            let time = format_time(&obj.modified);
            println!("{}. {} (modified: {})", index, obj.hash, time);
        }

        // Show navigation hints
        println!();
        if end_idx < matches.len() {
            println!("Press 'm' for more, Enter to advance one, or enter a number to select:");
        } else {
            println!("Enter a number to select (1-{}):", matches.len());
        }
        print!(":-) ");
        io::stdout().flush().unwrap();

        // Get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Handle input
        if input.is_empty() && end_idx < matches.len() {
            // Enter pressed - advance one
            start_idx = end_idx;
            println!();
            continue;
        } else if input.eq_ignore_ascii_case("m") && end_idx < matches.len() {
            // 'm' pressed - show next page
            start_idx = end_idx;
            println!();
            continue;
        } else if let Ok(choice) = input.parse::<usize>() {
            // Number entered - validate and return
            if choice > 0 && choice <= matches.len() {
                return Some(&matches[choice - 1]);
            } else {
                println!(
                    "Invalid selection. Please enter a number between 1 and {}.",
                    matches.len()
                );
                return None;
            }
        } else {
            // Invalid input
            println!("Invalid input.");
            return None;
        }
    }
}

fn format_time(time: &SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0);
            match datetime {
                Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => "Unknown".to_string(),
            }
        }
        Err(_) => "Unknown".to_string(),
    }
}

fn display_object(path: &PathBuf, pretty: bool) {
    let compressed = fs::read(path).expect("Failed to read object");
    let decompressed = decompress_data(&compressed);

    if let Some(pos) = decompressed.iter().position(|&b| b == 0) {
        let header = &decompressed[..pos];
        let data = &decompressed[pos + 1..];

        if pretty {
            print!("{}", String::from_utf8_lossy(data));
        } else {
            println!(
                "Header: {:?}\nData:\n{}",
                String::from_utf8_lossy(header),
                String::from_utf8_lossy(data)
            );
        }
    } else {
        panic!("Invalid object format");
    }
}
