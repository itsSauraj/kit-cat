pub mod commands;

// Re-export functions
pub use commands::add_to_index;
pub use commands::hash_file;
pub use commands::init;
pub use commands::read_file;
pub use commands::read_head;
pub use commands::read_index;
pub use commands::write_head;
