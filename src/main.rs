mod commands;
mod config;
mod index;
mod models;
mod object;
mod repo;
mod utils;

use clap::{Parser, Subcommand};
use commands::*;

/// Command line interface for KitKat
#[derive(Parser)]
#[command(name = "kitkat")]
#[command(version = "0.1.0")]
#[command(about = "A minimal Git implementation in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for KitKat
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init,
    /// Compute object ID and optionally create a blob from a file
    HashObject {
        /// The file to hash
        file: String,
    },
    /// Read the content of an object
    ReadFile {
        /// Pretty-print the object content
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// The hash of the object to read
        #[arg(short = 's', long = "hash")]
        hash: String,
    },
    /// Add file contents to the index
    Add {
        /// The file to add
        file: String,
    },
    /// Read the index
    ReadIndex,
    /// Write a value to HEAD
    WriteHead {
        /// The value to write
        value: String,
    },
    /// Read the current HEAD
    ReadHead,
    /// Write a tree from the current index
    WriteTree,
    /// List the contents of a tree
    ListTree {
        /// The hash of the tree to list
        hash: String,
    },
    /// Create a commit
    Commit {
        /// Commit message
        #[arg(short = 'm', long = "message")]
        message: String,
    },
    /// Show commit details
    ShowCommit {
        /// The hash of the commit to show
        hash: String,
    },
    /// Set or get a config value
    Config {
        /// Config key (e.g., user.name, user.email)
        key: String,
        /// Config value (if setting)
        value: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::HashObject { file } => {
            let hash = hash_file(file);
            println!("{}", hash);
        }
        Commands::ReadFile { pretty, hash } => read_file(hash, pretty),
        Commands::Add { file } => add_to_index(file),
        Commands::ReadIndex => {
            let index = read_index();
            for entry in index {
                println!("{} {}", entry.hash, entry.path);
            }
        }
        Commands::WriteHead { value } => write_head(&value),
        Commands::ReadHead => {
            let head = read_head();
            println!("{}", head);
        }
        Commands::WriteTree => {
            write_tree();
        }
        Commands::ListTree { hash } => list_tree(hash),
        Commands::Commit { message } => commit(message),
        Commands::ShowCommit { hash } => show_commit_cmd(hash),
        Commands::Config { key, value } => {
            if let Some(val) = value {
                set_config_cmd(key, val);
            } else {
                get_config_cmd(key);
            }
        }
    }
}
