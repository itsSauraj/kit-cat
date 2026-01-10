pub mod branch;
pub mod commands;
pub mod log;

// Re-export functions
pub use branch::{create_branch, delete_branch, list_branches, show_current_branch, switch_branch};
pub use log::{log, log_with_filter, LogFormat};
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
