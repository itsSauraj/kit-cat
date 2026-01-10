pub mod commands;

// Re-export functions
pub use commands::add_to_index;
pub use commands::commit;
pub use commands::get_config_cmd;
pub use commands::hash_file;
pub use commands::init;
pub use commands::list_tree;
pub use commands::read_file;
pub use commands::read_head;
pub use commands::read_index;
pub use commands::set_config_cmd;
pub use commands::show_commit_cmd;
pub use commands::write_head;
pub use commands::write_tree;
