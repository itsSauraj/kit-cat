mod commands;
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
    }
}
